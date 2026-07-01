use tauri::{AppHandle, Manager};

use crate::{errors::AppError, lastfm, state::AppState};

#[tauri::command]
pub async fn lastfm_status(app: AppHandle) -> Result<lastfm::LastfmStatus, AppError> {
    let pool = app.state::<AppState>().db.clone();
    lastfm::status(&pool).await
}

#[tauri::command]
pub async fn lastfm_save_api(_app: AppHandle, api_key: String, api_secret: String) -> Result<(), AppError> {
    lastfm::save_api(&api_key, &api_secret)
}


#[tauri::command]
pub async fn lastfm_start_auth(app: AppHandle) -> Result<String, AppError> {
    let (token, url) = lastfm::start_auth().await?;

    
    #[cfg(target_os = "linux")]
    crate::commands::auth::open_url_linux(&url).map_err(AppError::Auth)?;
    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(&url, None::<&str>)
            .map_err(|e| AppError::Auth(format!("Cannot open browser: {e}")))?;
    }

    Ok(token)
}

#[tauri::command]
pub async fn lastfm_finish_auth(app: AppHandle, token: String) -> Result<lastfm::LastfmStatus, AppError> {
    let pool = app.state::<AppState>().db.clone();
    lastfm::finish_auth(&pool, &token).await
}

#[tauri::command]
pub async fn lastfm_disconnect(app: AppHandle) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    lastfm::disconnect(&pool).await
}

#[tauri::command]
pub async fn lastfm_clear(app: AppHandle) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    lastfm::clear_all(&pool).await
}

#[tauri::command]
pub async fn lastfm_now_playing(
    _app: AppHandle,
    artist: String,
    track: String,
    album: String,
) -> Result<(), AppError> {
    lastfm::now_playing(&artist, &track, &album).await
}

#[tauri::command]
pub async fn lastfm_scrobble(
    _app: AppHandle,
    artist: String,
    track: String,
    album: String,
    timestamp: i64,
) -> Result<(), AppError> {
    lastfm::scrobble(&artist, &track, &album, timestamp).await
}
