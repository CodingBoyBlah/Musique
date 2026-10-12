use crate::{
    db::repos::{albums, artists, tracks},
    errors::AppError,
    spotify::{self, types::*},
};
use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use futures_util::{FutureExt, StreamExt};
use std::collections::HashSet;

const BASE: &str = "https://api.spotify.com/v1";

#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    pub liked_count:       usize,
    pub playlist_count:    usize,
    pub artist_count:      usize,
    pub album_count:       usize,
    pub top_track_count:   usize,
    pub top_artist_count:  usize,
    pub recent_count:      usize,
    pub new_release_count: usize,
}

// ─── timestamp helpers ────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// parse "YYYY-MM-DDTHH:MM:SSZ" into unix millis
/// just falls back to now if it cant parse it
pub(crate) fn parse_iso8601(s: &str) -> i64 {
    let s = s.trim_end_matches('Z');
    let (date, time) = s.split_once('T').unwrap_or((s, "00:00:00"));
    let mut d = date.splitn(3, '-');
    let mut t = time.split_once('.').map(|(s, _)| s).unwrap_or(time).splitn(3, ':');

    let parse = |s: Option<&str>| -> i64 { s.and_then(|v| v.parse().ok()).unwrap_or(0) };

    let y   = parse(d.next());
    let mon = parse(d.next());
    let day = parse(d.next());
    let h   = parse(t.next());
    let min = parse(t.next());
    let sec = parse(t.next());

    if y == 0 { return now_ms(); }

    // howard hinnants civil-to-days algorithm, dont ask me how it works lol
    let y  = if mon <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if mon > 2 { mon - 3 } else { mon + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    (days * 86400 + h * 3600 + min * 60 + sec) * 1000
}

// ─── db upsert helpers ────────────────────────────────────────────────────────
//
// Everything here writes through one connection so a sync step can put its
// whole batch in a single transaction. Autocommitting each row (what this used
// to do) meant a WAL commit per statement - ~6 per liked song - and held the
// sqlite write lock in thousands of tiny slices that every other writer,
// search included, had to queue between.

// One sync step writes at a time. The steps fetch concurrently, but sqlite
// only has one writer, and two long transactions racing for it would just sit
// in busy_timeout (or fail past it).
static SYNC_WRITE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn upsert_artist(conn: &mut SqliteConnection, a: &SpArtist) -> Result<(), AppError> {
    artists::upsert(&mut *conn, &artists::Artist {
        id:         a.id.clone(),
        name:       a.name.clone(),
        image_url:  a.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        genres:     a.genres.as_ref()
                      .filter(|g| !g.is_empty())
                      .map(|g| serde_json::to_string(g).unwrap_or_default()),
        popularity: a.popularity,
        updated_at: 0,
    })
    .await
}

/// Writes tracks with their album/artist rows. A library repeats the same
/// albums and artists constantly, so each one is written once per batch.
#[derive(Default)]
pub(crate) struct DepWriter {
    albums:  HashSet<String>,
    artists: HashSet<String>,
}

impl DepWriter {
    pub(crate) async fn track(&mut self, conn: &mut SqliteConnection, t: &SpTrack) -> Result<(), AppError> {
        if let Some(al) = &t.album {
            if self.albums.insert(al.id.clone()) {
                albums::upsert(&mut *conn, &albums::Album {
                    id:           al.id.clone(),
                    name:         al.name.clone(),
                    album_type:   al.album_type.clone(),
                    image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
                    release_date: al.release_date.clone(),
                    total_tracks: 0,
                    genres:       None,
                    popularity:   None,
                    updated_at:   0,
                })
                .await?;
            }
        }

        for a in &t.artists {
            if !self.artists.insert(a.id.clone()) { continue; }
            let _ = artists::upsert(&mut *conn, &artists::Artist {
                id:         a.id.clone(),
                name:       a.name.clone(),
                image_url:  None,
                genres:     None,
                popularity: None,
                updated_at: 0,
            })
            .await;
        }

        tracks::upsert(&mut *conn, &tracks::Track {
            id:           t.id.clone(),
            name:         t.name.clone(),
            album_id:     t.album.as_ref().map(|a| a.id.clone()),
            duration_ms:  t.duration_ms,
            track_number: t.track_number.unwrap_or(0),
            disc_number:  t.disc_number.unwrap_or(1),
            explicit:     t.explicit,
            popularity:   t.popularity,
            preview_url:  t.preview_url.clone(),
            is_local:     t.is_local.unwrap_or(false),
            updated_at:   0,
            isrc:         t.external_ids.as_ref().and_then(|e| e.isrc.clone()),
        })
        .await?;

        for (i, a) in t.artists.iter().enumerate() {
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO track_artists (track_id, artist_id, position) VALUES (?, ?, ?)",
            )
            .bind(&t.id)
            .bind(&a.id)
            .bind(i as i64)
            .execute(&mut *conn)
            .await;
        }

        Ok(())
    }
}

pub(crate) async fn upsert_track_with_deps(conn: &mut SqliteConnection, t: &SpTrack) -> Result<(), AppError> {
    DepWriter::default().track(conn, t).await
}

async fn upsert_album_simple(conn: &mut SqliteConnection, al: &SpAlbumSimple) -> Result<(), AppError> {
    albums::upsert(&mut *conn, &albums::Album {
        id:           al.id.clone(),
        name:         al.name.clone(),
        album_type:   al.album_type.clone(),
        image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        total_tracks: 0,
        genres:       None,
        popularity:   None,
        updated_at:   0,
    })
    .await?;
    album_artists(conn, &al.id, al.artists.as_deref().unwrap_or(&[])).await;
    Ok(())
}

async fn album_artists(conn: &mut SqliteConnection, album_id: &str, list: &[SpArtistSimple]) {
    for (i, a) in list.iter().enumerate() {
        let _ = sqlx::query("INSERT OR IGNORE INTO artists (id, name, updated_at) VALUES (?, ?, 0)")
            .bind(&a.id)
            .bind(&a.name)
            .execute(&mut *conn)
            .await;
        let _ = albums::add_artist(&mut *conn, album_id, &a.id, i as i64).await;
    }
}

// ─── sync liked songs ─────────────────────────────────────────────────────────

pub async fn sync_liked_songs(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    const LIMIT: i64 = 50;
    let pages: Vec<SpSavedTrackPage> = spotify::get_all_pages(
        token,
        LIMIT,
        |offset| format!("{BASE}/me/tracks?limit={LIMIT}&offset={offset}"),
        |p: &SpSavedTrackPage| p.total,
    )
    .await?;

    // wipe and replace the whole thing in a txn so unlikes actually show up
    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_tracks").execute(&mut *tx).await?;
    let mut deps = DepWriter::default();
    let mut count = 0usize;
    for item in pages.iter().flat_map(|p| &p.items) {
        let Some(track) = &item.track else { continue };
        if track.is_local.unwrap_or(false) { continue; }
        deps.track(&mut tx, track).await?;
        sqlx::query("INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)")
            .bind(&track.id)
            .bind(parse_iso8601(&item.added_at))
            .execute(&mut *tx)
            .await?;
        count += 1;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync playlists ───────────────────────────────────────────────────────────

pub async fn sync_playlists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    const LIMIT: i64 = 50;
    let pages: Vec<SpUserPlaylistPage> = spotify::get_all_pages(
        token,
        LIMIT,
        |offset| format!("{BASE}/me/playlists?limit={LIMIT}&offset={offset}"),
        |p: &SpUserPlaylistPage| p.total,
    )
    .await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    let now = now_ms();
    let mut count = 0usize;
    for item in pages.iter().flat_map(|p| &p.items) {
        let owner_id = item.owner.as_ref().and_then(|o| o.display_name.as_deref()).map(str::to_string);
        sqlx::query(
            "INSERT INTO playlists
                 (id, name, description, owner_id, image_url, total_tracks,
                  is_public, is_local, snapshot_id, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, 0, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 name         = excluded.name,
                 description  = excluded.description,
                 owner_id     = excluded.owner_id,
                 image_url    = excluded.image_url,
                 total_tracks = excluded.total_tracks,
                 snapshot_id  = excluded.snapshot_id,
                 updated_at   = excluded.updated_at",
        )
        .bind(&item.id)
        .bind(&item.name)
        .bind(&item.description)
        .bind(&owner_id)
        .bind(item.images.as_ref().and_then(|v| v.first()).map(|i| i.url.as_str()))
        .bind(item.tracks.as_ref().map(|t| t.total).unwrap_or(0))
        .bind(&item.snapshot_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync saved albums ────────────────────────────────────────────────────────

pub async fn sync_saved_albums(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    const LIMIT: i64 = 50;
    let pages: Vec<SpSavedAlbumPage> = spotify::get_all_pages(
        token,
        LIMIT,
        |offset| format!("{BASE}/me/albums?limit={LIMIT}&offset={offset}"),
        |p: &SpSavedAlbumPage| p.total,
    )
    .await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_albums").execute(&mut *tx).await?;
    let mut count = 0usize;
    for item in pages.iter().flat_map(|p| p.items.iter().flatten()) {
        let al = &item.album;
        albums::upsert(&mut *tx, &albums::Album {
            id:           al.id.clone(),
            name:         al.name.clone(),
            album_type:   al.album_type.clone(),
            image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
            release_date: al.release_date.clone(),
            total_tracks: al.total_tracks.unwrap_or(0),
            genres:       None,
            popularity:   al.popularity,
            updated_at:   0,
        })
        .await?;
        album_artists(&mut tx, &al.id, al.artists.as_deref().unwrap_or(&[])).await;

        sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
            .bind(&al.id)
            .bind(parse_iso8601(&item.added_at))
            .execute(&mut *tx)
            .await?;
        count += 1;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync followed artists ────────────────────────────────────────────────────

pub async fn sync_followed_artists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    // cursor paging, so this one has to walk page by page
    let mut followed: Vec<SpArtist> = Vec::new();
    let mut cursor: Option<String> = None;
    let limit = 50;

    loop {
        let url = match &cursor {
            Some(after) => format!("{BASE}/me/following?type=artist&limit={limit}&after={after}"),
            None        => format!("{BASE}/me/following?type=artist&limit={limit}"),
        };

        let resp: SpFollowedArtistsResponse = spotify::spotify_get(token, &url).await?;
        let page = resp.artists;
        let done = page.next.is_none() || page.items.len() < limit;
        cursor = page.cursors.as_ref().and_then(|c| c.after.clone());
        followed.extend(page.items);
        if done { break; }
    }

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM followed_artists").execute(&mut *tx).await?;
    let now = now_ms();
    for artist in &followed {
        upsert_artist(&mut tx, artist).await?;
        sqlx::query(
            "INSERT OR IGNORE INTO followed_artists (artist_id, followed_at) VALUES (?, ?)",
        )
        .bind(&artist.id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(followed.len())
}

// ─── sync top tracks / artists, all three time ranges ────────────────────────

const TIME_RANGES: [&str; 3] = ["short_term", "medium_term", "long_term"];

pub async fn sync_top_tracks(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let urls = TIME_RANGES.map(|range| format!("{BASE}/me/top/tracks?limit=50&time_range={range}"));
    let results = tokio::join!(
        spotify::spotify_get::<SpPage<SpTrack>>(token, &urls[0]),
        spotify::spotify_get::<SpPage<SpTrack>>(token, &urls[1]),
        spotify::spotify_get::<SpPage<SpTrack>>(token, &urls[2]),
    );
    let pages = [results.0?, results.1?, results.2?];

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    let mut deps = DepWriter::default();
    let mut total = 0usize;
    for (range, page) in TIME_RANGES.into_iter().zip(&pages) {
        // swap this ranges list all at once so removed entries actually disappear
        sqlx::query("DELETE FROM top_tracks WHERE time_range = ?")
            .bind(range)
            .execute(&mut *tx)
            .await?;

        for (i, t) in page.items.iter().enumerate() {
            if t.is_local.unwrap_or(false) { continue; }
            deps.track(&mut tx, t).await?;
            sqlx::query(
                "INSERT OR IGNORE INTO top_tracks (time_range, track_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&t.id)
            .bind(i as i64)
            .execute(&mut *tx)
            .await?;
            total += 1;
        }
    }
    tx.commit().await?;
    Ok(total)
}

pub async fn sync_top_artists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let urls = TIME_RANGES.map(|range| format!("{BASE}/me/top/artists?limit=50&time_range={range}"));
    let results = tokio::join!(
        spotify::spotify_get::<SpPage<SpArtist>>(token, &urls[0]),
        spotify::spotify_get::<SpPage<SpArtist>>(token, &urls[1]),
        spotify::spotify_get::<SpPage<SpArtist>>(token, &urls[2]),
    );
    let pages = [results.0?, results.1?, results.2?];

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    let mut total = 0usize;
    for (range, page) in TIME_RANGES.into_iter().zip(&pages) {
        sqlx::query("DELETE FROM top_artists WHERE time_range = ?")
            .bind(range)
            .execute(&mut *tx)
            .await?;

        for (i, a) in page.items.iter().enumerate() {
            upsert_artist(&mut tx, a).await?;
            sqlx::query(
                "INSERT OR IGNORE INTO top_artists (time_range, artist_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&a.id)
            .bind(i as i64)
            .execute(&mut *tx)
            .await?;
            total += 1;
        }
    }
    tx.commit().await?;
    Ok(total)
}

// ─── sync recently played, rolling log we trim down to the last 200 ──────────────────

pub async fn sync_recently_played(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let url = format!("{BASE}/me/player/recently-played?limit=50");
    let page: SpRecentlyPlayedPage = spotify::spotify_get(token, &url).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    let mut deps = DepWriter::default();
    let mut n = 0usize;
    for item in &page.items {
        if item.track.is_local.unwrap_or(false) { continue; }
        deps.track(&mut tx, &item.track).await?;
        sqlx::query("INSERT OR IGNORE INTO recently_played (track_id, played_at) VALUES (?, ?)")
            .bind(&item.track.id)
            .bind(parse_iso8601(&item.played_at))
            .execute(&mut *tx)
            .await?;
        n += 1;
    }

    // dont let the log grow forever
    let _ = sqlx::query(
        "DELETE FROM recently_played
         WHERE played_at NOT IN (SELECT played_at FROM recently_played ORDER BY played_at DESC LIMIT 200)",
    )
    .execute(&mut *tx)
    .await;
    tx.commit().await?;

    Ok(n)
}

// ─── known-artist signal, shared by recs + personalized new releases ─────────

use std::sync::RwLock;
use std::time::{Duration, Instant};

static KNOWN_ARTISTS_CACHE: RwLock<Option<(u64, Instant, Vec<String>)>> = RwLock::new(None);

pub fn clear_known_artists_cache() {
    if let Ok(mut guard) = KNOWN_ARTISTS_CACHE.write() {
        *guard = None;
    }
}

/// the users whole artist universe, strongest signal first so current top artists,
/// then followed artists, then artists from liked songs. we use this so recs
/// and "new releases" stay inside the music the user actually listens to
/// instead of random global catalog stuff in languages they dont speak
pub(crate) async fn gather_known_artists(pool: &SqlitePool, token: &str) -> Vec<String> {
    let current_epoch = crate::auth::auth_epoch();
    // 1. Check in-memory cache (15 min TTL, strictly isolated to the current auth epoch)
    if let Ok(guard) = KNOWN_ARTISTS_CACHE.read() {
        if let Some((epoch, ts, ref list)) = *guard {
            if epoch == current_epoch && ts.elapsed() < Duration::from_secs(15 * 60) && !list.is_empty() {
                return list.clone();
            }
        }
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut out:  Vec<String>     = Vec::new();

    // 2. Query local SQLite tables first (instant <1ms, zero network)
    if let Ok(rows) = sqlx::query_as::<_, (String,)>(
        "SELECT artist_id FROM top_artists ORDER BY CASE time_range WHEN 'short_term' THEN 1 WHEN 'medium_term' THEN 2 ELSE 3 END, position ASC LIMIT 100",
    )
    .fetch_all(pool)
    .await
    {
        for (id,) in rows { if !id.is_empty() && seen.insert(id.clone()) { out.push(id); } }
    }

    if let Ok(rows) = sqlx::query_as::<_, (String,)>("SELECT artist_id FROM followed_artists LIMIT 100")
        .fetch_all(pool)
        .await
    {
        for (id,) in rows { if !id.is_empty() && seen.insert(id.clone()) { out.push(id); } }
    }

    if let Ok(rows) = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT ta.artist_id
         FROM track_artists ta JOIN saved_tracks st ON st.track_id = ta.track_id
         LIMIT 200",
    )
    .fetch_all(pool)
    .await
    {
        for (id,) in rows { if !id.is_empty() && seen.insert(id.clone()) { out.push(id); } }
    }

    if let Ok(rows) = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT ta.artist_id
         FROM track_artists ta JOIN recently_played rp ON rp.track_id = ta.track_id
         LIMIT 50",
    )
    .fetch_all(pool)
    .await
    {
        for (id,) in rows { if !id.is_empty() && seen.insert(id.clone()) { out.push(id); } }
    }

    // 3. Only if local DB has fewer than 10 artists (e.g. brand new install), query Spotify concurrently
    if out.len() < 10 {
        let url_st = format!("{BASE}/me/top/artists?limit=50&time_range=short_term");
        let url_mt = format!("{BASE}/me/top/artists?limit=50&time_range=medium_term");
        let (st_res, mt_res) = tokio::join!(
            spotify::spotify_get::<SpPage<SpArtist>>(token, &url_st),
            spotify::spotify_get::<SpPage<SpArtist>>(token, &url_mt),
        );
        if let Ok(p) = st_res {
            for a in p.items {
                if !a.id.is_empty() && seen.insert(a.id.clone()) { out.push(a.id); }
            }
        }
        if let Ok(p) = mt_res {
            for a in p.items {
                if !a.id.is_empty() && seen.insert(a.id.clone()) { out.push(a.id); }
            }
        }
    }

    if let Ok(mut guard) = KNOWN_ARTISTS_CACHE.write() {
        *guard = Some((current_epoch, Instant::now(), out.clone()));
    }

    out
}

/// "YYYY" / "YYYY-MM" / "YYYY-MM-DD" into a sortable yyyymmdd int, bigger = newer
fn release_sort_key(d: Option<&str>) -> i64 {
    let Some(d) = d else { return 0 };
    let mut it = d.split('-');
    let y:   i64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let m:   i64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let day: i64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    y * 10000 + m * 100 + day
}

// ─── sync new releases, personalized, recent drops from artists u know ─────────

pub async fn sync_new_releases(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    use rand::seq::SliceRandom;

    let mut artists = gather_known_artists(pool, token).await;

    // no listening history yet so just fall back to spotifys global new releases
    if artists.is_empty() {
        let url = format!("{BASE}/browse/new-releases?limit=50");
        let resp: SpNewReleases = spotify::spotify_get(token, &url).await?;
        let list: Vec<&SpAlbumSimple> = resp.albums.items.iter().collect();
        return write_new_releases(pool, &list).await;
    }

    artists.shuffle(&mut rand::thread_rng());
    artists.truncate(30);

    // Keep at most eight artist requests/results resident, consuming in artist order.
    let mut seen: HashSet<String> = HashSet::new();
    let mut albums: Vec<(SpAlbumSimple, i64)> = Vec::new();

    let fetches = artists.into_iter().map(|aid| {
        let url = format!(
            "{BASE}/artists/{aid}/albums?include_groups=album,single&market=from_token&limit=10"
        );
        async move {
            spotify::spotify_get::<SpPage<SpAlbumSimple>>(token, &url).await
        }.boxed()
    });
    let mut results = futures_util::stream::iter(fetches).buffered(8);
    while let Some(page_res) = results.next().await {
        if let Ok(page) = page_res {
            for al in page.items {
                if !seen.insert(al.id.clone()) { continue; }
                let key = release_sort_key(al.release_date.as_deref());
                albums.push((al, key));
            }
        }
        if albums.len() > 250 { break; }
    }

    albums.sort_by(|a, b| b.1.cmp(&a.1));
    albums.truncate(50);

    let list: Vec<&SpAlbumSimple> = albums.iter().map(|(al, _)| al).collect();
    write_new_releases(pool, &list).await
}

async fn write_new_releases(pool: &SqlitePool, list: &[&SpAlbumSimple]) -> Result<usize, AppError> {
    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM new_releases").execute(&mut *tx).await?;
    for (i, al) in list.iter().enumerate() {
        upsert_album_simple(&mut tx, al).await?;
        sqlx::query("INSERT OR IGNORE INTO new_releases (album_id, position) VALUES (?, ?)")
            .bind(&al.id).bind(i as i64).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(list.len())
}

// ─── sync_all ─────────────────────────────────────────────────────────────────

// One sync at a time. Sign-in kicks one off while the background loop may be
// starting its own; the second caller waits and takes the first one's result
// instead of pulling the whole library again.
static SYNC_RUN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static LAST_SYNC: std::sync::Mutex<Option<(u64, Instant, SyncResult)>> = std::sync::Mutex::new(None);

/// Pull the whole library. `on_step` fires as each part lands in the db
/// (`"liked"`, `"playlists"`, ...) so the UI can fill in piece by piece
/// instead of waiting on the slowest endpoint.
pub async fn sync_all(
    pool:    &SqlitePool,
    token:   &str,
    on_step: &(dyn Fn(&'static str) + Sync),
) -> Result<SyncResult, AppError> {
    let epoch = crate::auth::auth_epoch();
    let asked = Instant::now();
    let _run = SYNC_RUN.lock().await;
    if let Ok(last) = LAST_SYNC.lock() {
        if let Some((e, finished, result)) = last.as_ref() {
            if *e == epoch && *finished > asked {
                return Ok(result.clone());
            }
        }
    }

    let result = spotify::background(sync_steps(pool, token, epoch, on_step)).await?;
    if let Ok(mut last) = LAST_SYNC.lock() {
        *last = Some((epoch, Instant::now(), result.clone()));
    }
    Ok(result)
}

async fn sync_steps(
    pool:    &SqlitePool,
    token:   &str,
    epoch:   u64,
    on_step: &(dyn Fn(&'static str) + Sync),
) -> Result<SyncResult, AppError> {
    // The steps fetch concurrently and take turns writing (SYNC_WRITE): the
    // network is the slow part, and sqlite only has one writer anyway. Each
    // step is fault tolerant, one failing endpoint just logs and returns 0
    // instead of nuking the whole sync (and the "last synced" timestamp below)
    //
    // Each step is also gated on the auth epoch. A sync is long and chatty, so
    // a sign-out (or an account switch) lands in the middle of one often enough
    // to matter - and every step that ran on afterwards wrote the *old*
    // account's library straight back into the tables logout had just emptied.
    async fn step(
        label:   &'static str,
        epoch:   u64,
        on_step: &(dyn Fn(&'static str) + Sync),
        fut:     impl std::future::Future<Output = Result<usize, AppError>>,
    ) -> usize {
        if !crate::auth::epoch_is_current(epoch) {
            return 0;
        }
        match fut.await {
            Ok(n) => { on_step(label); n }
            Err(e) => { eprintln!("[library] {label} sync failed: {e}"); 0 }
        }
    }

    // new releases are picked from the artists the other steps just stored, so
    // they go after followed + top artists
    let (liked, playlists, albums_n, top_tracks, recent, (artists, top_artists, new_rel)) = tokio::join!(
        step("liked",           epoch, on_step, sync_liked_songs(pool, token)),
        step("playlists",       epoch, on_step, sync_playlists(pool, token)),
        step("albums",          epoch, on_step, sync_saved_albums(pool, token)),
        step("top_tracks",      epoch, on_step, sync_top_tracks(pool, token)),
        step("recently_played", epoch, on_step, sync_recently_played(pool, token)),
        async {
            let (artists, top_artists) = tokio::join!(
                step("artists",     epoch, on_step, sync_followed_artists(pool, token)),
                step("top_artists", epoch, on_step, sync_top_artists(pool, token)),
            );
            let new_rel = step("new_releases", epoch, on_step, sync_new_releases(pool, token)).await;
            (artists, top_artists, new_rel)
        },
    );

    if !crate::auth::epoch_is_current(epoch) {
        eprintln!("[library] sync abandoned: the signed-in account changed while it ran");
        return Err(AppError::Auth("Signed out while syncing".into()));
    }

    let result = SyncResult {
        liked_count:       liked,
        playlist_count:    playlists,
        artist_count:      artists,
        album_count:       albums_n,
        top_track_count:   top_tracks,
        top_artist_count:  top_artists,
        recent_count:      recent,
        new_release_count: new_rel,
    };

    // stash the sync timestamp in settings
    let now_num = now_ms();
    let now = now_num.to_string();
    let _ = sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES ('library_last_synced', ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(&now)
    .bind(now_num)
    .execute(pool)
    .await;

    Ok(result)
}

/// `on_step` for a sync whose progress should reach the UI: each finished
/// step emits `library:updated` so the matching views refetch from sqlite.
pub fn emit_steps(app: &tauri::AppHandle) -> impl Fn(&'static str) + Sync + '_ {
    use tauri::Emitter;
    move |step| { let _ = app.emit("library:updated", step); }
}

// ─── background sync loop ─────────────────────────────────────────────────────

pub async fn library_sync_loop(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    // wait a bit first so the auth state has time to load
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    loop {
        let s    = app.state::<crate::state::AppState>();
        let pool = s.db.clone();
        let auth = s.auth.clone();
        let gate = s.sync_gate.clone();
        drop(s);

        if let Ok(token) = crate::auth::get_valid_token(&pool, &auth).await {
            let _syncing = gate.read().await;
            match sync_all(&pool, &token, &emit_steps(&app)).await {
                Ok(result) => {
                    let _ = app.emit("library:synced", result);
                }
                Err(e) => eprintln!("[library] sync failed: {e}"),
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
    }
}
