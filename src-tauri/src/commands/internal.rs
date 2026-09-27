//! ipc surface for the internal (spclient / pathfinder) data layer. the heavy
//! lifting lives in `crate::internal`; these just validate and forward.

use tauri::AppHandle;

use crate::{errors::AppError, spotify::types::TrackItem};

/// hydrate bare track ids into full rows via extended-metadata. order is kept,
/// unknown ids are dropped.
#[tauri::command]
pub async fn get_tracks_metadata(app: AppHandle, ids: Vec<String>) -> Result<Vec<TrackItem>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    crate::internal::metadata::tracks(&app, &ids).await
}
