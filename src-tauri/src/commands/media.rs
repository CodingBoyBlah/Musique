use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    media_controls::MediaMsg,
    state::AppState,
};

/// called from the frontend when a new track starts playing.
/// updates the os media controls (SMTC / MPRIS / macos) with the track metadata
#[tauri::command]
pub async fn update_now_playing(
    app:         AppHandle,
    title:       String,
    artist:      String,
    album:       String,
    cover_url:   Option<String>,
    track_url:   Option<String>,
    duration_ms: u64,
    position_ms: u64,
) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let _ = s.media_tx.try_send(MediaMsg::NowPlaying {
        title, artist, album, cover_url, track_url, duration_ms, position_ms,
    });
    Ok(())
}

/// enable/disable discord rich presence (general settings). the media controls
/// thread owns the discord client so we just pass the toggle along
#[tauri::command]
pub async fn set_discord_enabled(app: AppHandle, enabled: bool) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let _ = s.media_tx.try_send(MediaMsg::SetDiscordEnabled(enabled));
    Ok(())
}
