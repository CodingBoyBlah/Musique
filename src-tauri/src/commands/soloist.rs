//! Spotify's Soloist player as an audio source (see `crate::soloist`).
//!
//! Soloist is chosen like any other source, with `playback_backend` set to
//! "soloist" (see `commands::playback::switch_backend`). What is here is what
//! only it has: the personal key it starts with, the quality it streams at,
//! and what it is doing, pushed to the interface as `soloist:status` and
//! `soloist:format` whenever they change.

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::playback::{save_setting, soloist_chosen, spotify_available, switch_backend};
use crate::errors::AppError;
use crate::soloist::{self, Format, Quality, Status};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SoloistState {
    /// Spotify publishes a build for this kind of computer.
    pub supported: bool,
    /// A personal key is saved, so choosing Soloist needs nothing typed.
    pub has_key:   bool,
    /// Soloist is the audio source.
    pub chosen:    bool,
    pub status:    Status,
    pub quality:   Quality,
    /// What it is streaming, once it has opened a track.
    pub format:    Option<Format>,
}

#[tauri::command]
pub async fn soloist_status(app: AppHandle) -> Result<SoloistState, AppError> {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    let has_key = tauri::async_runtime::spawn_blocking(|| soloist::saved_key().is_some())
        .await
        .unwrap_or(false);
    Ok(SoloistState {
        supported: soloist::install::supported(),
        has_key,
        chosen: soloist_chosen(&pool).await,
        status: player.status(),
        quality: soloist::saved_quality(&pool).await,
        format: player.format(),
    })
}

/// Save a personal key and play through Soloist. `key` is the one from the
/// listener's Spotify developer dashboard; without one the saved key is
/// used. A key given while Soloist already plays replaces the old one and
/// restarts it. The first run downloads the player from Spotify.
#[tauri::command]
pub async fn soloist_enable(app: AppHandle, key: Option<String>) -> Result<SoloistState, AppError> {
    if !soloist::install::supported() {
        return Err(AppError::InvalidInput(
            "Spotify doesn't publish Soloist for this kind of computer".into(),
        ));
    }
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    if !spotify_available(&pool).await {
        return Err(AppError::InvalidInput("Soloist requires Spotify Premium".into()));
    }
    let key = key.map(|key| key.trim().to_string()).filter(|key| !key.is_empty());
    let replaced = key.is_some();
    if let Some(key) = key {
        tauri::async_runtime::spawn_blocking(move || soloist::save_key(&key))
            .await
            .map_err(|e| AppError::Playback(e.to_string()))?
            .map_err(AppError::InvalidInput)?;
    }
    let was_chosen = soloist_chosen(&pool).await;
    switch_backend(&app, "soloist").await?;
    // a new key, or another go after it failed: start it again
    if was_chosen && (replaced || !matches!(player.status(), Status::Ready { .. } | Status::Pairing { .. })) {
        player.start(app.clone(), None).await;
    }
    soloist_status(app).await
}

/// Go back to the built-in player. The key and the sign-in are kept, so
/// choosing Soloist again asks for nothing.
#[tauri::command]
pub async fn soloist_disable(app: AppHandle) -> Result<SoloistState, AppError> {
    let pool = app.state::<AppState>().db.clone();
    if soloist_chosen(&pool).await {
        switch_backend(&app, "spotify").await?;
    }
    soloist_status(app).await
}

/// Choose the quality Soloist streams at. A player that is playing reads it
/// when the next track starts; one that is not restarts now to read it.
#[tauri::command]
pub async fn soloist_set_quality(app: AppHandle, quality: String) -> Result<SoloistState, AppError> {
    let quality = Quality::parse(&quality)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown Soloist quality {quality:?}")))?;
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    save_setting(&pool, "soloist_quality", quality.as_str()).await?;
    if soloist_chosen(&pool).await {
        player.quality_changed(&app).await;
    }
    soloist_status(app).await
}

/// Go back to the built-in player and remove Soloist's key, sign-in and cache.
#[tauri::command]
pub async fn soloist_forget(app: AppHandle) -> Result<SoloistState, AppError> {
    forget(&app).await;
    soloist_status(app).await
}

/// What signing out does to Soloist: the key and the sign-in belong to the
/// account that is leaving.
pub async fn forget(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    if soloist_chosen(&pool).await {
        let _ = switch_backend(app, "spotify").await;
    }
    let _ = save_setting(&pool, "soloist_enabled", "0").await;
    player.stop().await;
    soloist::forget(app).await;
}

/// Start Soloist with the app when it is the audio source, so it is signed
/// in by the first play.
pub async fn start_if_enabled(app: AppHandle) {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    if !soloist_chosen(&pool).await {
        return;
    }
    // turned on the old way: it is a source of its own now
    let _ = save_setting(&pool, "playback_backend", "soloist").await;
    let _ = save_setting(&pool, "soloist_enabled", "0").await;
    player.start(app.clone(), None).await;
}
