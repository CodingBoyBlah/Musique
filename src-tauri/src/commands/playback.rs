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
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_inner(app: &AppHandle) -> Result<(), AppError> {
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
        Some(inner) => inner.session_invalid(),
    };
    if rebuild {
        let vol   = read_vol(&db).await;
        let muted = read_muted(&db).await;
        *guard = Some(
            crate::playback::create_inner(app.clone(), db, auth, vol, muted, media_tx).await?
        );
    }
    Ok(())
}

// commands

// pre warms the librespot session so the first play_track is instant.
// call it after login. idempotent so its safe to call over and over
#[tauri::command]
pub async fn warmup_playback(app: AppHandle) -> Result<(), AppError> {
    ensure_inner(&app).await
}

#[tauri::command]
pub async fn play_track(app: AppHandle, id: String) -> Result<(), AppError> {
    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.play_uri(uri, 0)?;
    }
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
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.pause()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_playback(app: AppHandle) -> Result<(), AppError> {
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.resume()?;
    }
    Ok(())
}

// the ▶ button. only resumes if a track is actually loaded, otherwise (fresh
// session right after launch, warmed but nothing loaded, or no session at all)
// it spins up the session and loads the given track at position_ms. this makes sure a
// song shown in the player bar always plays when u hit ▶
#[tauri::command]
pub async fn resume_or_play(app: AppHandle, id: String, position_ms: u32) -> Result<(), AppError> {
    // heal a missing/dead session first (rebuild resets `loaded` to false
    // so we fall thru to a real load below instead of a silent resume)
    ensure_inner(&app).await?;

    let uri      = crate::playback::track_uri(&id)?;
    let playback = app.state::<AppState>().playback.clone();
    let guard    = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        if inner.loaded.load(Ordering::Relaxed) {
            inner.resume()?;                 // track already loaded so just resume
        } else {
            inner.play_uri(uri, position_ms)?;  // fresh session so load it
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_playback(app: AppHandle) -> Result<(), AppError> {
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
    drop(s);

    let level_f = level as f64 / 100.0;

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_level(level_f);
        // keep Spotify's reported device volume in sync with ours
        inner.report_volume(level_f);
    }
    drop(guard);

    save_setting(&pool, "player_volume", &level.to_string()).await
}

#[tauri::command]
pub async fn set_muted(app: AppHandle, muted: bool) -> Result<(), AppError> {
    let s        = app.state::<AppState>();
    let pool     = s.db.clone();
    let playback = s.playback.clone();
    drop(s);

    let guard = playback.lock().await;
    if let Some(inner) = guard.as_ref() {
        inner.volume.set_muted(muted);
    }
    drop(guard);

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
