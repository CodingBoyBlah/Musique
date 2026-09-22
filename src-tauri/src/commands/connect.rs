use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    spotify::types::{AlbumItem, ArtistItem, SpAlbumSimple, SpArtistSimple, SpTrack, TrackItem},
    state::AppState,
};

const BASE: &str = "https://api.spotify.com/v1";

fn client() -> &'static reqwest::Client {
    crate::http::client()
}

async fn tok(app: &AppHandle) -> Result<String, AppError> {
    let s = app.state::<AppState>();
    let db = s.db.clone();
    let auth = s.auth.clone();
    crate::auth::get_valid_token(&db, &auth).await
}

async fn musique_device_id(app: &AppHandle) -> String {
    let db = app.state::<AppState>().db.clone();
    let device_name = crate::auth::get_setting_value(&db, "device_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::DEFAULT_DEVICE_NAME.to_string());
    crate::auth::compute_device_id(&device_name)
}

fn item_from_simple(a: &SpArtistSimple) -> ArtistItem {
    ArtistItem {
        id: a.id.clone(),
        name: a.name.clone(),
        image_url: None,
        popularity: None,
    }
}

fn item_from_album_simple(al: &SpAlbumSimple) -> AlbumItem {
    AlbumItem {
        id: al.id.clone(),
        name: al.name.clone(),
        album_type: al.album_type.clone(),
        image_url: al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        artists: al
            .artists
            .as_ref()
            .map(|v| v.iter().map(item_from_simple).collect())
            .unwrap_or_default(),
        popularity: None,
    }
}

fn item_from_track(t: &SpTrack) -> TrackItem {
    TrackItem {
        id: t.id.clone(),
        name: t.name.clone(),
        duration_ms: t.duration_ms,
        explicit: t.explicit,
        artists: t.artists.iter().map(item_from_simple).collect(),
        album: t.album.as_ref().map(item_from_album_simple),
        popularity: t.popularity,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectDevice {
    pub id: Option<String>,
    pub is_active: bool,
    #[serde(default)]
    pub is_private_session: bool,
    #[serde(default)]
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub volume_percent: Option<u32>,
    #[serde(default)]
    pub supports_volume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesPayload {
    pub devices: Vec<ConnectDevice>,
    pub musique_device_id: String,
}

#[derive(Debug, Deserialize)]
struct RawDevicesResponse {
    devices: Vec<ConnectDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePlaybackState {
    pub device: ConnectDevice,
    pub is_playing: bool,
    pub progress_ms: Option<u64>,
    pub timestamp: Option<u64>,
    pub shuffle_state: bool,
    pub repeat_state: String,
    pub track: Option<TrackItem>,
}

#[derive(Deserialize)]
struct RawPlaybackResponse {
    device: Option<ConnectDevice>,
    is_playing: Option<bool>,
    progress_ms: Option<u64>,
    timestamp: Option<u64>,
    shuffle_state: Option<bool>,
    repeat_state: Option<String>,
    item: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_devices(app: AppHandle) -> Result<DevicesPayload, AppError> {
    let token = tok(&app).await?;
    let db = app.state::<AppState>().db.clone();
    let device_name = crate::auth::get_setting_value(&db, "device_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::DEFAULT_DEVICE_NAME.to_string());

    let url = format!("{BASE}/me/player/devices");
    let raw: RawDevicesResponse = crate::spotify::spotify_get(&token, &url).await?;

    let matched_id = raw
        .devices
        .iter()
        .find(|d| d.name.eq_ignore_ascii_case(&device_name))
        .and_then(|d| d.id.clone());

    let musique_id = matched_id.unwrap_or_else(|| crate::auth::compute_device_id(&device_name));

    Ok(DevicesPayload {
        devices: raw.devices,
        musique_device_id: musique_id,
    })
}

#[tauri::command]
pub async fn get_playback_state(app: AppHandle) -> Result<Option<RemotePlaybackState>, AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player");

    let resp = client()
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    let status = resp.status();
    if status.as_u16() == 204 {
        return Ok(None);
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!("Spotify {status}: {body}")));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    if text.trim().is_empty() {
        return Ok(None);
    }

    let raw: RawPlaybackResponse = serde_json::from_str(&text)
        .map_err(|e| AppError::Network(format!("decode playback state: {e}")))?;

    let device = match raw.device {
        Some(d) => d,
        None => return Ok(None),
    };

    let track = raw.item.and_then(|val| {
        serde_json::from_value::<SpTrack>(val).ok().map(|t| item_from_track(&t))
    });

    Ok(Some(RemotePlaybackState {
        device,
        is_playing: raw.is_playing.unwrap_or(false),
        progress_ms: raw.progress_ms,
        timestamp: raw.timestamp,
        shuffle_state: raw.shuffle_state.unwrap_or(false),
        repeat_state: raw.repeat_state.unwrap_or_else(|| "off".to_string()),
        track,
    }))
}

#[tauri::command]
pub async fn transfer_playback(
    app: AppHandle,
    device_id: String,
    play: bool,
) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player");

    let body = serde_json::json!({
        "device_ids": [device_id],
        "play": play
    });

    let _ = crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::PUT,
        &url,
        body,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn remote_play_track(
    app: AppHandle,
    device_id: String,
    track_id: Option<String>,
    position_ms: Option<u32>,
) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/play?device_id={device_id}");

    let body = match (&track_id, position_ms) {
        (Some(tid), Some(pos)) => {
            let uri = if tid.starts_with("spotify:track:") {
                tid.clone()
            } else {
                format!("spotify:track:{tid}")
            };
            serde_json::json!({
                "uris": [uri],
                "position_ms": pos
            })
        }
        (Some(tid), None) => {
            let uri = if tid.starts_with("spotify:track:") {
                tid.clone()
            } else {
                format!("spotify:track:{tid}")
            };
            serde_json::json!({ "uris": [uri] })
        }
        (None, Some(pos)) => {
            serde_json::json!({ "position_ms": pos })
        }
        (None, None) => serde_json::json!({}),
    };

    let client = client();
    let resp = client
        .put(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    // If PUT /play fails on target device (e.g. device idle or asleep), wake via transfer and retry
    if !resp.status().is_success() {
        let transfer_url = format!("{BASE}/me/player");
        let transfer_body = serde_json::json!({
            "device_ids": [&device_id],
            "play": true
        });
        let _ = client
            .put(&transfer_url)
            .bearer_auth(&token)
            .json(&transfer_body)
            .send()
            .await;

        if track_id.is_some() || position_ms.is_some() {
            tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;
            let _ = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn remote_play(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/play");
    if let Some(id) = device_id {
        url = format!("{url}?device_id={id}");
    }
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url).await
}

#[tauri::command]
pub async fn remote_pause(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/pause");
    if let Some(id) = device_id {
        url = format!("{url}?device_id={id}");
    }
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url).await
}

#[tauri::command]
pub async fn remote_next(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/next");
    if let Some(id) = device_id {
        url = format!("{url}?device_id={id}");
    }
    crate::spotify::spotify_write(&token, reqwest::Method::POST, &url).await
}

#[tauri::command]
pub async fn remote_previous(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/previous");
    if let Some(id) = device_id {
        url = format!("{url}?device_id={id}");
    }
    crate::spotify::spotify_write(&token, reqwest::Method::POST, &url).await
}

#[tauri::command]
pub async fn remote_seek(app: AppHandle, position_ms: u32) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/seek?position_ms={position_ms}");
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url).await
}

#[tauri::command]
pub async fn remote_set_volume(app: AppHandle, volume_percent: u8) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/volume?volume_percent={volume_percent}");
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url).await
}

#[tauri::command]
pub async fn get_musique_device_id(app: AppHandle) -> Result<String, AppError> {
    Ok(musique_device_id(&app).await)
}
