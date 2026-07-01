use tauri::{AppHandle, Manager};

use crate::{errors::AppError, lyrics::{self, Lyrics}, state::AppState};

/// grab the lyrics for a track (or read em from cache). the frontend hands us the
/// track metadata it already has so we dont need a whole spotify round trip
#[tauri::command]
pub async fn get_lyrics(
    app:         AppHandle,
    track_id:    String,
    name:        String,
    artist:      String,
    album:       Option<String>,
    duration_ms: i64,
    force:       Option<bool>,
) -> Result<Lyrics, AppError> {
    let pool = app.state::<AppState>().db.clone();
    lyrics::get_or_fetch(
        &pool,
        &track_id,
        &name,
        &artist,
        album.as_deref().unwrap_or(""),
        duration_ms,
        force.unwrap_or(false),
    )
    .await
}
