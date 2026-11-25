use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    commands::receivers::LAN,
    errors::AppError,
    sonos::{PlayRequest, RemoteAction},
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

/// The name this computer's Soloist appears under in Spotify's device list.
fn soloist_name(app: &AppHandle) -> Option<String> {
    app.state::<AppState>().soloist.device_name(app)
}

/// Soloist is how this computer plays when it is the audio source, not
/// another device: it is folded into this computer's own row, which takes
/// its active state, so it never shows as somewhere else to play.
fn fold_soloist(devices: &mut Vec<ConnectDevice>, soloist: Option<&str>, musique_id: &str, device_name: &str) {
    let Some(soloist) = soloist else { return };
    let Some(at) = devices.iter().position(|d| d.name == soloist) else { return };
    let row = devices.remove(at);
    // a stale second listing of it goes too
    devices.retain(|d| d.name != soloist);
    if !row.is_active {
        return;
    }
    match devices.iter_mut().find(|d| d.id.as_deref() == Some(musique_id)) {
        Some(own) => {
            own.is_active = true;
            own.volume_percent = row.volume_percent.or(own.volume_percent);
        }
        None => devices.insert(0, ConnectDevice {
            id: Some(musique_id.to_string()),
            name: device_name.to_string(),
            ..row
        }),
    }
}

/// Sends a player command to a restricted device (a Sonos, mostly) over
/// Spotify Connect, the only route Spotify lets those take.
async fn connect_route(
    app: &AppHandle,
    to: &str,
    action: RemoteAction,
    play: Option<&PlayRequest>,
    position_ms: u32,
    flag: bool,
    repeat: &str,
) -> Result<(), AppError> {
    let session = crate::internal::spclient::session(app).await?;
    let body = crate::sonos::command_body(action, play, position_ms, flag, repeat);
    crate::sonos::command(&session, to, &body).await
}

/// A remote command goes over the Web API, except to a device Spotify keeps
/// to Spotify Connect: one it reported as restricted, a Sonos found on the
/// network, or one the Web API has just refused with "Restriction violated".
#[allow(clippy::too_many_arguments)]
async fn steer<F, Fut>(
    app: &AppHandle,
    device: Option<&str>,
    action: RemoteAction,
    play: Option<PlayRequest>,
    position_ms: u32,
    flag: bool,
    repeat: &str,
    web: F,
) -> Result<(), AppError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    if let Some(to) = LAN.connect.command_target(device, None) {
        return connect_route(app, &to, action, play.as_ref(), position_ms, flag, repeat).await;
    }
    match web().await {
        Err(e) if crate::sonos::is_restriction(&e) => {
            let Some(to) = device else { return Err(e) };
            LAN.connect.mark_restricted(to);
            connect_route(app, to, action, play.as_ref(), position_ms, flag, repeat).await
        }
        other => other,
    }
}

/// Numbers each playback poll, so an older answer never overrides a newer one.
static PLAYBACK_POLL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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
    let soloist = soloist_name(&app);
    let mut raw = raw;
    fold_soloist(&mut raw.devices, soloist.as_deref(), &musique_id, &device_name);

    // speakers on the network that are not signed in yet (or are an idle
    // sonos) are offered too; moving playback to one signs it in
    LAN.connect.observe_devices(&raw.devices);
    crate::commands::receivers::scan_in_background(&app);
    let mut devices = raw.devices;
    let waiting = crate::commands::receivers::waiting_rows(&devices);
    devices.extend(waiting);
    // Soloist also answers the speaker scan on the network, as a speaker
    // waiting to be signed in; it is this computer all the same
    if let Some(soloist) = soloist.as_deref() {
        devices.retain(|d| d.name != soloist);
    }

    Ok(DevicesPayload {
        devices,
        musique_device_id: musique_id,
    })
}

#[tauri::command]
pub async fn get_playback_state(app: AppHandle) -> Result<Option<RemotePlaybackState>, AppError> {
    let seq = PLAYBACK_POLL.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    LAN.connect.poll_sent(seq, None);
    let state = playback_state(&app).await;
    match &state {
        Ok(state) => LAN.connect.observe_playing(seq, state.as_ref().map(|state| &state.device), None),
        Err(_) => LAN.connect.poll_failed(seq),
    }
    state
}

async fn playback_state(app: &AppHandle) -> Result<Option<RemotePlaybackState>, AppError> {
    let token = tok(app).await?;
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

    let mut device = match raw.device {
        Some(d) => d,
        None => return Ok(None),
    };
    // Soloist playing is this computer playing (see `fold_soloist`)
    if soloist_name(app).is_some_and(|name| name == device.name) {
        let db = app.state::<AppState>().db.clone();
        let device_name = crate::auth::get_setting_value(&db, "device_name")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| crate::auth::DEFAULT_DEVICE_NAME.to_string());
        device.id = Some(musique_device_id(app).await);
        device.name = device_name;
    }

    let track = raw.item.and_then(item_from_value);

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
    if LAN.connect.restricted_target(Some(&device_id)).is_some() {
        return connect_transfer(&app, &device_id, play).await;
    }
    match web_transfer(&app, &device_id, play).await {
        Ok(()) => Ok(()),
        Err(e) if crate::sonos::is_restriction(&e) => {
            LAN.connect.mark_restricted(&device_id);
            connect_transfer(&app, &device_id, play).await
        }
        // a speaker found on the network but not signed in yet: sign it in,
        // wait for spotify to list it, then move playback to it
        Err(e) if crate::commands::receivers::knows(&device_id) => {
            eprintln!("[connect] signing in a waiting speaker after: {e}");
            crate::commands::receivers::sign_in_and_wait(&app, &device_id).await?;
            if LAN.connect.restricted_target(Some(&device_id)).is_some() {
                return connect_transfer(&app, &device_id, play).await;
            }
            web_transfer(&app, &device_id, play).await
        }
        Err(e) => Err(e),
    }
}

async fn web_transfer(app: &AppHandle, device_id: &str, play: bool) -> Result<(), AppError> {
    let token = tok(app).await?;
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

/// A transfer over Spotify Connect, for a device the Web API may not move to.
async fn connect_transfer(app: &AppHandle, device_id: &str, play: bool) -> Result<(), AppError> {
    let session = crate::internal::spclient::session(app).await?;
    LAN.connect.transfer(&session, device_id).await?;
    if !play {
        // a connect transfer carries on playing; the web api's play=false
        // means arrive paused
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        connect_route(app, device_id, RemoteAction::Pause, None, 0, false, "").await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_play_track(
    app: AppHandle,
    device_id: String,
    track_id: Option<String>,
    position_ms: Option<u32>,
) -> Result<(), AppError> {
    let play = match &track_id {
        Some(tid) => {
            let mut request = PlayRequest::context(&crate::playback::track_uri(tid)?);
            request.position_ms = position_ms.unwrap_or(0);
            Some(request)
        }
        None => None,
    };
    let (web_app, web_device) = (app.clone(), device_id.clone());
    steer(&app, Some(&device_id), RemoteAction::Play, play, 0, false, "", move || {
        web_play_track(web_app, web_device, track_id, position_ms)
    })
    .await
}

async fn web_play_track(
    app: AppHandle,
    device_id: String,
    track_id: Option<String>,
    position_ms: Option<u32>,
) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/play?device_id={device_id}");

    let mut body = serde_json::json!({});
    if let Some(tid) = &track_id {
        body["uris"] = serde_json::json!([crate::playback::track_uri(tid)?]);
    }
    if let Some(pos) = position_ms {
        body["position_ms"] = serde_json::json!(pos);
    }

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
        let transfer = client
            .put(&transfer_url)
            .bearer_auth(&token)
            .json(&transfer_body)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;
        if !transfer.status().is_success() {
            return Err(AppError::Playback(format!("playback transfer failed: {}", transfer.status())));
        }

        if track_id.is_some() || position_ms.is_some() {
            tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;
            let retry = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Network(e.to_string()))?;
            if !retry.status().is_success() {
                return Err(AppError::Playback(format!("remote playback failed: {}", retry.status())));
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn remote_play(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/play");
    if let Some(id) = &device_id {
        url = format!("{url}?device_id={id}");
    }
    steer(&app, device_id.as_deref(), RemoteAction::Play, None, 0, false, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url)
    })
    .await
}

#[tauri::command]
pub async fn remote_pause(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/pause");
    if let Some(id) = &device_id {
        url = format!("{url}?device_id={id}");
    }
    steer(&app, device_id.as_deref(), RemoteAction::Pause, None, 0, false, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url)
    })
    .await
}

#[tauri::command]
pub async fn remote_next(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/next");
    if let Some(id) = &device_id {
        url = format!("{url}?device_id={id}");
    }
    steer(&app, device_id.as_deref(), RemoteAction::Next, None, 0, false, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::POST, &url)
    })
    .await
}

#[tauri::command]
pub async fn remote_previous(app: AppHandle, device_id: Option<String>) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = format!("{BASE}/me/player/previous");
    if let Some(id) = &device_id {
        url = format!("{url}?device_id={id}");
    }
    steer(&app, device_id.as_deref(), RemoteAction::Previous, None, 0, false, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::POST, &url)
    })
    .await
}

#[tauri::command]
pub async fn remote_seek(app: AppHandle, position_ms: u32) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/seek?position_ms={position_ms}");
    steer(&app, None, RemoteAction::Seek, None, position_ms, false, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url)
    })
    .await
}

#[tauri::command]
pub async fn remote_set_volume(app: AppHandle, volume_percent: u8) -> Result<(), AppError> {
    if let Some(to) = LAN.connect.command_target(None, None) {
        let session = crate::internal::spclient::session(&app).await?;
        return crate::sonos::set_volume(&session, &to, volume_percent).await;
    }
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/volume?volume_percent={volume_percent}");
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url).await
}

#[tauri::command]
pub async fn get_musique_device_id(app: AppHandle) -> Result<String, AppError> {
    Ok(musique_device_id(&app).await)
}

// ── remote queue / shuffle / repeat / context playback ──────────────────────

/// a queue/playback item is a track OR a podcast episode. episodes come back
/// with the show where a track has its album, and keep their full uri as the id
/// so the player can tell the two apart.
pub(crate) fn item_from_value(val: serde_json::Value) -> Option<TrackItem> {
    let kind = val.get("type").and_then(|t| t.as_str()).unwrap_or("track");
    if kind == "episode" {
        let show = val.get("show");
        let image = val
            .get("images")
            .or_else(|| show.and_then(|s| s.get("images")))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|i| i.get("url"))
            .and_then(|u| u.as_str())
            .map(str::to_string);
        let show_id = show.and_then(|s| s.get("id")).and_then(|v| v.as_str()).unwrap_or_default();
        let show_name = show.and_then(|s| s.get("name")).and_then(|v| v.as_str()).unwrap_or_default();
        let publisher = show.and_then(|s| s.get("publisher")).and_then(|v| v.as_str()).unwrap_or(show_name);
        return Some(TrackItem {
            id: format!("spotify:episode:{}", val.get("id")?.as_str()?),
            name: val.get("name")?.as_str()?.to_string(),
            duration_ms: val.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0),
            explicit: val.get("explicit").and_then(|v| v.as_bool()).unwrap_or(false),
            artists: vec![ArtistItem { id: show_id.to_string(), name: publisher.to_string(), image_url: None, popularity: None }],
            album: Some(AlbumItem {
                id: show_id.to_string(),
                name: show_name.to_string(),
                album_type: "show".into(),
                image_url: image,
                release_date: val.get("release_date").and_then(|v| v.as_str()).map(str::to_string),
                artists: Vec::new(),
                popularity: None,
            }),
            popularity: None,
        });
    }
    serde_json::from_value::<SpTrack>(val).ok().map(|t| item_from_track(&t))
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteQueue {
    pub currently_playing: Option<TrackItem>,
    pub queue: Vec<TrackItem>,
}

#[derive(Deserialize)]
struct RawQueue {
    currently_playing: Option<serde_json::Value>,
    #[serde(default)]
    queue: Vec<serde_json::Value>,
}

/// what spotify itself has queued on the active device. only meaningful while
/// another device is playing - musique keeps its own queue locally.
#[tauri::command]
pub async fn get_remote_queue(app: AppHandle) -> Result<RemoteQueue, AppError> {
    let token = tok(&app).await?;
    let raw: RawQueue = crate::spotify::spotify_get(&token, &format!("{BASE}/me/player/queue")).await?;
    Ok(RemoteQueue {
        currently_playing: raw.currently_playing.and_then(item_from_value),
        queue: raw.queue.into_iter().filter_map(item_from_value).collect(),
    })
}

#[tauri::command]
pub async fn remote_set_shuffle(app: AppHandle, state: bool) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/shuffle?state={state}");
    steer(&app, None, RemoteAction::Shuffle, None, 0, state, "", || {
        crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url)
    })
    .await
}

/// `state` is spotify's own vocabulary: "off" | "context" | "track"
#[tauri::command]
pub async fn remote_set_repeat(app: AppHandle, state: String) -> Result<(), AppError> {
    if !matches!(state.as_str(), "off" | "context" | "track") {
        return Err(AppError::InvalidInput(format!("bad repeat state: {state}")));
    }
    let token = tok(&app).await?;
    let url = format!("{BASE}/me/player/repeat?state={state}");
    steer(&app, None, RemoteAction::Repeat, None, 0, false, &state, || {
        crate::spotify::spotify_write(&token, reqwest::Method::PUT, &url)
    })
    .await
}

fn playable_uri(id: &str) -> String {
    if id.starts_with("spotify:") {
        id.to_string()
    } else {
        format!("spotify:track:{id}")
    }
}

#[tauri::command]
pub async fn remote_add_to_queue(app: AppHandle, id: String) -> Result<(), AppError> {
    if let Some(to) = LAN.connect.command_target(None, None) {
        let session = crate::internal::spclient::session(&app).await?;
        return LAN.connect.add_to_queue(&session, &to, &[playable_uri(&id)]).await.1;
    }
    let token = tok(&app).await?;
    let mut url = url::Url::parse(&format!("{BASE}/me/player/queue")).unwrap();
    url.query_pairs_mut().append_pair("uri", &playable_uri(&id));
    crate::spotify::spotify_write(&token, reqwest::Method::POST, url.as_str()).await
}

/// start a whole context (album / playlist / artist / show) on a remote device,
/// optionally from a given track in it - so next/prev on the phone walk the
/// real context instead of a one-track queue
#[tauri::command]
pub async fn remote_play_context(
    app: AppHandle,
    device_id: Option<String>,
    context_uri: String,
    offset_uri: Option<String>,
    offset_position: Option<u32>,
    position_ms: Option<u32>,
) -> Result<(), AppError> {
    let token = tok(&app).await?;
    let mut url = url::Url::parse(&format!("{BASE}/me/player/play")).unwrap();
    if let Some(id) = &device_id {
        url.query_pairs_mut().append_pair("device_id", id);
    }
    let mut body = serde_json::json!({ "context_uri": context_uri });
    let mut request = PlayRequest::context(&context_uri);
    if let Some(u) = offset_uri {
        body["offset"] = serde_json::json!({ "uri": playable_uri(&u) });
        request.offset_uri = Some(playable_uri(&u));
    } else if let Some(p) = offset_position {
        body["offset"] = serde_json::json!({ "position": p });
        request.offset_position = Some(p);
    }
    if let Some(p) = position_ms {
        body["position_ms"] = serde_json::json!(p);
        request.position_ms = p;
    }
    steer(&app, device_id.as_deref(), RemoteAction::Play, Some(request), 0, false, "", || async {
        crate::spotify::spotify_write_json(&token, reqwest::Method::PUT, url.as_str(), body).await.map(drop)
    })
    .await
}

#[cfg(test)]
mod soloist_rows {
    use super::*;

    fn row(id: &str, name: &str, active: bool) -> ConnectDevice {
        ConnectDevice {
            id: Some(id.into()),
            is_active: active,
            is_private_session: false,
            is_restricted: false,
            name: name.into(),
            device_type: "Computer".into(),
            volume_percent: Some(40),
            supports_volume: true,
        }
    }

    const SOLOIST: &str = "Musique Soloist (8071de28)";

    #[test]
    fn soloist_playing_shows_as_this_computer_and_never_as_another_device() {
        let mut devices = vec![row("m", "Musique", false), row("s", SOLOIST, true), row("p", "Phone", false)];
        fold_soloist(&mut devices, Some(SOLOIST), "m", "Musique");
        let names: Vec<_> = devices.iter().map(|d| (d.name.as_str(), d.is_active)).collect();
        assert_eq!(names, [("Musique", true), ("Phone", false)]);
    }

    #[test]
    fn an_idle_soloist_is_just_left_out() {
        let mut devices = vec![row("m", "Musique", false), row("s", SOLOIST, false)];
        fold_soloist(&mut devices, Some(SOLOIST), "m", "Musique");
        assert_eq!(devices.len(), 1);
        assert!(!devices[0].is_active);
    }

    #[test]
    fn without_the_built_in_device_listed_soloist_stands_in_for_it() {
        let mut devices = vec![row("p", "Phone", false), row("s", SOLOIST, true)];
        fold_soloist(&mut devices, Some(SOLOIST), "m", "Musique");
        assert_eq!(devices[0].id.as_deref(), Some("m"));
        assert_eq!(devices[0].name, "Musique");
        assert!(devices[0].is_active);
    }

    #[test]
    fn another_computers_soloist_is_another_device() {
        let mut devices = vec![row("m", "Musique", false), row("o", "Musique Soloist (deadbeef)", true)];
        fold_soloist(&mut devices, Some(SOLOIST), "m", "Musique");
        assert_eq!(devices.len(), 2);
    }
}
