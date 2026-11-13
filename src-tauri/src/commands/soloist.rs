//! Turning Spotify's Soloist player on and off (see `crate::soloist`).
//!
//! `soloist_enabled` in settings is the listener's choice; whether the player
//! is actually up is its status, pushed to the interface as `soloist:status`
//! whenever it changes.

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::playback::{save_setting, silence, soloist_enabled, spotify_available, Backend};
use crate::errors::AppError;
use crate::soloist::{self, Status};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct SoloistState {
    /// The listener has turned Soloist on.
    pub enabled:   bool,
    /// Spotify publishes a build for this kind of computer.
    pub supported: bool,
    /// A personal key is saved, so turning it on needs nothing typed.
    pub has_key:   bool,
    pub status:    Status,
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
        enabled: soloist_enabled(&pool).await,
        supported: soloist::install::supported(),
        has_key,
        status: player.status(),
    })
}

/// Turn Soloist on. `key` is the personal key from the listener's Spotify
/// developer dashboard; without one the saved key is used. The first run
/// downloads the player from Spotify, and it then waits to be picked once in
/// a Spotify app's device list.
#[tauri::command]
pub async fn soloist_enable(app: AppHandle, key: Option<String>) -> Result<SoloistState, AppError> {
    if !soloist::install::supported() {
        return Err(AppError::InvalidInput(
            "Spotify does not publish Soloist for this kind of computer".into(),
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

    // the built-in player owns an output of its own and would carry on
    // underneath; whichever of the two was playing stops here
    silence(&app, Backend::Spotify).await;
    save_setting(&pool, "soloist_enabled", "1").await?;
    player.start(app.clone(), key).await;
    soloist_status(app).await
}

/// Turn Soloist off and go back to the built-in player. The key and the
/// pairing are kept, so turning it on again asks for nothing.
#[tauri::command]
pub async fn soloist_disable(app: AppHandle) -> Result<SoloistState, AppError> {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    save_setting(&pool, "soloist_enabled", "0").await?;
    player.stop().await;
    soloist_status(app).await
}

/// Turn Soloist off and remove its key, pairing and cache.
#[tauri::command]
pub async fn soloist_forget(app: AppHandle) -> Result<SoloistState, AppError> {
    forget(&app).await;
    soloist_status(app).await
}

/// What signing out does to Soloist: the key and the pairing belong to the
/// account that is leaving.
pub async fn forget(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    let _ = save_setting(&pool, "soloist_enabled", "0").await;
    player.stop().await;
    soloist::forget(app).await;
}

/// Start Soloist with the app when the listener left it on.
pub async fn start_if_enabled(app: AppHandle) {
    let state = app.state::<AppState>();
    let pool = state.db.clone();
    let player = state.soloist.clone();
    drop(state);
    if soloist_enabled(&pool).await && spotify_available(&pool).await {
        player.start(app.clone(), None).await;
    }
}
