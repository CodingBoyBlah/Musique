//! Synchronizes the Apple Music library into musique's shared SQLite schema.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sqlx::{SqliteConnection, SqlitePool};

use crate::apple::api;
use crate::apple::client::{self, AppleClient};
use crate::apple::wire::{self, ParsedAlbum, ParsedArtist, ParsedPlaylist, ParsedTrack};
use crate::db::repos::{albums, artists, tracks};
use crate::errors::AppError;
use crate::library::{parse_iso8601, SyncResult};
use crate::spotify::types::{AlbumItem, ArtistItem, TrackDetail, TrackItem};
use crate::commands::library::PlaylistSummary;

const TIME_RANGES: [&str; 3] = ["short_term", "medium_term", "long_term"];

static SYNC_WRITE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static SYNC_RUN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static LAST_SYNC: std::sync::Mutex<Option<(u64, u64, Instant, SyncResult)>> =
    std::sync::Mutex::new(None);
static APPLE_SESSION_EPOCH: AtomicU64 = AtomicU64::new(0);

pub(crate) fn bump_session_epoch() -> u64 {
    APPLE_SESSION_EPOCH.fetch_add(1, Ordering::SeqCst) + 1
}

fn current_apple_epoch() -> u64 {
    APPLE_SESSION_EPOCH.load(Ordering::SeqCst)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Pulls the entire Apple Music library into SQLite, emitting Spotify-compatible step labels
/// (`"liked"`, `"playlists"`, `"albums"`, `"artists"`, `"top_tracks"`, `"top_artists"`,
/// `"recently_played"`, `"new_releases"`).
pub async fn sync_all(
    pool: &SqlitePool,
    on_step: &(dyn Fn(&'static str) + Sync),
) -> Result<SyncResult, AppError> {
    let client = client::current().await?;
    sync_all_with(pool, &client, on_step).await
}

pub(crate) async fn sync_all_with(
    pool: &SqlitePool,
    client: &AppleClient,
    on_step: &(dyn Fn(&'static str) + Sync),
) -> Result<SyncResult, AppError> {
    let sp_epoch = crate::auth::auth_epoch();
    let ap_epoch = current_apple_epoch();
    let asked = Instant::now();

    let _run = SYNC_RUN.lock().await;
    if let Ok(last) = LAST_SYNC.lock() {
        if let Some((se, ae, finished, res)) = last.as_ref() {
            if *se == sp_epoch
                && *ae == ap_epoch
                && *finished > asked
                && finished.elapsed() < Duration::from_secs(5)
            {
                return Ok(res.clone());
            }
        }
    }

    let result = Box::pin(sync_steps(pool, client, sp_epoch, ap_epoch, on_step)).await?;
    if let Ok(mut last) = LAST_SYNC.lock() {
        *last = Some((sp_epoch, ap_epoch, Instant::now(), result.clone()));
    }
    Ok(result)
}

fn session_valid(sp_epoch: u64, ap_epoch: u64) -> bool {
    crate::auth::epoch_is_current(sp_epoch) && current_apple_epoch() == ap_epoch
}

async fn sync_steps(
    pool: &SqlitePool,
    client: &AppleClient,
    sp_epoch: u64,
    ap_epoch: u64,
    on_step: &(dyn Fn(&'static str) + Sync),
) -> Result<SyncResult, AppError> {
    async fn run_step(
        label: &'static str,
        sp_epoch: u64,
        ap_epoch: u64,
        on_step: &(dyn Fn(&'static str) + Sync),
        fut: impl std::future::Future<Output = Result<usize, AppError>>,
    ) -> Result<usize, AppError> {
        if !session_valid(sp_epoch, ap_epoch) {
            return Err(AppError::Auth("Signed out while syncing Apple Music".into()));
        }
        match fut.await {
            Ok(n) => {
                on_step(label);
                Ok(n)
            }
            Err(e) => {
                eprintln!("[apple-library] {label} sync failed: {e}");
                Err(e)
            }
        }
    }

    let (liked, playlists, albums_n, artists_n, top_tracks, top_artists, recent, new_rel) =
        tokio::join!(
            run_step(
                "liked",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_liked_songs(pool, client)),
            ),
            run_step(
                "playlists",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_playlists(pool, client)),
            ),
            run_step(
                "albums",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_saved_albums(pool, client)),
            ),
            run_step(
                "artists",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_followed_artists(pool, client)),
            ),
            run_step(
                "top_tracks",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_top_tracks(pool, client)),
            ),
            run_step(
                "top_artists",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_top_artists(pool, client)),
            ),
            run_step(
                "recently_played",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_recently_played(pool, client)),
            ),
            run_step(
                "new_releases",
                sp_epoch,
                ap_epoch,
                on_step,
                Box::pin(sync_new_releases(pool, client)),
            ),
        );

    if !session_valid(sp_epoch, ap_epoch) {
        return Err(AppError::Auth("Signed out while syncing Apple Music".into()));
    }

    let result = SyncResult {
        liked_count: liked?,
        playlist_count: playlists?,
        artist_count: artists_n?,
        album_count: albums_n?,
        top_track_count: top_tracks?,
        top_artist_count: top_artists?,
        recent_count: recent?,
        new_release_count: new_rel?,
    };

    let now_num = now_ms();
    let _ = sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES ('library_last_synced', ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(now_num.to_string())
    .bind(now_num)
    .execute(pool)
    .await;

    Ok(result)
}

#[derive(Default)]
struct DepWriter {
    albums: HashSet<String>,
    artists: HashSet<String>,
}

impl DepWriter {
    async fn artist(
        &mut self,
        conn: &mut SqliteConnection,
        artist: &ArtistItem,
        genres: Option<String>,
    ) -> Result<(), AppError> {
        if artist.id.is_empty() || !self.artists.insert(artist.id.clone()) {
            return Ok(());
        }
        artists::upsert(
            &mut *conn,
            &artists::Artist {
                id: artist.id.clone(),
                name: artist.name.clone(),
                image_url: artist.image_url.clone(),
                genres,
                popularity: artist.popularity,
                updated_at: 0,
            },
        )
        .await
    }

    async fn album(
        &mut self,
        conn: &mut SqliteConnection,
        album: &AlbumItem,
        total_tracks: i64,
        genres: Option<String>,
    ) -> Result<(), AppError> {
        if album.id.is_empty() {
            return Ok(());
        }
        if self.albums.insert(album.id.clone()) {
            albums::upsert(
                &mut *conn,
                &albums::Album {
                    id: album.id.clone(),
                    name: album.name.clone(),
                    album_type: album.album_type.clone(),
                    image_url: album.image_url.clone(),
                    release_date: album.release_date.clone(),
                    total_tracks,
                    genres,
                    popularity: album.popularity,
                    updated_at: 0,
                },
            )
            .await?;
            for (pos, a) in album.artists.iter().enumerate() {
                self.artist(&mut *conn, a, None).await?;
                if !a.id.is_empty() {
                    let _ = albums::add_artist(&mut *conn, &album.id, &a.id, pos as i64).await;
                }
            }
        }
        Ok(())
    }

    async fn track(
        &mut self,
        conn: &mut SqliteConnection,
        track: &TrackItem,
        isrc: Option<&str>,
        preview_url: Option<&str>,
        track_number: i64,
        disc_number: i64,
    ) -> Result<(), AppError> {
        if let Some(ref al) = track.album {
            self.album(&mut *conn, al, 0, None).await?;
        }
        for a in &track.artists {
            self.artist(&mut *conn, a, None).await?;
        }

        tracks::upsert(
            &mut *conn,
            &tracks::Track {
                id: track.id.clone(),
                name: track.name.clone(),
                album_id: track
                    .album
                    .as_ref()
                    .map(|a| a.id.clone())
                    .filter(|id| !id.is_empty()),
                duration_ms: track.duration_ms,
                track_number,
                disc_number,
                explicit: track.explicit,
                popularity: track.popularity,
                preview_url: preview_url.map(str::to_owned),
                is_local: false,
                updated_at: 0,
                isrc: isrc.map(str::to_owned),
            },
        )
        .await?;

        for (pos, a) in track.artists.iter().enumerate() {
            if a.id.is_empty() {
                continue;
            }
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO track_artists (track_id, artist_id, position) VALUES (?, ?, ?)",
            )
            .bind(&track.id)
            .bind(&a.id)
            .bind(pos as i64)
            .execute(&mut *conn)
            .await;
        }
        Ok(())
    }

    async fn parsed_track(
        &mut self,
        conn: &mut SqliteConnection,
        t: &ParsedTrack,
    ) -> Result<(), AppError> {
        self.track(
            conn,
            &t.item,
            t.isrc.as_deref(),
            t.preview_url.as_deref(),
            t.track_number,
            t.disc_number,
        )
        .await
    }
}

async fn sync_liked_songs(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let songs = api::liked_songs_parsed_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_tracks")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    let base_ms = now_ms();
    let mut count = 0usize;

    for (idx, t) in songs.iter().enumerate() {
        deps.parsed_track(&mut tx, t).await?;
        let added_at = t
            .date_added
            .as_deref()
            .map(parse_iso8601)
            .unwrap_or_else(|| base_ms - idx as i64);
        sqlx::query("INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)")
            .bind(&t.item.id)
            .bind(added_at)
            .execute(&mut *tx)
            .await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

async fn sync_playlists(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let pls = api::library_playlists_parsed_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    let keep_ids: HashSet<&str> = pls.iter().map(|p| p.id.as_str()).collect();
    let existing: Vec<(String,)> = sqlx::query_as("SELECT id FROM playlists")
        .fetch_all(&mut *tx)
        .await?;
    for (existing_id,) in existing {
        if !keep_ids.contains(existing_id.as_str()) {
            sqlx::query("DELETE FROM playlists WHERE id = ?")
                .bind(&existing_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let base_ms = now_ms();
    for (idx, pl) in pls.iter().enumerate() {
        upsert_playlist_row(&mut tx, pl, base_ms - idx as i64).await?;
    }

    tx.commit().await?;
    Ok(pls.len())
}

async fn upsert_playlist_row(
    conn: &mut SqliteConnection,
    pl: &ParsedPlaylist,
    updated_at: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO playlists
             (id, name, description, owner_id, image_url, total_tracks,
              is_public, is_local, snapshot_id, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name         = excluded.name,
             description  = excluded.description,
             owner_id     = excluded.owner_id,
             image_url    = excluded.image_url,
             total_tracks = CASE WHEN excluded.total_tracks > 0
                                 THEN excluded.total_tracks ELSE playlists.total_tracks END,
             is_public    = excluded.is_public,
             snapshot_id  = excluded.snapshot_id,
             updated_at   = excluded.updated_at",
    )
    .bind(&pl.id)
    .bind(&pl.name)
    .bind(&pl.description)
    .bind(&pl.owner_name)
    .bind(&pl.image_url)
    .bind(pl.total_tracks)
    .bind(pl.public.unwrap_or(false))
    .bind(&pl.snapshot_id)
    .bind(updated_at)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn sync_saved_albums(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let albs = api::library_albums_parsed_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM saved_albums")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    let base_ms = now_ms();
    for (idx, al) in albs.iter().enumerate() {
        upsert_parsed_album(&mut tx, &mut deps, al).await?;
        let added_at = al
            .date_added
            .as_deref()
            .map(parse_iso8601)
            .unwrap_or_else(|| base_ms - idx as i64);
        sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
            .bind(&al.item.id)
            .bind(added_at)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(albs.len())
}

async fn upsert_parsed_album(
    conn: &mut SqliteConnection,
    deps: &mut DepWriter,
    al: &ParsedAlbum,
) -> Result<(), AppError> {
    let genres = (!al.genres.is_empty())
        .then(|| serde_json::to_string(&al.genres).unwrap_or_default());
    deps.album(conn, &al.item, al.total_tracks, genres).await
}

async fn sync_followed_artists(
    pool: &SqlitePool,
    client: &AppleClient,
) -> Result<usize, AppError> {
    let arts = api::library_artists_parsed_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM followed_artists")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    let base_ms = now_ms();
    for (idx, a) in arts.iter().enumerate() {
        upsert_parsed_artist(&mut tx, &mut deps, a).await?;
        let followed_at = a
            .date_added
            .as_deref()
            .map(parse_iso8601)
            .unwrap_or_else(|| base_ms - idx as i64);
        sqlx::query(
            "INSERT OR IGNORE INTO followed_artists (artist_id, followed_at) VALUES (?, ?)",
        )
        .bind(&a.item.id)
        .bind(followed_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(arts.len())
}

async fn upsert_parsed_artist(
    conn: &mut SqliteConnection,
    deps: &mut DepWriter,
    a: &ParsedArtist,
) -> Result<(), AppError> {
    let genres = (!a.genres.is_empty())
        .then(|| serde_json::to_string(&a.genres).unwrap_or_default());
    deps.artist(conn, &a.item, genres).await
}

async fn sync_top_tracks(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let top = api::heavy_rotation_tracks_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM top_tracks")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    for t in &top {
        deps.track(&mut tx, t, None, None, 0, 1).await?;
    }
    for range in TIME_RANGES {
        for (pos, t) in top.iter().enumerate() {
            sqlx::query(
                "INSERT OR IGNORE INTO top_tracks (time_range, track_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&t.id)
            .bind(pos as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(top.len())
}

async fn sync_top_artists(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let top = api::heavy_rotation_artists_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM top_artists")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    for a in &top {
        deps.artist(&mut tx, a, None).await?;
    }
    for range in TIME_RANGES {
        for (pos, a) in top.iter().enumerate() {
            sqlx::query(
                "INSERT OR IGNORE INTO top_artists (time_range, artist_id, position) VALUES (?, ?, ?)",
            )
            .bind(range)
            .bind(&a.id)
            .bind(pos as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(top.len())
}

async fn sync_recently_played(
    pool: &SqlitePool,
    client: &AppleClient,
) -> Result<usize, AppError> {
    let recent = api::recently_played_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM recently_played")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    let base_ms = now_ms();
    for (idx, t) in recent.iter().enumerate() {
        deps.track(&mut tx, t, None, None, 0, 1).await?;
        let played_at = base_ms - (idx as i64) * 1000;
        sqlx::query("INSERT OR IGNORE INTO recently_played (track_id, played_at) VALUES (?, ?)")
            .bind(&t.id)
            .bind(played_at)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(recent.len())
}

async fn sync_new_releases(pool: &SqlitePool, client: &AppleClient) -> Result<usize, AppError> {
    let releases = api::new_releases_with(client).await?;

    let _write = SYNC_WRITE.lock().await;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM new_releases")
        .execute(&mut *tx)
        .await?;

    let mut deps = DepWriter::default();
    for (pos, al) in releases.iter().enumerate() {
        deps.album(&mut tx, al, 0, None).await?;
        sqlx::query("INSERT OR IGNORE INTO new_releases (album_id, position) VALUES (?, ?)")
            .bind(&al.id)
            .bind(pos as i64)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(releases.len())
}

// ─── write-path local SQLite mirror helpers ──────────────────────────────────

/// Mirrors saving or unsaving a track in the local `saved_tracks` table (upserting catalog rows first).
pub async fn mirror_saved_track(
    pool: &SqlitePool,
    track: &TrackItem,
    saved: bool,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    if saved {
        let mut deps = DepWriter::default();
        deps.track(&mut tx, track, None, None, 0, 1).await?;
        sqlx::query("INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)")
            .bind(&track.id)
            .bind(now_ms())
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("DELETE FROM saved_tracks WHERE track_id = ?")
            .bind(&track.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Mirrors saving or unsaving an album in `saved_albums`.
pub async fn mirror_saved_album(
    pool: &SqlitePool,
    album: &AlbumItem,
    saved: bool,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    if saved {
        let mut deps = DepWriter::default();
        deps.album(&mut tx, album, 0, None).await?;
        sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
            .bind(&album.id)
            .bind(now_ms())
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("DELETE FROM saved_albums WHERE album_id = ?")
            .bind(&album.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Mirrors following or unfollowing an artist in `followed_artists`.
pub async fn mirror_followed_artist(
    pool: &SqlitePool,
    artist: &ArtistItem,
    followed: bool,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    if followed {
        let mut deps = DepWriter::default();
        deps.artist(&mut tx, artist, None).await?;
        sqlx::query(
            "INSERT OR IGNORE INTO followed_artists (artist_id, followed_at) VALUES (?, ?)",
        )
        .bind(&artist.id)
        .bind(now_ms())
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query("DELETE FROM followed_artists WHERE artist_id = ?")
            .bind(&artist.id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Mirrors playlist creation or metadata edits into `playlists`.
pub async fn mirror_playlist_summary(
    pool: &SqlitePool,
    pl: &PlaylistSummary,
    owner_name: Option<&str>,
    is_public: bool,
) -> Result<(), AppError> {
    let parsed = ParsedPlaylist {
        id: pl.id.clone(),
        library_id: wire::is_library_id(&pl.id).then(|| pl.id.clone()),
        name: pl.name.clone(),
        description: pl.description.clone(),
        image_url: pl.image_url.clone(),
        owner_name: owner_name.map(str::to_owned),
        owner_id: owner_name.map(str::to_owned),
        public: Some(is_public),
        collaborative: None,
        can_edit: true,
        total_tracks: pl.total_tracks,
        snapshot_id: pl.snapshot_id.clone(),
        tracks: Vec::new(),
    };
    let mut tx = pool.begin().await?;
    upsert_playlist_row(&mut tx, &parsed, now_ms()).await?;
    tx.commit().await?;
    Ok(())
}

/// Removes a deleted or unfollowed playlist from `playlists` and `playlist_tracks`.
pub async fn mirror_deleted_playlist(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Removes a track from a playlist in `playlist_tracks`.
pub async fn mirror_playlist_track_removed(
    pool: &SqlitePool,
    playlist_id: &str,
    track_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
        .bind(playlist_id)
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Persists a `TrackDetail` and optional `isrc` into `tracks` (plus its album and artists).
pub async fn upsert_track_detail(
    pool: &SqlitePool,
    detail: &TrackDetail,
    isrc: Option<&str>,
) -> Result<(), AppError> {
    let item = TrackItem {
        id: detail.id.clone(),
        name: detail.name.clone(),
        duration_ms: detail.duration_ms,
        explicit: detail.explicit,
        artists: detail.artists.clone(),
        album: detail.album.clone(),
        popularity: detail.popularity,
    };
    let mut tx = pool.begin().await?;
    let mut deps = DepWriter::default();
    deps.track(&mut tx, &item, isrc, detail.preview_url.as_deref(), 0, 1)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use std::sync::{atomic::AtomicUsize, Arc};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("apple")
            .join("testdata")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn jwt() -> String {
        let h = URL_SAFE_NO_PAD.encode(r#"{"alg":"ES256"}"#);
        let p = URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":2000000000,"p":"{}"}}"#, "a".repeat(60)));
        format!("{h}.{p}.sig")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sync_all_populates_sqlite_tables_and_emits_expected_steps() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let songs_json = fixture("library_songs.json");
        let albums_json = fixture("library_albums.json");
        let pls_json = fixture("library_playlists.json");
        let charts_json = fixture("charts.json");
        let cat_song_json = fixture("catalog_song.json");
        let artists_json = r#"{"meta":{"total":1},"data":[{"id":"r.art1","type":"library-artists","attributes":{"name":"New Order"},"relationships":{"catalog":{"data":[{"id":"555","type":"artists","attributes":{"name":"New Order","artwork":{"url":"https://is1.mzstatic.com/a/{w}x{h}bb.{f}"}}}]}}}]}"#.to_owned();
        let hr_json = r#"{"data":[{"id":"617154241","type":"albums","attributes":{"name":"Substance","artistName":"New Order","isCompilation":true},"relationships":{"artists":{"data":[{"id":"555","type":"artists","attributes":{"name":"New Order"}}]}}}]}"#.to_owned();
        let album_detail_json = r#"{"data":[{"id":"617154241","type":"albums","attributes":{"name":"Substance","artistName":"New Order","isCompilation":true,"trackCount":1},"relationships":{"artists":{"data":[{"id":"555","type":"artists","attributes":{"name":"New Order"}}]},"tracks":{"data":[{"id":"1440857781","type":"songs","attributes":{"name":"Blue Monday","artistName":"New Order","durationInMillis":447000}}]}}}]}"#.to_owned();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]);
                let path = req
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("");

                let (status, body) = if path.starts_with("/v1/me/storefront") {
                    ("200 OK", r#"{"data":[{"id":"us"}]}"#.to_owned())
                } else if path.starts_with("/v1/me/library/songs") {
                    ("200 OK", songs_json.clone())
                } else if path.starts_with("/v1/me/library/playlists") {
                    ("200 OK", pls_json.clone())
                } else if path.starts_with("/v1/me/library/albums") {
                    ("200 OK", albums_json.clone())
                } else if path.starts_with("/v1/me/library/artists") {
                    ("200 OK", artists_json.clone())
                } else if path.starts_with("/v1/me/favorites") {
                    ("404 Not Found", "{}".to_owned())
                } else if path.starts_with("/v1/me/history/heavy-rotation") {
                    ("200 OK", hr_json.clone())
                } else if path.starts_with("/v1/catalog/us/albums/617154241") {
                    ("200 OK", album_detail_json.clone())
                } else if path.starts_with("/v1/me/recent/played/tracks") {
                    ("200 OK", cat_song_json.clone())
                } else if path.starts_with("/v1/catalog/us/charts") {
                    ("200 OK", charts_json.clone())
                } else {
                    ("404 Not Found", "{}".to_owned())
                };

                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let client = AppleClient::connect_with_base(
            "user-tok",
            &jwt(),
            &format!("http://{addr}/v1"),
            None,
        )
        .await
        .unwrap();

        let steps = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));
        let steps_cb = steps.clone();
        let calls = AtomicUsize::new(0);

        let result = sync_all_with(&pool, &client, &|label| {
            calls.fetch_add(1, Ordering::SeqCst);
            steps_cb.lock().unwrap().push(label);
        })
        .await
        .unwrap();

        assert_eq!(result.liked_count, 3);
        assert_eq!(result.playlist_count, 2);
        assert_eq!(result.album_count, 2);
        assert_eq!(result.artist_count, 1);
        assert_eq!(result.top_track_count, 1);
        assert_eq!(result.top_artist_count, 1);
        assert_eq!(result.recent_count, 1);
        assert_eq!(result.new_release_count, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 8);
        let emitted = steps.lock().unwrap().clone();
        for expected in [
            "liked",
            "playlists",
            "albums",
            "artists",
            "top_tracks",
            "top_artists",
            "recently_played",
            "new_releases",
        ] {
            assert!(emitted.contains(&expected), "missing step {expected}");
        }

        for (table, min_rows) in [
            ("playlists", 2i64),
            ("saved_albums", 2),
            ("followed_artists", 1),
            ("top_tracks", 3),
            ("top_artists", 3),
            ("recently_played", 1),
            ("new_releases", 1),
        ] {
            let (cnt,): (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(cnt, min_rows, "unexpected row count in {table}");
        }

        let (isrc,): (Option<String>,) =
            sqlx::query_as("SELECT isrc FROM tracks WHERE id = '1440857781'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(isrc.as_deref(), Some("GBAAP8300001"));

        let (saved_cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM saved_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(saved_cnt, 3);

        // Verify mirror helpers work on the same schema
        let extra = TrackItem {
            id: "8888".into(),
            name: "Extra Track".into(),
            duration_ms: 123_000,
            explicit: false,
            artists: vec![ArtistItem {
                id: "555".into(),
                name: "New Order".into(),
                image_url: None,
                popularity: None,
            }],
            album: None,
            popularity: None,
        };
        mirror_saved_track(&pool, &extra, true).await.unwrap();
        let (saved_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM saved_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(saved_after, 4);

        mirror_saved_track(&pool, &extra, false).await.unwrap();
        let (saved_unliked,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM saved_tracks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(saved_unliked, 3);

        server.abort();
    }
}
