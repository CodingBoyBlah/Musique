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

/// displays a native desktop notification when a new track starts playing
#[tauri::command]
pub async fn show_playback_notification(
    _app: AppHandle,
    title: String,
    body: String,
    icon: Option<String>,
) -> Result<(), AppError> {
    let local_icon: Option<String> = match icon {
        Some(ref icon_str) if icon_str.starts_with("http://") || icon_str.starts_with("https://") => {
            if let Ok(resp) = reqwest::get(icon_str).await {
                if let Ok(bytes) = resp.bytes().await {
                    let temp_file = std::env::temp_dir().join("musique_now_playing.jpg");
                    if tokio::fs::write(&temp_file, &bytes).await.is_ok() {
                        Some(temp_file.to_string_lossy().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Some(icon_str) => Some(icon_str),
        None => None,
    };

    #[cfg(target_os = "windows")]
    {
        let app_handle = _app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            use tauri_winrt_notification::{IconCrop, Toast};

            let mut toast = Toast::new("dev.boyblah.musique")
                .title(&title)
                .text1(&body);

            // Compact layout: use the album cover as the thumbnail on the left.
            // Falls back to the Musique logo if no album artwork is available.
            if let Some(ref cp) = local_icon {
                toast = toast.icon(std::path::Path::new(cp), IconCrop::Square, "Album Art");
            } else if let Ok(app_dir) = app_handle.path().app_local_data_dir() {
                let logo_file = app_dir.join("musique_logo.png");
                if !logo_file.exists() {
                    let _ = std::fs::create_dir_all(&app_dir);
                    let _ = std::fs::write(&logo_file, include_bytes!("../../icons/icon.png"));
                }
                if logo_file.exists() {
                    toast = toast.icon(&logo_file, IconCrop::Square, "Musique");
                }
            }

            if let Err(e) = toast.show() {
                log::warn!("[notification] failed to show windows toast: {e}");
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let app_handle = _app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let mut notification = notify_rust::Notification::new();
            notification.summary(&title);
            notification.body(&body);

            #[cfg(target_os = "macos")]
            {
                let _ = notify_rust::set_application("dev.boyblah.musique");
            }
            #[cfg(target_os = "linux")]
            {
                notification.app_id("dev.boyblah.musique");
            }

            if let Some(ref p) = local_icon {
                notification.icon(p);
            } else if let Ok(app_dir) = app_handle.path().app_local_data_dir() {
                let logo_file = app_dir.join("musique_logo.png");
                if logo_file.exists() {
                    notification.icon(&logo_file.to_string_lossy());
                } else {
                    notification.auto_icon();
                }
            } else {
                notification.auto_icon();
            }

            if let Err(e) = notification.show() {
                log::warn!("[notification] failed to show notification: {e}");
            }
        });
    }

    Ok(())
}
