use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    state::AppState,
};

// helper stuff

async fn read_vol(pool: &sqlx::SqlitePool) -> f64 {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'player_volume'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.and_then(|(v,)| v.parse::<f64>().ok())
        .map(|v| v / 100.0)
        .unwrap_or(0.8)
}

async fn read_muted(pool: &sqlx::SqlitePool) -> bool {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'player_muted'",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(v,)| v == "1").unwrap_or(false)
}

async fn save_setting(pool: &sqlx::SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_inner(app: &AppHandle) -> Result<(), AppError> {
    ensure_inner_with(app, true).await
}

/// podcast playback speed (0.5x - 3x, pitch kept). applies to whatever the
/// sink is playing; the frontend sets it back to 1x for music
#[tauri::command]
pub async fn set_playback_speed(app: AppHandle, speed: f32) -> Result<(), AppError> {
    if !(0.5..=3.0).contains(&speed) {
        return Err(AppError::InvalidInput(format!("speed out of range: {speed}")));
    }
    let before = crate::stretch::speed();
    crate::stretch::set_speed(speed);
    if (before - speed).abs() < 1e-3 {
        return Ok(());
    }
    // seconds of audio are queued ahead, stretched at the old speed. the
    // youtube backend's prebuffer starts over on its own; spotify's player
    // has to be sent back to what is being heard
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.speed_changed()?;
    }
    Ok(())
}

/// bring the librespot session up for the internal api layer (spclient calls)
/// without ever popping a browser authorization at the user
pub(crate) async fn warm_session(app: &AppHandle) -> Result<(), AppError> {
    ensure_inner_with(app, false).await
}

/// `interactive = false` never opens a browser authorization; see `create_inner`.
async fn ensure_inner_with(app: &AppHandle, interactive: bool) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let db       = s.db.clone();
    let auth     = s.auth.clone();
    let playback = s.playback.clone();
    let media_tx = s.media_tx.clone();
    drop(s);

    let mut guard = playback.lock().await;
    // rebuild when theres no session yet OR the existing one went invalid
    // (ap disconnect / expiry). a stale session just eats load/play calls so
    // the ui says "playing" but no audio comes out. healing here fixes that
    let rebuild = match guard.as_ref() {
        None        => true,
        Some(inner) => inner.session_invalid() || inner.needs_rebuild.load(Ordering::Relaxed),
    };
    if rebuild {
        let vol   = read_vol(&db).await;
        let muted = read_muted(&db).await;
        *guard = Some(
            crate::playback::create_inner(app.clone(), db, auth, vol, muted, media_tx, interactive).await?
        );
    }
    Ok(())
}

// backend routing
//
// Free Spotify accounts cannot stream through librespot - Spotify gates audio
// delivery on Premium, so every track comes back Unavailable no matter how
// healthy the session is. Those accounts get audio from YouTube Music instead
// (see `crate::youtube`), while Spotify continues to supply all metadata,
// artwork, lyrics and library data.
//
// The librespot session is still built on a free account: Connect device
// registration and the spclient calls the lyrics pipeline makes both keep
// working. Only the audio path changes.

/// Which backend supplies audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Spotify,
    YouTube,
}

async fn setting_value(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|(v,)| v)
        .filter(|v| !v.is_empty())
}

/// Whether the signed-in account can stream through Spotify at all.
///
/// Spotify only delivers audio to Premium, so this is what makes the Spotify
/// backend available rather than a setting. Treating an unknown product as
/// non-premium is the safe direction: guessing "premium" wrong gives silent
/// playback that looks like a bug, while guessing "free" wrong merely routes a
/// Premium user through a path that works.
pub(crate) async fn spotify_available(pool: &sqlx::SqlitePool) -> bool {
    setting_value(pool, "spotify_product").await.as_deref() == Some("premium")
}

/// Decide which audio backend supplies audio.
///
/// Free accounts have no choice - Spotify cannot stream to them, so they are
/// always on YouTube. Premium accounts pick with `playback_backend` and default
/// to Spotify, since that is the service they are paying for.
///
/// Any unrecognised stored value (including the legacy `"auto"` this setting
/// used to accept) falls through to that default, so old settings rows resolve
/// sensibly instead of needing a migration.
async fn backend(pool: &sqlx::SqlitePool) -> Backend {
    if !spotify_available(pool).await {
        return Backend::YouTube;
    }
    match setting_value(pool, "playback_backend").await.as_deref() {
        Some("youtube") => Backend::YouTube,
        _ => Backend::Spotify,
    }
}

async fn uses_youtube(app: &AppHandle) -> bool {
    let pool = app.state::<AppState>().db.clone();
    backend(&pool).await == Backend::YouTube
}

/// Build the YouTube audio backend if it does not exist yet.
async fn ensure_yt(app: &AppHandle) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let db       = s.db.clone();
    let yt       = s.yt.clone();
    let media_tx = s.media_tx.clone();
    drop(s);

    let mut guard = yt.lock().await;
    if guard.is_none() {
        let vol   = read_vol(&db).await;
        let muted = read_muted(&db).await;
        *guard = Some(crate::playback::youtube::YtPlayback::new(
            app.clone(),
            crate::playback::SharedVolume::new(vol, muted),
            media_tx,
        )?);
    }
    Ok(())
}

/// Build the match query for a track, fetching it from Spotify if the local
/// catalog doesn't have it yet.
///
/// The local `tracks` table only holds what library sync and page loads have
/// written. Plenty of playable tracks never pass through either - anything
/// started from search results, radio, a queue entry or an autoplay
/// continuation can be pressed play on while its row does not exist. Treating
/// that as "track not found" made every such play fail on the YouTube backend.
///
/// The query is built from the API response directly rather than by upserting
/// and re-reading, so playback does not depend on the cache write succeeding.
/// The upsert still happens, best-effort, so the next play skips the round trip.
async fn match_query(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<crate::youtube::TrackQuery, AppError> {
    match crate::youtube::query_for_track(pool, id).await {
        Ok(q) => return Ok(q),
        // Only a missing/incomplete row is worth a network fallback; a real
        // database error should surface as itself.
        Err(AppError::NotFound(_)) => {}
        Err(e) => return Err(e),
    }

    let token = crate::commands::spotify::tok(app).await?;
    let track: crate::spotify::types::SpTrack = crate::spotify::spotify_get(
        &token,
        &format!("{}/tracks/{id}", crate::commands::spotify::BASE),
    )
    .await?;

    if track.artists.is_empty() {
        // Without an artist the matcher can't tell two identically titled
        // songs apart, so refuse rather than match on title alone.
        return Err(AppError::NotFound(format!(
            "\"{}\" has no artist information; can't match it safely",
            track.name
        )));
    }

    let query = crate::youtube::TrackQuery {
        title:       track.name.clone(),
        artists:     track.artists.iter().map(|a| a.name.clone()).collect(),
        album:       track.album.as_ref().map(|a| a.name.clone()),
        duration_ms: track.duration_ms.max(0) as u64,
        explicit:    track.explicit,
    };

    // Best-effort catalog fill. Failures here are logged, never fatal - the
    // query above is already complete.
    if let Err(e) = cache_track(pool, &track).await {
        eprintln!("[playback] caching {id} for later failed: {e}");
    }

    Ok(query)
}

/// Write a fetched track into the local catalog: album, artists, track and the
/// join rows `query_for_track` reads back. The album has to go first -
/// `tracks.album_id` is a foreign key, so upserting the track alone fails for
/// exactly the not-yet-cached tracks this fallback exists for.
async fn cache_track(
    pool: &sqlx::SqlitePool,
    track: &crate::spotify::types::SpTrack,
) -> Result<(), AppError> {
    crate::library::upsert_track_with_deps(pool, track).await
}

/// Latest YouTube transport request. `yt_play` downloads before it takes the
/// player lock, so a slow load can finish after the user has already moved on
/// (clicked another track, paused). Every transport command claims a new
/// ticket; a load whose ticket is no longer current when its download finishes
/// is dropped instead of replacing what the user asked for last.
static YT_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn yt_claim() -> u64 {
    YT_REQUEST.fetch_add(1, Ordering::SeqCst) + 1
}

/// Resolve, fetch and play a Spotify track through YouTube Music.
///
/// Resolution, extraction and download all happen *before* the state lock is
/// taken. The download is ~1s of network work; holding the playback mutex
/// across it would block every transport command for its duration.
async fn yt_play(app: &AppHandle, id: &str, position_ms: u32) -> Result<(), AppError> {
    if let Some(episode_id) = id.strip_prefix("spotify:episode:") {
        return yt_play_episode(app, id, episode_id, position_ms).await;
    }
    let ticket = yt_claim();
    ensure_yt(app).await?;
    let pool = app.state::<AppState>().db.clone();

    let query = match_query(app, &pool, id).await?;
    // Uses the prefetched copy when `preload_track` already fetched this track,
    // which is the difference between an instant track change and a ~1s gap.
    let prepared = crate::youtube::prepare(&pool, id, &query).await?;

    let yt    = app.state::<AppState>().yt.clone();
    let guard = yt.lock().await;
    // Checked under the lock: a newer yt_play also plays under it, so this
    // cannot interleave with the load that superseded us.
    if YT_REQUEST.load(Ordering::SeqCst) != ticket {
        eprintln!("[youtube] dropping superseded load of {id}");
        return Ok(());
    }
    if let Some(player) = guard.as_ref() {
        player.play_track(
            id,
            &prepared.stream,
            std::sync::Arc::clone(&prepared.audio),
            position_ms,
        )?;
    }
    Ok(())
}

/// Play a podcast episode on the YouTube backend. Spotify only streams episodes
/// to its own session, so the audio comes from the show's RSS feed (or YouTube
/// Music) instead - see `episode_audio`. Episodes stream rather than download
/// whole, so this starts on the first chunk.
async fn yt_play_episode(
    app: &AppHandle,
    id: &str,
    episode_id: &str,
    position_ms: u32,
) -> Result<(), AppError> {
    use rodio::Source;

    let ticket = yt_claim();
    let current = || YT_REQUEST.load(Ordering::SeqCst) == ticket;
    ensure_yt(app).await?;
    let pool = app.state::<AppState>().db.clone();
    let token = crate::commands::spotify::tok(app).await?;

    let found = crate::episode_audio::resolve(&pool, &token, episode_id).await?;
    if !current() {
        return Ok(());
    }
    let opened = match crate::episode_audio::remote::open(&found.url, found.user_agent).await {
        Ok(opened) => opened,
        Err(e) => {
            // a remembered url that stopped working shouldn't stick
            crate::episode_audio::forget(&pool, episode_id).await;
            return Err(e);
        }
    };
    if !current() {
        return Ok(());
    }
    eprintln!("[youtube] episode {episode_id} via {} ({} bytes)", found.via, opened.len);

    let handle = opened.handle.clone();
    if position_ms > 0 && found.duration_ms > 0 {
        // rough (see EpisodeStream::byte_for); the exact spot for an mp4 is
        // only known once its index is read, below
        let byte = (opened.len as u128 * position_ms as u128 / found.duration_ms as u128) as u64;
        handle.prefetch(byte.saturating_sub(64 * 1024), std::time::Duration::from_secs(8)).await;
    }

    // the container type without its codec parameters, which the probe
    // doesn't match on; the feed's word first, the server's second
    let mime = found
        .mime
        .or(opened.content_type)
        .map(|m| m.split(';').next().unwrap_or_default().trim().to_string())
        .filter(|m| m.starts_with("audio/"));
    let reader = opened.reader;
    let clock = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let source_clock = std::sync::Arc::clone(&clock);
    // probing and the first seek read the file, which can wait on the network,
    // so they run on a blocking thread rather than an async worker
    let (source, segments, started_at) = tokio::task::spawn_blocking(move || -> Result<_, AppError> {
        let mut decoder = crate::episode_audio::source::EpisodeDecoder::open(reader, mime)
            .map_err(|e| AppError::Playback(format!("episode decode: {e}")))?;
        let segments = decoder.segments();
        let mut start = 0u32;
        if position_ms > 0 {
            match decoder.try_seek(std::time::Duration::from_millis(position_ms as u64)) {
                Ok(()) => start = position_ms,
                Err(e) => eprintln!("[youtube] episode seek to {position_ms}ms on load failed: {e}"),
            }
        }
        Ok((crate::episode_audio::source::EpisodeSource::new(decoder, source_clock, start as u64), segments, start))
    })
    .await
    .map_err(|e| AppError::Playback(format!("episode decode: {e}")))??;

    let stream = crate::playback::youtube::EpisodeStream { handle, duration_ms: found.duration_ms, segments };
    let yt = app.state::<AppState>().yt.clone();
    let guard = yt.lock().await;
    if !current() {
        eprintln!("[youtube] dropping superseded load of {id}");
        return Ok(());
    }
    if let Some(player) = guard.as_ref() {
        player.play_episode(id, source, clock, stream, started_at);
    }
    Ok(())
}

/// Seek on the YouTube backend. Music is fully in memory, so that's instant;
/// a streaming episode first pulls the bytes around the target so the audio
/// thread doesn't sit waiting on the network mid-seek.
async fn yt_seek(app: &AppHandle, position_ms: u32) -> Result<(), AppError> {
    let yt = app.state::<AppState>().yt.clone();
    let episode = yt.lock().await.as_ref().and_then(|p| p.episode_stream());
    if let Some(episode) = episode {
        if let Some(byte) = episode.byte_for(position_ms) {
            episode.handle.prefetch(byte, std::time::Duration::from_secs(6)).await;
        }
    }
    let guard = yt.lock().await;
    if let Some(player) = guard.as_ref() {
        player.seek(position_ms)?;
    }
    Ok(())
}

/// Resolve, extract and download a track ahead of playing it.
///
/// Fire-and-forget: a preload failing must never surface to the user or block
/// anything, because the track may never actually be played. If it does get
/// played, `yt_play` simply pays the cost then instead.
async fn yt_preload(app: &AppHandle, id: &str) {
    let pool = app.state::<AppState>().db.clone();
    if let Some(episode_id) = id.strip_prefix("spotify:episode:") {
        // not the audio (that's hundreds of MB) - just find where it lives,
        // so pressing play skips the feed lookup
        if let Ok(token) = crate::commands::spotify::tok(app).await {
            if let Err(e) = crate::episode_audio::resolve(&pool, &token, episode_id).await {
                eprintln!("[youtube] preload {id}: {e}");
            }
        }
        return;
    }
    let query = match match_query(app, &pool, id).await {
        Ok(q) => q,
        Err(e) => {
            eprintln!("[youtube] preload {id}: {e}");
            return;
        }
    };
    match crate::youtube::prepare(&pool, id, &query).await {
        Ok(p) => eprintln!(
            "[youtube] preloaded {id} ({} bytes, itag {})",
            p.audio.len(),
            p.stream.format.itag
        ),
        Err(e) => eprintln!("[youtube] preload {id}: {e}"),
    }
}

// backend / match inspection commands
//
// These exist so the YouTube path is never a black box to the user. When audio
// comes from somewhere other than Spotify, they can see that it does, see which
// upload was chosen, and correct it. A wrong match the user cannot fix would be
// worse than no match at all.

#[derive(Debug, Clone, Serialize)]
pub struct BackendState {
    /// Backend actually in use right now: "spotify" or "youtube".
    pub active:            String,
    /// Spotify account product, when known.
    pub product:           Option<String>,
    /// Whether Spotify is a selectable option. False on a free account, which
    /// is what the settings UI disables the Spotify choice on.
    pub spotify_available: bool,
}

#[tauri::command]
pub async fn get_playback_backend(app: AppHandle) -> Result<BackendState, AppError> {
    let pool = app.state::<AppState>().db.clone();

    // `active` doubles as the selected value in the UI. There is no separate
    // "setting" to report any more: with the automatic option gone, the
    // resolved backend *is* the selection, and a free account's forced
    // YouTube shows up as YouTube being selected rather than as a stored
    // preference that disagrees with reality.
    Ok(BackendState {
        active: match backend(&pool).await {
            Backend::Spotify => "spotify".into(),
            Backend::YouTube => "youtube".into(),
        },
        product:           setting_value(&pool, "spotify_product").await,
        spotify_available: spotify_available(&pool).await,
    })
}

/// Choose the audio backend. Accepts "spotify" or "youtube".
#[tauri::command]
pub async fn set_playback_backend(app: AppHandle, mode: String) -> Result<(), AppError> {
    if !matches!(mode.as_str(), "spotify" | "youtube") {
        return Err(AppError::InvalidInput(format!(
            "unknown playback backend {mode:?}; expected spotify or youtube"
        )));
    }
    let pool = app.state::<AppState>().db.clone();

    // The UI disables this choice on a free account, but enforce it here too -
    // storing "spotify" for an account that cannot stream would produce silent
    // playback with no visible cause.
    if mode == "spotify" && !spotify_available(&pool).await {
        return Err(AppError::InvalidInput(
            "Spotify playback requires Premium".into(),
        ));
    }

    // Captured before the write, since `backend()` resolves from the setting.
    let previous = backend(&pool).await;
    save_setting(&pool, "playback_backend", &mode).await?;
    let next = backend(&pool).await;

    if previous != next {
        yt_claim();
        // Silence whichever backend was playing. Each owns its own audio
        // device and neither knows about the other, so without this the old
        // one keeps playing underneath the new one and you hear both tracks
        // at once.
        //
        // Pause rather than stop: the track and position survive, so pressing
        // play resumes where it left off, now through the new backend.
        match previous {
            Backend::Spotify => {
                let playback = app.state::<AppState>().playback.clone();
                let guard = playback.lock().await;
                if let Some(inner) = guard.as_ref() {
                    if let Err(e) = inner.pause() {
                        eprintln!("[playback] pausing spotify on backend switch: {e}");
                    }
                }
            }
            Backend::YouTube => {
                let yt = app.state::<AppState>().yt.clone();
                let guard = yt.lock().await;
                if let Some(player) = guard.as_ref() {
                    if let Err(e) = player.pause() {
                        eprintln!("[playback] pausing youtube on backend switch: {e}");
                    }
                }
            }
        }
    }

    // Prepared YouTube buffers are useless once audio comes from Spotify, and
    // holding several megabytes for a backend that is no longer running is
    // pure waste.
    if next == Backend::Spotify {
        crate::youtube::clear_prepared().await;
    }
    Ok(())
}

/// The cached YouTube mapping for a track, if any.
#[tauri::command]
pub async fn get_yt_match(
    app: AppHandle,
    track_id: String,
) -> Result<Option<crate::db::repos::yt_match::YtMatch>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::get(&pool, &track_id).await
}

#[derive(Debug, Clone, Serialize)]
pub struct YtCandidate {
    pub video_id:    String,
    pub title:       String,
    pub artists:     Vec<String>,
    pub album:       Option<String>,
    pub duration_ms: Option<u64>,
    pub explicit:    bool,
}

/// Search YouTube Music for a track and return the raw candidates, ungated.
///
/// This powers the manual-override picker, so it deliberately does *not* apply
/// the matcher's gates: the whole point is to show the user what was rejected
/// so they can choose for themselves. Automatic playback never uses this.
#[tauri::command]
pub async fn search_yt_candidates(
    app: AppHandle,
    track_id: String,
) -> Result<Vec<YtCandidate>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    // Same fallback as playback - the picker must work for a track that is not
    // in the local catalog, which is exactly when a user is most likely to be
    // investigating a failed match.
    let query = match_query(&app, &pool, &track_id).await?;
    let terms = format!(
        "{} {}",
        query.title,
        query.artists.first().map(String::as_str).unwrap_or_default()
    );

    Ok(crate::youtube::search::songs(terms.trim())
        .await?
        .into_iter()
        .map(|r| YtCandidate {
            video_id:    r.video_id,
            title:       r.title,
            artists:     r.artists,
            album:       r.album,
            duration_ms: r.duration_ms,
            explicit:    r.explicit,
        })
        .collect())
}

/// Pin a user-chosen video for a track. Survives re-resolution.
#[tauri::command]
pub async fn pin_yt_match(
    app: AppHandle,
    track_id: String,
    video_id: String,
) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::pin(&pool, &track_id, &video_id).await?;
    // Otherwise the next play reuses the already-downloaded wrong upload.
    crate::youtube::evict_prepared(&track_id).await;
    Ok(())
}

/// Forget a cached mapping so the next play re-resolves it.
#[tauri::command]
pub async fn forget_yt_match(app: AppHandle, track_id: String) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::db::repos::yt_match::forget(&pool, &track_id).await?;
    crate::youtube::evict_prepared(&track_id).await;
    Ok(())
}

// commands

// pre warms the librespot session so the first play_track is instant.
// call it after login. idempotent so its safe to call over and over
#[tauri::command]
pub async fn warmup_playback(app: AppHandle) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        // Scraping the visitor id costs a ~380 KB page fetch, measured at
        // ~630ms. Cached process-wide once fetched, so doing it here moves it
        // off the first play entirely. Detached because nothing below needs it
        // and a failure is non-fatal (see visitor.rs).
        tokio::spawn(async { crate::youtube::visitor::get().await; });

        // Opening the audio device is the other one-time cost on this path.
        if let Err(e) = ensure_yt(&app).await {
            eprintln!("[playback] youtube warmup failed: {e}");
        }
    }
    ensure_inner(&app).await
}

/// Startup warm-up, fired from `setup()` the moment app state exists.
///
/// The frontend also calls `warmup_playback`, but only after the webview has
/// booted, mounted React, round-tripped `get_auth_status` over IPC and flipped
/// `isLoggedIn`. Building the session is the expensive part of the first play -
/// connecting to the access point, registering the Connect device, waiting on
/// the country packet - and none of it depends on the UI. Starting it here runs
/// it concurrently with webview startup, so by the time there is something to
/// click, the session is usually already up.
///
/// Deliberately conservative: this returns early unless everything needed is
/// already on disk, and builds the session non-interactively. A stored playback
/// token is usually expired by the next launch, so the on-disk check alone does
/// not stop `create_inner` from falling back to a browser authorization - the
/// `interactive = false` flag does. A background task must never do that
/// unprompted; if no silent credential works, the user's first play recovers.
pub async fn warm_session_if_possible(app: AppHandle) {
    use tauri::Manager;

    let s = app.state::<AppState>();
    let db = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    // not logged in yet -> nothing to warm, and get_valid_token would just fail
    if auth.read().await.refresh_token.is_none() {
        return;
    }

    // would create_inner have to go interactive? if so, leave it to the user's
    // first real play (or the frontend's warmup after an explicit login).
    let has_cached_creds = app
        .path()
        .app_data_dir()
        .map(|d| d.join("credentials").join("credentials.json").exists())
        .unwrap_or(false);
    if !has_cached_creds {
        let has_token = crate::auth::get_setting_value(&db, "spotify_playback_token")
            .await
            .ok()
            .flatten()
            .is_some_and(|t| !t.trim().is_empty());
        if !has_token {
            return;
        }
    }

    if let Err(e) = ensure_inner_with(&app, false).await {
        eprintln!("[playback] startup warm-up skipped: {e}");
    }
}

#[tauri::command]
pub async fn play_track(app: AppHandle, id: String) -> Result<(), AppError> {
    eprintln!("[playback cmd] play_track id={id}");

    if uses_youtube(&app).await {
        // A resolution failure here is a real, expected outcome (no acceptable
        // YouTube match). It propagates to the frontend as an error naming the
        // track rather than silently playing something else.
        yt_play(&app, &id, 0).await?;
    } else {
        ensure_inner(&app).await?;
        let uri      = crate::playback::track_uri(&id)?;
        let playback = app.state::<AppState>().playback.clone();
        let guard    = playback.lock().await;
        if let Some(inner) = guard.as_ref() {
            inner.play_uri(uri, 0)?;
        }
    }

    let pool = app.state::<AppState>().db.clone();
    let track_id = id.clone();
    tokio::spawn(async move {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let _ = sqlx::query("INSERT INTO recently_played (track_id, played_at) VALUES (?, ?)")
            .bind(&track_id)
            .bind(now)
            .execute(&pool)
            .await;
    });

    Ok(())
}

// recovery path for a track that came back Unavailable. every so often the
// access point just stops answering audio key requests for a given librespot session
// (the key request times out even after in session retries) which makes an
// otherwise playable track fail to load. a brand new session almost always gets
// a healthy key channel so here we tear the current session down and rebuild a
// fresh one before loading the track again. the frontend calls this AT MOST once
// per track (see App.tsx) so a genuinely unavailable track cant loop. safe
// cuz it only runs when the failed track is the current one, i.e nothings
// actually playing to interrupt
#[tauri::command]
pub async fn retry_play_track(app: AppHandle, id: String) -> Result<(), AppError> {
    {
        // drop the wedged session so ensure_inner just builds a fresh one
        let playback = app.state::<AppState>().playback.clone();
        let mut guard = playback.lock().await;
        *guard = None;
    }
    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.play_uri(uri, 0)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_playback(app: AppHandle) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        // cancels a load still downloading, so it can't start audio after this
        yt_claim();
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.pause()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_playback(app: AppHandle) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        // After end-of-track the decoder queue is drained, so there is nothing
        // to un-pause - the track has to be fetched and decoded again.
        let (ended, track) = {
            let yt = app.state::<AppState>().yt.clone();
            let guard = yt.lock().await;
            match guard.as_ref() {
                Some(p) => (p.is_ended(), p.current_track()),
                None => (false, None),
            }
        };

        match (ended, track) {
            (true, Some(id)) => yt_play(&app, &id, 0).await?,
            _ => {
                yt_claim();
                let yt = app.state::<AppState>().yt.clone();
                let guard = yt.lock().await;
                if let Some(player) = guard.as_ref() {
                    player.resume()?;
                }
            }
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        if !inner.is_ended() {
            if let Err(e) = inner.resume() {
                eprintln!("[playback cmd] resume failed ({e}), falling back to play_uri");
                if let Some(uri) = inner.current_uri() {
                    inner.play_uri(uri, 0)?;
                }
            }
        } else if let Some(uri) = inner.current_uri() {
            inner.play_uri(uri, 0)?;
        }
    }
    Ok(())
}

// the ▶ button. resumes if the matching track is loaded and not ended,
// otherwise loads and plays the requested track at position_ms.
#[tauri::command]
pub async fn resume_or_play(app: AppHandle, id: String, position_ms: u32) -> Result<(), AppError> {
    eprintln!("[playback cmd] resume_or_play id={id} pos={position_ms}");

    if uses_youtube(&app).await {
        // Un-pause when the very same track is still loaded and live; anything
        // else is a fresh load. The frontend passes the current position on
        // every resume, so a non-zero position must not force a reload - the
        // whole track is in memory, so seek in place when it has drifted.
        let resumable = {
            let yt = app.state::<AppState>().yt.clone();
            let guard = yt.lock().await;
            guard.as_ref().is_some_and(|p| {
                p.current_track().as_deref() == Some(id.as_str()) && !p.is_ended()
            })
        };

        if resumable {
            yt_claim();
            let yt = app.state::<AppState>().yt.clone();
            let drifted = yt
                .lock()
                .await
                .as_ref()
                .is_some_and(|p| position_ms > 0 && p.position_ms().abs_diff(position_ms) > 2000);
            if drifted {
                if let Err(e) = yt_seek(&app, position_ms).await {
                    eprintln!("[youtube] seek on resume failed: {e}");
                }
            }
            let guard = yt.lock().await;
            if let Some(player) = guard.as_ref() {
                player.resume()?;
            }
        } else {
            yt_play(&app, &id, position_ms).await?;
        }
        return Ok(());
    }

    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        let loaded = inner.loaded.load(Ordering::Relaxed);
        let same_uri = inner.current_uri().as_deref() == Some(&uri);
        let ended = inner.is_ended();

        if loaded && same_uri && !ended && position_ms == 0 {
            if let Err(e) = inner.resume() {
                eprintln!("[playback cmd] resume failed ({e}), falling back to play_uri");
                inner.play_uri(uri, position_ms)?;
            }
        } else {
            inner.play_uri(uri, position_ms)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_playback(app: AppHandle) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        yt_claim();
        // Unlike the Spirc path there is no connect-state to keep consistent,
        // so pausing is the right analogue: it keeps the decoded track loaded
        // so pressing play again is instant instead of a re-download.
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        if let Some(player) = guard.as_ref() {
            player.pause()?;
        }
        return Ok(());
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        // Spirc has no hard "stop"; pause keeps the connect-state consistent
        // (device stays active/paused) instead of desyncing by poking the player
        // directly. The frontend uses this rarely (mostly pause is what's wanted).
        inner.pause()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn seek_playback(app: AppHandle, position_ms: u32) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        return yt_seek(&app, position_ms).await;
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.seek(position_ms)?;
    }
    Ok(())
}

// preload the next track so it kicks in instantly when the current one ends
#[tauri::command]
pub async fn preload_track(app: AppHandle, id: String) -> Result<(), AppError> {
    if uses_youtube(&app).await {
        // Returns immediately - the frontend fires this on a timer while a
        // track is playing, so it must not hold up the IPC call or the caller.
        let app = app.clone();
        tokio::spawn(async move { yt_preload(&app, &id).await });
        return Ok(());
    }

    let track_id = crate::playback::parse_track_id(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.player.preload(track_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn set_volume(app: AppHandle, level: u8) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    let yt       = s.yt.clone();
    drop(s);

    let level_f = level as f64 / 100.0;

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_level(level_f);
        // keep Spotify's reported device volume in sync with ours
        inner.report_volume(level_f);
    }
    drop(guard);

    // The two backends own separate volume state, so both are updated
    // unconditionally rather than only the active one - otherwise switching
    // backends would jump the volume back to whatever the other last saw.
    let yt_guard = yt.lock().await;
    if let Some(player) = yt_guard.as_ref() {
        player.set_level(level_f);
    }
    drop(yt_guard);

    save_setting(&pool, "player_volume", &level.to_string()).await
}

#[tauri::command]
pub async fn set_muted(app: AppHandle, muted: bool) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    let yt       = s.yt.clone();
    drop(s);

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_muted(muted);
    }
    drop(guard);

    let yt_guard = yt.lock().await;
    if let Some(player) = yt_guard.as_ref() {
        player.set_muted(muted);
    }
    drop(yt_guard);

    save_setting(&pool, "player_muted", if muted { "1" } else { "0" }).await
}

#[derive(Debug, Clone, Serialize)]
pub struct VolumeState {
    pub level: u8,   // 0-100
    pub muted: bool,
}

#[tauri::command]
pub async fn get_volume(app: AppHandle) -> Result<VolumeState, AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        return Ok(VolumeState {
            level: (inner.volume.level() * 100.0).round() as u8,
            muted: inner.volume.is_muted(),
        });
    }
    drop(guard);

    let level = (read_vol(&pool).await * 100.0).round() as u8;
    let muted = read_muted(&pool).await;
    Ok(VolumeState { level, muted })
}

#[tauri::command]
pub async fn get_audio_quality(app: AppHandle) -> Result<String, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    drop(s);

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'audio_quality'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    Ok(row.map(|(v,)| v).unwrap_or_else(|| "320".to_string()))
}

#[tauri::command]
pub async fn set_audio_quality(app: AppHandle, quality: String) -> Result<(), AppError> {
    let val = match quality.as_str() {
        "96" => "96",
        "160" => "160",
        _ => "320",
    };

    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    save_setting(&pool, "audio_quality", val).await?;

    // Reset playback session if idle so the new bitrate takes effect on next playback.
    // If music is actively playing, keep the session so it doesn't cut off mid-track.
    let mut guard = playback.lock().await;
    let playing = guard.as_ref().map(|i| i.is_playing()).unwrap_or(false);
    if !playing {
        *guard = None;
    } else if let Some(inner) = guard.as_ref() {
        inner.needs_rebuild.store(true, Ordering::Relaxed);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_audio_cache_limit(app: AppHandle) -> Result<u64, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    drop(s);

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM settings WHERE key = 'audio_cache_limit_mb'",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let limit = row.and_then(|(v,)| v.parse::<u64>().ok()).unwrap_or(2048);
    Ok(limit)
}

#[tauri::command]
pub async fn set_audio_cache_limit(app: AppHandle, limit_mb: u64) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    save_setting(&pool, "audio_cache_limit_mb", &limit_mb.to_string()).await?;

    let mut guard = playback.lock().await;
    let playing = guard.as_ref().map(|i| i.is_playing()).unwrap_or(false);
    if !playing {
        *guard = None;
    } else if let Some(inner) = guard.as_ref() {
        inner.needs_rebuild.store(true, Ordering::Relaxed);
    }

    Ok(())
}

/// Returns the current audio output latency in milliseconds.
///
/// Accounts for both the audio chunks queued in rodio's sink and the
/// negotiated hardware buffer duration granted to cpal. Returns 0 when
/// playback is not initialized or when running through the null sink.
#[tauri::command]
pub async fn get_output_latency_ms(app: AppHandle) -> Result<i64, AppError> {
    // Has to follow the active backend. The librespot sink only updates its
    // figure while it is actually writing audio, so on the YouTube path it
    // reports 0 - and `lib/outputLatency.ts` reads 0 as "no sink" and keeps its
    // 250ms placeholder. That placeholder is what was desyncing lyrics, since
    // the real buffer is nothing like 250ms.
    if uses_youtube(&app).await {
        let yt = app.state::<AppState>().yt.clone();
        let guard = yt.lock().await;
        return Ok(guard.as_ref().map(|p| p.output_latency_ms()).unwrap_or(0));
    }

    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    let latency = guard
        .as_ref()
        .map(|inner| inner.output_latency_ms())
        .unwrap_or(0);
    Ok(latency)
}

// ── connect-driven playback (spotify jam) ───────────────────────────────────
//
// in a jam the queue belongs to spotify: spirc holds it and social-connect
// edits it for everyone. these drive spirc's queue directly instead of the
// app's own, and the ui mirrors it from the "connect:state" event.

async fn with_spirc<T>(
    app: &AppHandle,
    f: impl FnOnce(&crate::playback::PlaybackInner) -> Result<T, AppError>,
) -> Result<T, AppError> {
    if uses_youtube(app).await {
        return Err(AppError::Playback(
            "Jams play through Spotify. Switch the audio source to Spotify in Settings.".into(),
        ));
    }
    ensure_inner(app).await?;
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    let inner = guard
        .as_ref()
        .ok_or_else(|| AppError::Playback("spotify session unavailable".into()))?;
    f(inner)
}

/// the queue, current track and context this device last reported
#[tauri::command]
pub async fn get_connect_state(app: AppHandle) -> Result<Option<crate::playback::ConnectStateMsg>, AppError> {
    let playback = app.state::<AppState>().playback.clone();
    let guard = playback.lock().await;
    Ok(guard.as_ref().map(|inner| inner.connect_state()))
}

/// play a list through spirc so it becomes the connect queue (a jam host's
/// music is what the jam hears). `keep_stream` carries on the track already
/// playing when it's the one at `index`
#[tauri::command]
pub async fn connect_load_tracks(
    app: AppHandle,
    ids: Vec<String>,
    index: u32,
    position_ms: u32,
    start_playing: bool,
    keep_stream: bool,
) -> Result<(), AppError> {
    let uris = ids
        .iter()
        .map(|id| crate::playback::track_uri(id))
        .collect::<Result<Vec<_>, _>>()?;
    if uris.is_empty() {
        return Err(AppError::InvalidInput("nothing to play".into()));
    }
    let index = index.min(uris.len() as u32 - 1);
    with_spirc(&app, |inner| inner.load_tracks(uris, index, position_ms, start_playing, keep_stream)).await
}

#[tauri::command]
pub async fn connect_add_to_queue(app: AppHandle, id: String) -> Result<(), AppError> {
    let uri = crate::playback::track_uri(&id)?;
    with_spirc(&app, |inner| inner.add_to_queue(uri)).await
}

#[tauri::command]
pub async fn connect_skip_to(app: AppHandle, id: String) -> Result<(), AppError> {
    let uri = crate::playback::track_uri(&id)?;
    with_spirc(&app, |inner| inner.skip_to(uri)).await
}

#[tauri::command]
pub async fn connect_next(app: AppHandle) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.next()).await
}

#[tauri::command]
pub async fn connect_prev(app: AppHandle) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.prev()).await
}

#[tauri::command]
pub async fn connect_set_shuffle(app: AppHandle, on: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_shuffle(on)).await
}

#[tauri::command]
pub async fn connect_set_repeat(app: AppHandle, context: bool, track: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_repeat(context, track)).await
}

/// a jam guest's pause is theirs alone (hold = true); resuming rejoins the
/// jam where it has got to meanwhile
#[tauri::command]
pub async fn jam_hold(app: AppHandle, hold: bool) -> Result<(), AppError> {
    with_spirc(&app, |inner| inner.set_jam_hold(hold)).await
}
