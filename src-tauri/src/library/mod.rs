use crate::{
    db::repos::{albums, artists, tracks},
    errors::AppError,
    spotify::{self, types::*},
};
use serde::Serialize;
use sqlx::SqlitePool;

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

async fn upsert_artist(pool: &SqlitePool, a: &SpArtist) -> Result<(), AppError> {
    artists::upsert(pool, &artists::Artist {
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

pub(crate) async fn upsert_track_with_deps(pool: &SqlitePool, t: &SpTrack) -> Result<(), AppError> {
    // upsert the album if theres one
    if let Some(al) = &t.album {
        albums::upsert(pool, &albums::Album {
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

    // upsert every artist
    for a in &t.artists {
        let _ = artists::upsert(pool, &artists::Artist {
            id:         a.id.clone(),
            name:       a.name.clone(),
            image_url:  None,
            genres:     None,
            popularity: None,
            updated_at: 0,
        })
        .await;
    }

    // upsert the track itself
    tracks::upsert(pool, &tracks::Track {
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
    })
    .await?;

    // upsert the track_artists join rows
    for (i, a) in t.artists.iter().enumerate() {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO track_artists (track_id, artist_id, position) VALUES (?, ?, ?)",
        )
        .bind(&t.id)
        .bind(&a.id)
        .bind(i as i64)
        .execute(pool)
        .await;
    }

    Ok(())
}

async fn upsert_album_simple(pool: &SqlitePool, al: &SpAlbumSimple) -> Result<(), AppError> {
    albums::upsert(pool, &albums::Album {
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

    for (i, a) in al.artists.as_ref().map(|v| v.as_slice()).unwrap_or(&[]).iter().enumerate() {
        let _ = sqlx::query("INSERT OR IGNORE INTO artists (id, name, updated_at) VALUES (?, ?, 0)")
            .bind(&a.id)
            .bind(&a.name)
            .execute(pool)
            .await;
        let _ = albums::add_artist(pool, &al.id, &a.id, i as i64).await;
    }
    Ok(())
}

// ─── sync liked songs ─────────────────────────────────────────────────────────

pub async fn sync_liked_songs(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut liked: Vec<(String, i64)> = Vec::new();
    let mut offset = 0i64;
    let limit      = 50i64;

    loop {
        let url = format!("{BASE}/me/tracks?limit={limit}&offset={offset}");
        let page: SpSavedTrackPage = spotify::spotify_get(token, &url).await?;

        for item in &page.items {
            if let Some(track) = &item.track {
                if track.is_local.unwrap_or(false) { continue; }
                upsert_track_with_deps(pool, track).await?;
                liked.push((track.id.clone(), parse_iso8601(&item.added_at)));
            }
        }

        if page.next.is_none() || (page.items.len() as i64) < limit { break; }
        offset += limit;
    }

    let count = liked.len();

    // wipe and replace the whole thing in a txn so unlikes actually show up
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_tracks").execute(&mut *tx).await?;
    for (track_id, added_at) in &liked {
        sqlx::query(
            "INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)",
        )
        .bind(track_id)
        .bind(added_at)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync playlists ───────────────────────────────────────────────────────────

pub async fn sync_playlists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut synced_ids: Vec<String> = Vec::new();
    let mut offset = 0i64;
    let limit      = 50i64;

    loop {
        let url = format!("{BASE}/me/playlists?limit={limit}&offset={offset}");
        let page: SpUserPlaylistPage = spotify::spotify_get(token, &url).await?;

        for item in &page.items {
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
            .bind(now_ms())
            .execute(pool)
            .await?;

            synced_ids.push(item.id.clone());
        }

        if page.next.is_none() || (page.items.len() as i64) < limit { break; }
        offset += limit;
    }

    Ok(synced_ids.len())
}

// ─── sync saved albums ────────────────────────────────────────────────────────

pub async fn sync_saved_albums(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut saved: Vec<(String, i64)> = Vec::new();
    let mut offset = 0i64;
    let limit      = 50i64;

    loop {
        let url = format!("{BASE}/me/albums?limit={limit}&offset={offset}");
        let page: SpSavedAlbumPage = spotify::spotify_get(token, &url).await?;

        let page_len = page.items.len() as i64;
        for item in page.items.iter().flatten() {
            let al = &item.album;
            albums::upsert(pool, &albums::Album {
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

            for (i, a) in al.artists.as_ref().map(|v| v.as_slice()).unwrap_or(&[]).iter().enumerate() {
                let _ = sqlx::query("INSERT OR IGNORE INTO artists (id, name, updated_at) VALUES (?, ?, 0)")
                    .bind(&a.id)
                    .bind(&a.name)
                    .execute(pool)
                    .await;
                let _ = albums::add_artist(pool, &al.id, &a.id, i as i64).await;
            }

            saved.push((al.id.clone(), parse_iso8601(&item.added_at)));
        }

        if page.next.is_none() || page_len < limit { break; }
        offset += limit;
    }

    let count = saved.len();

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_albums").execute(&mut *tx).await?;
    for (album_id, added_at) in &saved {
        sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
            .bind(album_id)
            .bind(added_at)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync followed artists ────────────────────────────────────────────────────

pub async fn sync_followed_artists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut followed: Vec<String> = Vec::new();
    let mut cursor: Option<String> = None;
    let limit = 50;

    loop {
        let url = match &cursor {
            Some(after) => format!("{BASE}/me/following?type=artist&limit={limit}&after={after}"),
            None        => format!("{BASE}/me/following?type=artist&limit={limit}"),
        };

        let resp: SpFollowedArtistsResponse = spotify::spotify_get(token, &url).await?;
        let page = resp.artists;

        for artist in &page.items {
            upsert_artist(pool, artist).await?;
            followed.push(artist.id.clone());
        }

        cursor = page.cursors.as_ref().and_then(|c| c.after.clone());
        if page.next.is_none() || page.items.len() < limit { break; }
    }

    let count = followed.len();

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM followed_artists").execute(&mut *tx).await?;
    let now = now_ms();
    for artist_id in &followed {
        sqlx::query(
            "INSERT OR IGNORE INTO followed_artists (artist_id, followed_at) VALUES (?, ?)",
        )
        .bind(artist_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(count)
}

// ─── sync top tracks / artists, all three time ranges ────────────────────────

const TIME_RANGES: [&str; 3] = ["short_term", "medium_term", "long_term"];

pub async fn sync_top_tracks(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut total = 0usize;
    for range in TIME_RANGES {
        let url  = format!("{BASE}/me/top/tracks?limit=50&time_range={range}");
        let page: SpPage<SpTrack> = spotify::spotify_get(token, &url).await?;

        // swap this ranges list all at once so removed entries actually disappear
        sqlx::query("DELETE FROM top_tracks WHERE time_range = ?")
            .bind(range)
            .execute(pool)
            .await?;

        for (i, t) in page.items.iter().enumerate() {
            if t.is_local.unwrap_or(false) { continue; }
            upsert_track_with_deps(pool, t).await?;
            sqlx::query(
                "INSERT OR IGNORE INTO top_tracks (time_range, track_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&t.id)
            .bind(i as i64)
            .execute(pool)
            .await?;
            total += 1;
        }
    }
    Ok(total)
}

pub async fn sync_top_artists(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let mut total = 0usize;
    for range in TIME_RANGES {
        let url  = format!("{BASE}/me/top/artists?limit=50&time_range={range}");
        let page: SpPage<SpArtist> = spotify::spotify_get(token, &url).await?;

        sqlx::query("DELETE FROM top_artists WHERE time_range = ?")
            .bind(range)
            .execute(pool)
            .await?;

        for (i, a) in page.items.iter().enumerate() {
            upsert_artist(pool, a).await?;
            sqlx::query(
                "INSERT OR IGNORE INTO top_artists (time_range, artist_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&a.id)
            .bind(i as i64)
            .execute(pool)
            .await?;
            total += 1;
        }
    }
    Ok(total)
}

// ─── sync recently played, rolling log we trim down to the last 200 ──────────────────

pub async fn sync_recently_played(pool: &SqlitePool, token: &str) -> Result<usize, AppError> {
    let url = format!("{BASE}/me/player/recently-played?limit=50");
    let page: SpRecentlyPlayedPage = spotify::spotify_get(token, &url).await?;

    let mut n = 0usize;
    for item in &page.items {
        if item.track.is_local.unwrap_or(false) { continue; }
        upsert_track_with_deps(pool, &item.track).await?;
        sqlx::query("INSERT OR IGNORE INTO recently_played (track_id, played_at) VALUES (?, ?)")
            .bind(&item.track.id)
            .bind(parse_iso8601(&item.played_at))
            .execute(pool)
            .await?;
        n += 1;
    }

    // dont let the log grow forever
    let _ = sqlx::query(
        "DELETE FROM recently_played
         WHERE played_at NOT IN (SELECT played_at FROM recently_played ORDER BY played_at DESC LIMIT 200)",
    )
    .execute(pool)
    .await;

    Ok(n)
}

// ─── known-artist signal, shared by recs + personalized new releases ─────────

/// the users whole artist universe, strongest signal first so current top artists,
/// then followed artists, then artists from liked songs. we use this so recs
/// and "new releases" stay inside the music the user actually listens to
/// instead of random global catalog stuff in languages they dont speak
pub(crate) async fn gather_known_artists(pool: &SqlitePool, token: &str) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out:  Vec<String>     = Vec::new();

    for range in ["short_term", "medium_term"] {
        if let Ok(p) = spotify::spotify_get::<SpPage<SpArtist>>(
            token,
            &format!("{BASE}/me/top/artists?limit=50&time_range={range}"),
        )
        .await
        {
            for a in p.items {
                if !a.id.is_empty() && seen.insert(a.id.clone()) { out.push(a.id); }
            }
        }
    }

    if let Ok(rows) = sqlx::query_as::<_, (String,)>("SELECT artist_id FROM followed_artists")
        .fetch_all(pool)
        .await
    {
        for (id,) in rows { if seen.insert(id.clone()) { out.push(id); } }
    }

    if let Ok(rows) = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT ta.artist_id
         FROM track_artists ta JOIN saved_tracks st ON st.track_id = ta.track_id
         LIMIT 200",
    )
    .fetch_all(pool)
    .await
    {
        for (id,) in rows { if seen.insert(id.clone()) { out.push(id); } }
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
    use std::collections::HashSet;

    let mut artists = gather_known_artists(pool, token).await;

    // no listening history yet so just fall back to spotifys global new releases
    if artists.is_empty() {
        let url = format!("{BASE}/browse/new-releases?limit=50");
        let resp: SpNewReleases = spotify::spotify_get(token, &url).await?;
        sqlx::query("DELETE FROM new_releases").execute(pool).await?;
        let mut n = 0usize;
        for (i, al) in resp.albums.items.iter().enumerate() {
            upsert_album_simple(pool, al).await?;
            sqlx::query("INSERT OR IGNORE INTO new_releases (album_id, position) VALUES (?, ?)")
                .bind(&al.id).bind(i as i64).execute(pool).await?;
            n += 1;
        }
        return Ok(n);
    }

    artists.shuffle(&mut rand::thread_rng());
    artists.truncate(30);

    // grab each artists latest albums/singles, kill dupes, sort newest first
    let mut seen: HashSet<String> = HashSet::new();
    let mut albums: Vec<(SpAlbumSimple, i64)> = Vec::new();
    for aid in &artists {
        let url = format!(
            "{BASE}/artists/{aid}/albums?include_groups=album,single&market=from_token&limit=10"
        );
        if let Ok(page) = spotify::spotify_get::<SpPage<SpAlbumSimple>>(token, &url).await {
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

    sqlx::query("DELETE FROM new_releases").execute(pool).await?;
    let mut n = 0usize;
    for (i, (al, _)) in albums.iter().enumerate() {
        upsert_album_simple(pool, al).await?;
        sqlx::query("INSERT OR IGNORE INTO new_releases (album_id, position) VALUES (?, ?)")
            .bind(&al.id).bind(i as i64).execute(pool).await?;
        n += 1;
    }
    Ok(n)
}

// ─── sync_all ─────────────────────────────────────────────────────────────────

pub async fn sync_all(pool: &SqlitePool, token: &str) -> Result<SyncResult, AppError> {
    // run these one after another NOT with tokio::join!. sqlite even in wal only
    // lets one writer go at a time so concurrent sync txns fight over the write lock
    // and thats what was killing the saved albums sync. each step is also
    // made fault tolerant, one failing endpoint just logs and returns 0 instead of
    // nuking the whole sync (and the "last synced" timestamp below)
    async fn step(
        label: &str,
        fut: impl std::future::Future<Output = Result<usize, AppError>>,
    ) -> usize {
        match fut.await {
            Ok(n) => n,
            Err(e) => { eprintln!("[library] {label} sync failed: {e}"); 0 }
        }
    }

    let liked       = step("liked",           sync_liked_songs(pool, token)).await;
    let playlists   = step("playlists",       sync_playlists(pool, token)).await;
    let artists     = step("artists",         sync_followed_artists(pool, token)).await;
    let albums_n    = step("albums",          sync_saved_albums(pool, token)).await;
    let top_tracks  = step("top_tracks",      sync_top_tracks(pool, token)).await;
    let top_artists = step("top_artists",     sync_top_artists(pool, token)).await;
    let recent      = step("recently_played", sync_recently_played(pool, token)).await;
    let new_rel     = step("new_releases",    sync_new_releases(pool, token)).await;

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
    let now = now_ms().to_string();
    let _ = sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('library_last_synced', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&now)
    .execute(pool)
    .await;

    Ok(result)
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
        drop(s);

        if let Ok(token) = crate::auth::get_valid_token(&pool, &auth).await {
            match sync_all(&pool, &token).await {
                Ok(result) => {
                    let _ = app.emit("library:synced", result);
                }
                Err(e) => eprintln!("[library] sync failed: {e}"),
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
    }
}
