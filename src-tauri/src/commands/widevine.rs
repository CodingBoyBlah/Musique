//! Installing and removing Google's Widevine module (see `crate::widevine`), which Apple Music
//! playback needs. Nothing is downloaded until the user has seen Google's terms
//! (`widevine_license_text`) and asked for the install.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::ipc::Channel;

use crate::errors::AppError;
use crate::widevine;

#[derive(Debug, Clone, Serialize)]
pub struct WidevineStatus {
    /// A module is installed and ready to load.
    pub installed: bool,
    /// The installed module's version.
    pub version: Option<String>,
    /// Where the installed module is.
    pub path: Option<String>,
    /// Google builds a module for this computer.
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WidevineProgress {
    /// Bytes downloaded so far.
    pub done: u64,
    /// Bytes to download, when known.
    pub total: Option<u64>,
}

fn current() -> WidevineStatus {
    let status = widevine::status();
    WidevineStatus {
        installed: status.installed,
        version: status.version,
        path: status.path.map(|p| p.display().to_string()),
        supported: status.supported,
    }
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| AppError::Playback(format!("widevine task failed: {e}")))
}

#[tauri::command]
pub async fn widevine_status() -> WidevineStatus {
    blocking(current).await.unwrap_or(WidevineStatus {
        installed: false,
        version: None,
        path: None,
        supported: widevine::supported(),
    })
}

/// Google's terms for the module, to show before installing. When nothing is installed this
/// downloads the current release to read them (and keeps it for the install that follows).
#[tauri::command]
pub async fn widevine_license_text() -> Result<String, AppError> {
    widevine::license_text().await
}

#[tauri::command]
pub async fn widevine_install(
    on_progress: Channel<WidevineProgress>,
) -> Result<WidevineStatus, AppError> {
    // About a hundred updates over the download, not one per network chunk.
    let sent = Arc::new(AtomicU64::new(u64::MAX));
    widevine::install(move |done, total| {
        let step = match total {
            Some(t) if t > 0 => done.saturating_mul(100) / t,
            _ => done >> 18,
        };
        if sent.swap(step, Ordering::Relaxed) != step {
            let _ = on_progress.send(WidevineProgress { done, total });
        }
    })
    .await?;
    blocking(current).await
}

#[tauri::command]
pub async fn widevine_uninstall() -> Result<(), AppError> {
    blocking(widevine::uninstall).await?
}
