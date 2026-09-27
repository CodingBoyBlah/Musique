// ─────────────────────────────────────────────────────────────────────────────
// Taste engine
//
// Spotify killed /recommendations, /audio-features and /related-artists for
// third-party apps (Nov 2024), so we build the model ourselves from signals we
// own. The engine is:
//
//   • Self-improving — every play / full-listen / skip the user makes is stored
//     and folded into a recency-decayed artist-affinity profile.
//   • Fresh-first — it explicitly refuses to resurface tracks the user already
//     liked years ago: for the home feed, anything already in the library is
//     dropped from the primary pool and older catalogue cuts rank below new
//     releases from artists the user actually listens to.
//   • Lightweight — no ML runtime, no training. A handful of indexed SQL
//     queries over a few thousand local rows + a small in-memory cache of
//     Spotify artist lookups. Ranking is O(limit * candidates).
//
// Pipeline: signals → artist affinity → candidate generation (artist top
// tracks + newest releases) → weighted scoring (affinity, freshness, novelty,
// popularity) → MMR re-ranking for artist/album diversity.
// ─────────────────────────────────────────────────────────────────────────────

use crate::commands::spotify::{item_from_album_simple, item_from_album_track, item_from_track};
use crate::errors::AppError;
use crate::spotify::{self, types::*};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::{Duration, Instant};

const BASE: &str = "https://api.spotify.com/v1";

// signal tuning
const HALF_LIFE_DAYS: f64 = 21.0;
const LIKE_HALF_LIFE_DAYS: f64 = 120.0;
const EVENT_WINDOW_DAYS: i64 = 180;
const HOME_RECENT_PLAY_EXCLUDE_DAYS: i64 = 10;
const RADIO_RECENT_PLAY_EXCLUDE_DAYS: i64 = 2;
const SKIP_EXCLUDE_DAYS: i64 = 21;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn now_days() -> i64 {
    now_ms() / 86_400_000
}

// ─── caches ──────────────────────────────────────────────────────────────────

static DEFAULT_RECS_CACHE: RwLock<Option<(Instant, Vec<TrackItem>)>> = RwLock::new(None);
static PROFILE_CACHE: RwLock<Option<(Instant, TasteProfile)>> = RwLock::new(None);
static TOP_TRACKS_CACHE: RwLock<Option<HashMap<String, (Instant, Vec<TrackItem>)>>> = RwLock::new(None);
static ARTIST_ALBUMS_CACHE: RwLock<Option<HashMap<String, (Instant, Vec<AlbumItem>)>>> = RwLock::new(None);
static ALBUM_TRACKS_CACHE: RwLock<Option<HashMap<String, (Instant, Vec<TrackItem>)>>> = RwLock::new(None);

/// Drop everything we've memoized. Called on logout and whenever a fresh
/// listening signal lands so the next rec request reflects it immediately.
pub fn clear_caches() {
    if let Ok(mut g) = DEFAULT_RECS_CACHE.write() { *g = None; }
    if let Ok(mut g) = PROFILE_CACHE.write()      { *g = None; }
    if let Ok(mut g) = TOP_TRACKS_CACHE.write()   { *g = None; }
    if let Ok(mut g) = ARTIST_ALBUMS_CACHE.write(){ *g = None; }
    if let Ok(mut g) = ALBUM_TRACKS_CACHE.write() { *g = None; }
}

/// Drop only what a listening event changes: the taste profile and the feed
/// built from it. The artist/album caches are Spotify catalog data a play or
/// skip says nothing about, and refilling them costs dozens of API calls.
fn clear_taste_caches() {
    if let Ok(mut g) = DEFAULT_RECS_CACHE.write() { *g = None; }
    if let Ok(mut g) = PROFILE_CACHE.write()      { *g = None; }
}

/// Events since the log was last trimmed. The trim scans the whole log, so it
/// runs every `TRIM_EVERY` events instead of on each one.
static EVENTS_SINCE_TRIM: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const TRIM_EVERY: u32 = 50;

// ─── event capture ───────────────────────────────────────────────────────────

/// Record one listening signal and roll it into the per-track aggregate.
/// `event_type` is play | complete | skip.
pub async fn record_event(
    pool:         &SqlitePool,
    track_id:     &str,
    event_type:   &str,
    ms_played:    i64,
    duration_ms:  i64,
    context_type: Option<&str>,
    context_id:   Option<&str>,
) -> Result<(), AppError> {
    if track_id.is_empty() {
        return Ok(());
    }
    let ev = match event_type {
        "complete" => "complete",
        "skip"     => "skip",
        _          => "play",
    };
    let now = now_ms();
    let ms_played   = ms_played.max(0);
    let duration_ms = duration_ms.max(0);

    sqlx::query(
        "INSERT INTO listen_events
             (track_id, event_type, ms_played, track_duration_ms, context_type, context_id, occurred_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(track_id)
    .bind(ev)
    .bind(ms_played)
    .bind(duration_ms)
    .bind(context_type)
    .bind(context_id)
    .bind(now)
    .execute(pool)
    .await?;

    let early = if duration_ms > 0 && ms_played * 2 < duration_ms { 1 } else { 0 };

    let (play_inc, comp_inc, skip_inc, early_inc, last_play, last_skip, last_comp) = match ev {
        "complete" => (0, 1, 0, 0, None, None, Some(now)),
        "skip"     => (0, 0, 1, early, None, Some(now), None),
        _          => (1, 0, 0, 0, Some(now), None, None),
    };

    sqlx::query(
        "INSERT INTO track_stats
             (track_id, play_count, complete_count, skip_count, early_skip_count,
              last_played_at, last_skipped_at, last_completed_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(track_id) DO UPDATE SET
             play_count       = play_count       + excluded.play_count,
             complete_count   = complete_count   + excluded.complete_count,
             skip_count       = skip_count       + excluded.skip_count,
             early_skip_count = early_skip_count + excluded.early_skip_count,
             last_played_at   = COALESCE(excluded.last_played_at,   last_played_at),
             last_skipped_at  = COALESCE(excluded.last_skipped_at,  last_skipped_at),
             last_completed_at= COALESCE(excluded.last_completed_at,last_completed_at),
             updated_at       = excluded.updated_at",
    )
    .bind(track_id)
    .bind(play_inc)
    .bind(comp_inc)
    .bind(skip_inc)
    .bind(early_inc)
    .bind(last_play)
    .bind(last_skip)
    .bind(last_comp)
    .bind(now)
    .execute(pool)
    .await?;

    // keep the log bounded: age it out and cap the row count. counter starts
    // at 0, so the first event of each session also trims
    use std::sync::atomic::Ordering;
    if EVENTS_SINCE_TRIM.fetch_add(1, Ordering::Relaxed) % TRIM_EVERY == 0 {
        let cutoff = now - EVENT_WINDOW_DAYS * 86_400_000;
        let _ = sqlx::query("DELETE FROM listen_events WHERE occurred_at < ?")
            .bind(cutoff)
            .execute(pool)
            .await;
        let _ = sqlx::query(
            "DELETE FROM listen_events
             WHERE id NOT IN (SELECT id FROM listen_events ORDER BY occurred_at DESC LIMIT 12000)",
        )
        .execute(pool)
        .await;
    }

    // the user just told us something — don't serve a stale profile/feed
    clear_taste_caches();

    Ok(())
}

// ─── taste profile ───────────────────────────────────────────────────────────

#[derive(Default, Clone)]
pub struct TasteProfile {
    pub affinity:        HashMap<String, f64>,
    pub max_affinity:    f64,
    pub played_recently: HashSet<String>,
    pub library_tracks:  HashSet<String>,
    pub last_played_at:  HashMap<String, i64>,
    pub last_skipped_at: HashMap<String, i64>,
    pub skip_count:      HashMap<String, i64>,
}

impl TasteProfile {
    fn affinity_norm(&self, artist_id: &str) -> f64 {
        let a = self.affinity.get(artist_id).copied().unwrap_or(0.0);
        if self.max_affinity <= 0.0 { 0.0 } else { (a / self.max_affinity).clamp(-0.5, 1.3) }
    }
}

async fn load_profile(pool: &SqlitePool) -> TasteProfile {
    if let Ok(g) = PROFILE_CACHE.read() {
        if let Some((ts, p)) = &*g {
            if ts.elapsed() < Duration::from_secs(90) {
                return p.clone();
            }
        }
    }

    let now = now_ms();
    let event_cutoff = now - EVENT_WINDOW_DAYS * 86_400_000;
    let recent_cutoff = now - 30 * 86_400_000;

    let mut aff: HashMap<String, f64> = HashMap::new();

    // 1. behaviour: real plays, completions and skips, recency-decayed
    if let Ok(rows) = sqlx::query_as::<_, (String, i64, String, i64, i64, i64)>(
        "SELECT ta.artist_id, ta.position, le.event_type, le.ms_played, le.track_duration_ms, le.occurred_at
         FROM listen_events le
         JOIN track_artists ta ON ta.track_id = le.track_id
         WHERE le.occurred_at >= ?
         LIMIT 20000",
    )
    .bind(event_cutoff)
    .fetch_all(pool)
    .await
    {
        let half = HALF_LIFE_DAYS * 86_400_000.0;
        for (aid, pos, ev, ms, dur, at) in rows {
            if aid.is_empty() { continue; }
            let decay = 0.5f64.powf(((now - at).max(0) as f64) / half);
            let base = match ev.as_str() {
                "complete" => 1.5,
                "skip"     => if dur > 0 && ms * 2 < dur { -1.7 } else { -0.9 },
                _          => 1.0,
            };
            let w = if pos == 0 { 1.0 } else { 0.35 };
            *aff.entry(aid).or_insert(0.0) += base * decay * w;
        }
    }

    // 2. Spotify top artists across all three time ranges, short term weighted most
    if let Ok(rows) = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT artist_id, time_range, position FROM top_artists",
    )
    .fetch_all(pool)
    .await
    {
        for (aid, range, pos) in rows {
            if aid.is_empty() { continue; }
            let wr = match range.as_str() {
                "short_term"  => 7.0,
                "medium_term" => 3.5,
                _             => 1.5,
            };
            let pf = ((50 - pos).max(0) as f64) / 50.0 + 0.2;
            *aff.entry(aid).or_insert(0.0) += wr * pf;
        }
    }

    // 3. followed artists
    if let Ok(rows) = sqlx::query_as::<_, (String,)>("SELECT artist_id FROM followed_artists")
        .fetch_all(pool)
        .await
    {
        for (aid,) in rows {
            if !aid.is_empty() { *aff.entry(aid).or_insert(0.0) += 5.0; }
        }
    }

    // 4. liked-song artists, weighted by how recently the song was liked so old
    //    library love fades instead of dominating forever
    if let Ok(rows) = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT ta.artist_id, ta.position, st.added_at
         FROM saved_tracks st JOIN track_artists ta ON ta.track_id = st.track_id
         LIMIT 4000",
    )
    .fetch_all(pool)
    .await
    {
        let half = LIKE_HALF_LIFE_DAYS * 86_400_000.0;
        for (aid, pos, added) in rows {
            if aid.is_empty() { continue; }
            let decay = 0.5f64.powf(((now - added).max(0) as f64) / half);
            let w = if pos == 0 { 1.0 } else { 0.4 };
            *aff.entry(aid).or_insert(0.0) += 3.0 * decay * w;
        }
    }

    // 5. saved-album artists
    if let Ok(rows) = sqlx::query_as::<_, (String, i64)>(
        "SELECT aa.artist_id, sa.added_at
         FROM saved_albums sa JOIN album_artists aa ON aa.album_id = sa.album_id
         LIMIT 4000",
    )
    .fetch_all(pool)
    .await
    {
        let half = LIKE_HALF_LIFE_DAYS * 86_400_000.0;
        for (aid, added) in rows {
            if aid.is_empty() { continue; }
            let decay = 0.5f64.powf(((now - added).max(0) as f64) / half);
            *aff.entry(aid).or_insert(0.0) += 2.0 * decay;
        }
    }

    // 6. negative/positive per-track state + library membership
    let mut played_recently: HashSet<String> = HashSet::new();
    if let Ok(rows) = sqlx::query_as::<_, (String, String)>(
        "SELECT DISTINCT track_id, event_type FROM listen_events WHERE occurred_at >= ?",
    )
    .bind(recent_cutoff)
    .fetch_all(pool)
    .await
    {
        for (tid, ev) in rows {
            if ev != "skip" { played_recently.insert(tid); }
        }
    }
    // fold in Spotify's synced recently-played so a brand-new install still has signal
    if let Ok(rows) = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT track_id FROM recently_played WHERE played_at >= ?",
    )
    .bind(now - 14 * 86_400_000)
    .fetch_all(pool)
    .await
    {
        for (tid,) in rows { played_recently.insert(tid); }
    }

    let library_tracks: HashSet<String> = sqlx::query_as::<_, (String,)>(
        "SELECT track_id FROM saved_tracks
         UNION SELECT track_id FROM top_tracks",
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|(t,)| t).collect())
    .unwrap_or_default();

    let mut last_played_at:  HashMap<String, i64> = HashMap::new();
    let mut last_skipped_at: HashMap<String, i64> = HashMap::new();
    let mut skip_count:      HashMap<String, i64> = HashMap::new();
    if let Ok(rows) = sqlx::query_as::<_, (String, Option<i64>, Option<i64>, i64)>(
        "SELECT track_id, last_played_at, last_skipped_at, skip_count FROM track_stats",
    )
    .fetch_all(pool)
    .await
    {
        for (tid, lp, ls, sc) in rows {
            if let Some(t) = lp { last_played_at.insert(tid.clone(), t); }
            if let Some(t) = ls { last_skipped_at.insert(tid.clone(), t); }
            if sc > 0 { skip_count.insert(tid, sc); }
        }
    }

    let max_affinity = aff.values().cloned().fold(0.0f64, f64::max).max(1.0);
    let profile = TasteProfile {
        affinity: aff,
        max_affinity,
        played_recently,
        library_tracks,
        last_played_at,
        last_skipped_at,
        skip_count,
    };

    if let Ok(mut g) = PROFILE_CACHE.write() {
        *g = Some((Instant::now(), profile.clone()));
    }
    profile
}

// ─── release-date helpers ────────────────────────────────────────────────────

/// "YYYY" / "YYYY-MM" / "YYYY-MM-DD" → days since the unix epoch (approx, day 1 default)
fn release_days(d: &str) -> Option<i64> {
    let mut it = d.split('-');
    let y: i64 = it.next()?.trim().parse().ok()?;
    let m: i64 = it.next().and_then(|s| s.trim().parse().ok()).unwrap_or(1);
    let day: i64 = it.next().and_then(|s| s.trim().parse().ok()).unwrap_or(1);
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

fn release_key(d: Option<&str>) -> i64 {
    let Some(d) = d else { return 0 };
    release_days(d).unwrap_or(0)
}

/// 0..1 freshness, half-life of two years (a brand-new drop ≈ 1.0)
fn freshness(d: Option<&str>) -> f64 {
    let Some(d) = d else { return 0.0 };
    let Some(rel) = release_days(d) else { return 0.0 };
    let age = (now_days() - rel) as f64;
    if age <= 0.0 { return 1.0; }
    0.5f64.powf(age / 730.0)
}

// ─── spotify lookups (cached) ────────────────────────────────────────────────

async fn cached_top_tracks(token: &str, aid: &str) -> Option<Vec<TrackItem>> {
    if let Ok(g) = TOP_TRACKS_CACHE.read() {
        if let Some(map) = &*g {
            if let Some((ts, v)) = map.get(aid) {
                if ts.elapsed() < Duration::from_secs(3600) {
                    return Some(v.clone());
                }
            }
        }
    }
    let url = format!("{BASE}/artists/{aid}/top-tracks?market=from_token");
    let tt = spotify::spotify_get::<SpTopTracks>(token, &url).await.ok()?;
    let items: Vec<TrackItem> = tt.tracks.iter()
        .filter(|t| !t.is_local.unwrap_or(false))
        .map(item_from_track)
        .collect();
    if let Ok(mut g) = TOP_TRACKS_CACHE.write() {
        let map = g.get_or_insert_with(HashMap::new);
        if map.len() > 600 { map.clear(); }
        map.insert(aid.to_string(), (Instant::now(), items.clone()));
    }
    Some(items)
}

async fn cached_artist_albums(token: &str, aid: &str) -> Option<Vec<AlbumItem>> {
    if let Ok(g) = ARTIST_ALBUMS_CACHE.read() {
        if let Some(map) = &*g {
            if let Some((ts, v)) = map.get(aid) {
                if ts.elapsed() < Duration::from_secs(6 * 3600) {
                    return Some(v.clone());
                }
            }
        }
    }
    let url = format!(
        "{BASE}/artists/{aid}/albums?include_groups=album,single&market=from_token&limit=10"
    );
    let page = spotify::spotify_get::<SpPage<SpAlbumSimple>>(token, &url).await.ok()?;
    let items: Vec<AlbumItem> = page.items.iter().map(item_from_album_simple).collect();
    if let Ok(mut g) = ARTIST_ALBUMS_CACHE.write() {
        let map = g.get_or_insert_with(HashMap::new);
        if map.len() > 600 { map.clear(); }
        map.insert(aid.to_string(), (Instant::now(), items.clone()));
    }
    Some(items)
}

fn album_item_from_sp(al: &SpAlbum) -> AlbumItem {
    AlbumItem {
        id:           al.id.clone(),
        name:         al.name.clone(),
        album_type:   al.album_type.clone(),
        image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        artists:      al.artists.as_ref()
                        .map(|v| v.iter().map(|a| ArtistItem {
                            id: a.id.clone(), name: a.name.clone(), image_url: None, popularity: None,
                        }).collect())
                        .unwrap_or_default(),
        popularity:   al.popularity,
    }
}

async fn cached_album_tracks(token: &str, album_id: &str) -> Option<Vec<TrackItem>> {
    if let Ok(g) = ALBUM_TRACKS_CACHE.read() {
        if let Some(map) = &*g {
            if let Some((ts, v)) = map.get(album_id) {
                if ts.elapsed() < Duration::from_secs(6 * 3600) {
                    return Some(v.clone());
                }
            }
        }
    }
    let url = format!("{BASE}/albums/{album_id}?market=from_token");
    let al = spotify::spotify_get::<SpAlbum>(token, &url).await.ok()?;
    let album_item = album_item_from_sp(&al);
    let items: Vec<TrackItem> = al.tracks.as_ref()
        .map(|p| p.items.iter().filter(|t| !t.is_local.unwrap_or(false)).map(|t| {
            let mut item = item_from_album_track(t);
            item.album = Some(album_item.clone());
            item
        }).collect())
        .unwrap_or_default();
    if let Ok(mut g) = ALBUM_TRACKS_CACHE.write() {
        let map = g.get_or_insert_with(HashMap::new);
        if map.len() > 800 { map.clear(); }
        map.insert(album_id.to_string(), (Instant::now(), items.clone()));
    }
    Some(items)
}

// ─── candidate generation ────────────────────────────────────────────────────

#[derive(Clone)]
struct Candidate {
    track:    TrackItem,
    is_fresh: bool,
}

async fn gather_artist(token: &str, aid: &str, want_fresh: bool) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(tt) = cached_top_tracks(token, aid).await {
        for t in tt {
            if seen.insert(t.id.clone()) {
                out.push(Candidate { track: t, is_fresh: false });
            }
        }
    }

    if want_fresh {
        if let Some(mut albums) = cached_artist_albums(token, aid).await {
            albums.sort_by(|a, b| {
                release_key(b.release_date.as_deref()).cmp(&release_key(a.release_date.as_deref()))
            });
            let today = now_days();
            let mut fetched = 0;
            for al in albums {
                if fetched >= 2 { break; }
                let Some(rd) = al.release_date.as_deref() else { continue };
                let age = today - release_days(rd).unwrap_or(today);
                if age > 3 * 365 { break; }
                if let Some(tracks) = cached_album_tracks(token, &al.id).await {
                    for t in tracks {
                        if seen.insert(t.id.clone()) {
                            out.push(Candidate { track: t, is_fresh: true });
                        }
                    }
                    fetched += 1;
                }
            }
        }
    }

    out
}

// ─── scoring + MMR selection ─────────────────────────────────────────────────

#[derive(Clone)]
struct Scored {
    track:   TrackItem,
    score:   f64,
    primary: String,
    album:   String,
    artists: Vec<String>,
}

fn score_candidate(c: &Candidate, p: &TasteProfile, is_home: bool) -> f64 {
    let aff_n = c.track.artists.iter()
        .map(|a| p.affinity_norm(&a.id))
        .fold(f64::NEG_INFINITY, f64::max);
    let aff_n = if aff_n.is_finite() { aff_n } else { 0.0 };

    let fresh = freshness(c.track.album.as_ref().and_then(|a| a.release_date.as_deref()));

    let novelty = match p.last_played_at.get(&c.track.id) {
        None    => 1.0,
        Some(t) => (((now_ms() - t).max(0) as f64) / (45.0 * 86_400_000.0)).clamp(0.0, 1.0),
    };

    let pop = c.track.popularity.unwrap_or(50) as f64 / 100.0;
    let in_library = p.library_tracks.contains(&c.track.id);
    let lib_pen = if in_library { if is_home { 1.0 } else { 0.45 } } else { 0.0 };
    let skip_pen = (p.skip_count.get(&c.track.id).copied().unwrap_or(0) as f64 * 0.12).min(0.7);

    let mut score = 3.0 * aff_n
        + 1.7 * fresh
        + 0.9 * novelty
        + 0.35 * (1.0 - pop)
        - 2.4 * lib_pen
        - 1.5 * skip_pen;
    if c.is_fresh { score += 0.5; }

    // small jitter so refreshes aren't identical while staying taste-consistent
    score * (1.0 + (rand::random::<f64>() - 0.5) * 0.24)
}

fn similarity(a: &Scored, b: &Scored) -> f64 {
    if !a.primary.is_empty() && a.primary == b.primary { return 1.0; }
    if !a.album.is_empty() && a.album == b.album { return 0.9; }
    if a.artists.iter().any(|x| b.artists.contains(x)) { return 0.45; }
    0.0
}

fn select_mmr(pool: &[Scored], limit: usize) -> Vec<Scored> {
    let lambda = 0.72;
    let mut chosen: Vec<Scored> = Vec::new();
    let mut used = vec![false; pool.len()];
    let mut artist_count: HashMap<String, usize> = HashMap::new();
    let mut album_count:  HashMap<String, usize> = HashMap::new();

    while chosen.len() < limit {
        let mut best: Option<usize> = None;
        let mut best_mmr = f64::NEG_INFINITY;
        for (i, c) in pool.iter().enumerate() {
            if used[i] { continue; }
            if artist_count.get(&c.primary).copied().unwrap_or(0) >= 2 { continue; }
            if !c.album.is_empty() && album_count.get(&c.album).copied().unwrap_or(0) >= 2 { continue; }

            let redundancy = chosen.iter().map(|s| similarity(c, s)).fold(0.0, f64::max);
            let mmr = lambda * c.score - (1.0 - lambda) * redundancy;
            if mmr > best_mmr {
                best_mmr = mmr;
                best = Some(i);
            }
        }
        let Some(i) = best else { break };
        used[i] = true;
        let c = &pool[i];
        *artist_count.entry(c.primary.clone()).or_insert(0) += 1;
        if !c.album.is_empty() {
            *album_count.entry(c.album.clone()).or_insert(0) += 1;
        }
        chosen.push(c.clone());
    }
    chosen
}

// ─── main entry point ────────────────────────────────────────────────────────

pub async fn recommend(
    token:             &str,
    pool:              &SqlitePool,
    seed_artist_ids:   Vec<String>,
    exclude_track_ids: HashSet<String>,
    limit:             usize,
    is_default_home:   bool,
) -> Result<Vec<TrackItem>, AppError> {
    let limit = limit.clamp(1, 100);

    if is_default_home {
        if let Ok(g) = DEFAULT_RECS_CACHE.read() {
            if let Some((ts, recs)) = &*g {
                if ts.elapsed() < Duration::from_secs(5 * 60) && recs.len() >= limit.min(10) {
                    let mut cached = recs.clone();
                    cached.truncate(limit);
                    return Ok(cached);
                }
            }
        }
    }

    let profile = load_profile(pool).await;

    let filter_explicit = crate::auth::get_setting_value(pool, "spotify_explicit_filter")
        .await
        .ok()
        .flatten()
        .as_deref()
        == Some("1");

    // ── seeds ────────────────────────────────────────────────────────────────
    let mut seeds: Vec<String> = seed_artist_ids.into_iter().filter(|s| !s.is_empty()).collect();
    let provided: HashSet<String> = seeds.iter().cloned().collect();

    let known = crate::library::gather_known_artists(pool, token).await;
    for a in known {
        if !a.is_empty() && !seeds.contains(&a) {
            seeds.push(a);
        }
    }
    if seeds.is_empty() {
        return Ok(local_fallback(pool, limit).await);
    }

    let mut seed_scores: Vec<(String, f64)> = Vec::new();
    let mut seen_seeds: HashSet<String> = HashSet::new();
    for a in seeds {
        if !seen_seeds.insert(a.clone()) { continue; }
        let mut sc = profile.affinity.get(&a).copied().unwrap_or(0.0) + 0.5;
        if provided.contains(&a) { sc += 100.0; }
        sc *= 1.0 + (rand::random::<f64>() - 0.5) * 0.35;
        seed_scores.push((a, sc));
    }
    seed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let sample = if limit <= 16 { 10 } else { 14 };
    seed_scores.truncate(sample);
    let chosen_seeds: Vec<String> = seed_scores.iter().map(|(a, _)| a.clone()).collect();

    // artists we trust most get the extra (cached) newest-release lookup
    let fresh_cutoff = profile.affinity.get(&chosen_seeds[0]).copied().unwrap_or(0.0) * 0.35;
    let fresh_set: HashSet<String> = chosen_seeds.iter()
        .filter(|a| profile.affinity.get(*a).copied().unwrap_or(0.0) >= fresh_cutoff)
        .cloned()
        .collect();

    // ── fetch candidates concurrently ─────────────────────────────────────────
    let mut handles = Vec::with_capacity(chosen_seeds.len());
    for aid in &chosen_seeds {
        let token = token.to_string();
        let aid = aid.clone();
        let want_fresh = fresh_set.contains(&aid);
        handles.push(tokio::spawn(async move { gather_artist(&token, &aid, want_fresh).await }));
    }
    let mut candidates: Vec<Candidate> = Vec::new();
    for h in handles {
        if let Ok(v) = h.await {
            candidates.extend(v);
        }
    }

    // ── filter + score ────────────────────────────────────────────────────────
    let now = now_ms();
    let play_window = if is_default_home {
        HOME_RECENT_PLAY_EXCLUDE_DAYS
    } else {
        RADIO_RECENT_PLAY_EXCLUDE_DAYS
    } * 86_400_000;
    let skip_window = SKIP_EXCLUDE_DAYS * 86_400_000;

    let mut scored: Vec<Scored> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for c in candidates {
        let t = &c.track;
        if t.id.is_empty() || !seen.insert(t.id.clone()) { continue; }
        if exclude_track_ids.contains(&t.id) { continue; }
        if filter_explicit && t.explicit { continue; }

        // never resurface what's actively being rotated
        if let Some(lp) = profile.last_played_at.get(&t.id) {
            if now - lp < play_window { continue; }
        }
        // drop anything the user bailed on recently
        if let Some(ls) = profile.last_skipped_at.get(&t.id) {
            if now - ls < skip_window { continue; }
        }
        // Spotify-synced plays carry no precise timestamp, so keep them off the
        // home feed entirely (they're still fine as radio seeds)
        if is_default_home
            && !profile.last_played_at.contains_key(&t.id)
            && profile.played_recently.contains(&t.id)
        {
            continue;
        }

        let primary = t.artists.first().map(|a| a.id.clone()).unwrap_or_default();
        let album   = t.album.as_ref().map(|a| a.id.clone()).unwrap_or_default();
        let artists = t.artists.iter().map(|a| a.id.clone()).collect();
        let score   = score_candidate(&c, &profile, is_default_home);
        scored.push(Scored { track: c.track, score, primary, album, artists });
    }

    let mut out: Vec<TrackItem> = if is_default_home {
        // primary pool: everything the user does NOT already have saved/liked,
        // which is the whole point — no rehashing 4-year-old library tracks
        let fresh_pool: Vec<Scored> = scored.iter()
            .filter(|s| !profile.library_tracks.contains(&s.track.id))
            .cloned()
            .collect();
        select_mmr(&fresh_pool, limit).into_iter().map(|s| s.track).collect()
    } else {
        select_mmr(&scored, limit).into_iter().map(|s| s.track).collect()
    };

    // relax if the strict pool couldn't fill the request
    if out.len() < limit {
        let have: HashSet<String> = out.iter().map(|t| t.id.clone()).collect();
        let rest: Vec<Scored> = scored.into_iter().filter(|s| !have.contains(&s.track.id)).collect();
        let more = select_mmr(&rest, limit - out.len());
        out.extend(more.into_iter().map(|s| s.track));
    }

    if out.is_empty() {
        out = local_fallback(pool, limit).await;
    }

    if is_default_home && !out.is_empty() {
        if let Ok(mut g) = DEFAULT_RECS_CACHE.write() {
            *g = Some((Instant::now(), out.clone()));
        }
    }

    Ok(out)
}

/// Last-resort pool straight from the local catalogue (offline / brand-new
/// account with no artist graph yet). Still avoids recently played tracks.
async fn local_fallback(pool: &SqlitePool, limit: usize) -> Vec<TrackItem> {
    let rows: Vec<(String, String, i64, i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, al.name, al.image_url
         FROM tracks t
         LEFT JOIN albums al ON al.id = t.album_id
         LEFT JOIN track_stats ts ON ts.track_id = t.id
         WHERE ts.last_played_at IS NULL
            OR ts.last_played_at < ?
         ORDER BY RANDOM() LIMIT ?",
    )
    .bind(now_ms() - HOME_RECENT_PLAY_EXCLUDE_DAYS * 86_400_000)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|(id, name, dur, explicit, al_name, al_img)| TrackItem {
            id,
            name,
            duration_ms: dur,
            explicit: explicit == 1,
            artists: Vec::new(),
            album: Some(AlbumItem {
                id: String::new(),
                name: al_name.unwrap_or_default(),
                album_type: "album".to_string(),
                image_url: al_img,
                release_date: None,
                artists: Vec::new(),
                popularity: None,
            }),
            popularity: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_days_parses_partial_and_full_dates() {
        assert!(release_days("2020").is_some());
        assert!(release_days("2020-05").is_some());
        assert!(release_days("2020-05-17").is_some());
        assert!(release_days("garbage").is_none());
        assert!(release_days("2020-05-17").unwrap() > release_days("2020-01-01").unwrap());
    }

    #[test]
    fn freshness_favours_new_releases() {
        let now = freshness(Some("2999-01-01"));
        assert!((0.0..=1.0).contains(&now));
        assert!(freshness(Some("2000-01-01")) < 0.05);
    }

    fn sc(id: &str, primary: &str, album: &str, score: f64) -> Scored {
        Scored {
            track: TrackItem {
                id: id.into(), name: id.into(), duration_ms: 0, explicit: false,
                artists: Vec::new(), album: None, popularity: None,
            },
            score,
            primary: primary.into(),
            album: album.into(),
            artists: vec![primary.into()],
        }
    }

    #[test]
    fn mmr_caps_repeats_per_artist() {
        let pool: Vec<Scored> = (0..6)
            .map(|i| sc(&format!("t{i}"), "a1", &format!("al{i}"), 1.0 - i as f64 * 0.01))
            .collect();
        // a single artist can only occupy two slots even with a deep pool
        assert_eq!(select_mmr(&pool, 5).len(), 2);
    }

    #[test]
    fn mmr_prefers_a_fresh_artist_over_a_redundant_one() {
        let pool = vec![
            sc("a1", "a1", "x1", 1.0),
            sc("a2", "a1", "x2", 0.5), // same artist as the top pick
            sc("b1", "b2", "y1", 0.6), // different artist
        ];
        let picked = select_mmr(&pool, 3);
        assert_eq!(picked.len(), 3);
        assert_eq!(picked[1].primary, "b2");
    }
}
