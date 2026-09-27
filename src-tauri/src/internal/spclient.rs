//! spclient access over the live librespot session.
//!
//! the session carries spotify's first-party client identity, which is what
//! these endpoints are gated on (see the global notes on client ids - never
//! swap the session's id for the user's own, it 400s at clienttoken).

use librespot_core::{error::ErrorKind, Session};
use reqwest::{header::HeaderMap, Method};
use tauri::{AppHandle, Manager};

use crate::{errors::AppError, state::AppState};

/// librespot error -> ours. NotFound stays NotFound so callers can tell "this
/// entity has no credits/canvas/..." apart from the endpoint actually failing.
pub fn map_err(e: librespot_core::Error) -> AppError {
    match e.kind {
        ErrorKind::NotFound => AppError::NotFound(e.to_string()),
        ErrorKind::Unauthenticated | ErrorKind::PermissionDenied => AppError::Auth(e.to_string()),
        _ => AppError::Network(e.to_string()),
    }
}

async fn current(app: &AppHandle) -> Result<Option<Session>, AppError> {
    let playback = app
        .try_state::<AppState>()
        .ok_or_else(|| AppError::Playback("app state not ready".into()))?
        .playback
        .clone();
    let guard = playback.lock().await;
    // clone the handle out and drop the guard before any network i/o - holding
    // the playback mutex across a request would stall every transport control
    Ok(guard
        .as_ref()
        .filter(|inner| !inner.session_invalid())
        .map(|inner| inner.session()))
}

/// the live session, warming one up (non-interactively) when there isn't one.
pub async fn session(app: &AppHandle) -> Result<Session, AppError> {
    if let Some(s) = current(app).await? {
        return Ok(s);
    }
    crate::commands::playback::warm_session(app).await?;
    current(app)
        .await?
        .ok_or_else(|| AppError::Playback("spotify session unavailable".into()))
}

/// GET an spclient endpoint that speaks json
pub async fn get_json_value(app: &AppHandle, endpoint: &str) -> Result<serde_json::Value, AppError> {
    let bytes = get_json_bytes(app, endpoint).await?;
    serde_json::from_slice(&bytes).map_err(|e| AppError::Network(format!("decode {endpoint}: {e}")))
}

pub async fn get_json<T: serde::de::DeserializeOwned>(app: &AppHandle, endpoint: &str) -> Result<T, AppError> {
    let bytes = get_json_bytes(app, endpoint).await?;
    serde_json::from_slice(&bytes).map_err(|e| AppError::Network(format!("decode {endpoint}: {e}")))
}

pub async fn get_json_bytes(app: &AppHandle, endpoint: &str) -> Result<bytes::Bytes, AppError> {
    let s = session(app).await?;
    s.spclient()
        .request_as_json(&Method::GET, endpoint, None, None)
        .await
        .map_err(map_err)
}

/// send json to an spclient endpoint and read json back
pub async fn send_json(
    app: &AppHandle,
    method: Method,
    endpoint: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let s = session(app).await?;
    let body = body.map(|b| b.to_string());
    let mut headers = HeaderMap::new();
    if body.is_some() {
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse().unwrap());
    }
    let bytes = s
        .spclient()
        .request_as_json(&method, endpoint, Some(headers), body.as_deref())
        .await
        .map_err(map_err)?;
    if bytes.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_slice(&bytes).map_err(|e| AppError::Network(format!("decode {endpoint}: {e}")))
}

/// raw GET (protobuf endpoints like the rootlist / playlist v2)
pub async fn get_raw(app: &AppHandle, endpoint: &str) -> Result<bytes::Bytes, AppError> {
    let s = session(app).await?;
    s.spclient()
        .request(&Method::GET, endpoint, None, None)
        .await
        .map_err(map_err)
}

/// POST a protobuf message, get the raw response bytes back
pub async fn post_protobuf<M>(app: &AppHandle, endpoint: &str, msg: &M) -> Result<bytes::Bytes, AppError>
where
    M: protobuf::Message + protobuf::MessageFull,
{
    let s = session(app).await?;
    s.spclient()
        .request_with_protobuf(&Method::POST, endpoint, None, msg)
        .await
        .map_err(map_err)
}

/// the signed-in user's canonical spotify username (what spclient paths want)
pub async fn username(app: &AppHandle) -> Result<String, AppError> {
    Ok(session(app).await?.username())
}

/// a first-party bearer token (login5) plus the matching client-token. this is
/// the pair the official client sends to api-partner / pathfinder
pub async fn first_party_tokens(app: &AppHandle) -> Result<(String, String), AppError> {
    let s = session(app).await?;
    let token = s.login5().auth_token().await.map_err(map_err)?;
    let client_token = s.spclient().client_token().await.map_err(map_err)?;
    Ok((token.access_token, client_token))
}

/// spotify's image cdn url for a raw image file id
pub fn image_url(file_id: &[u8]) -> Option<String> {
    if file_id.is_empty() {
        return None;
    }
    let hex: String = file_id.iter().map(|b| format!("{b:02x}")).collect();
    Some(format!("https://i.scdn.co/image/{hex}"))
}

/// `spotify:image:<hex>` / bare hex / full url -> https url
pub fn image_uri_to_url(uri: &str) -> Option<String> {
    if uri.starts_with("http") {
        return Some(uri.to_string());
    }
    let id = uri.strip_prefix("spotify:image:").unwrap_or(uri);
    if id.is_empty() {
        return None;
    }
    Some(format!("https://i.scdn.co/image/{id}"))
}

/// last path segment of a spotify uri ("spotify:track:abc" -> "abc")
pub fn uri_id(uri: &str) -> &str {
    uri.rsplit(':').next().unwrap_or(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_helpers() {
        assert_eq!(
            image_url(&[0xab, 0x67, 0x01]).as_deref(),
            Some("https://i.scdn.co/image/ab6701")
        );
        assert_eq!(image_url(&[]), None);
        assert_eq!(
            image_uri_to_url("spotify:image:ab67").as_deref(),
            Some("https://i.scdn.co/image/ab67")
        );
        assert_eq!(uri_id("spotify:track:abc"), "abc");
        assert_eq!(uri_id("abc"), "abc");
    }
}
