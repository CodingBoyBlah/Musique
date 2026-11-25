//! Widevine license acquisition and authenticated Apple CDN asset requests.
//!
//! Never logs user tokens, bearer tokens, Widevine challenges, license blobs, or keys.

use std::sync::OnceLock;
use std::time::Duration;

use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
    Engine as _,
};
use serde_json::Value;

use crate::errors::AppError;
use crate::widevine::{self, Cdm};

use super::resolve::{apply_apple_headers, send_reconnecting, KeyInfo, SAFARI_USER_AGENT};

const LICENSE_URL: &str =
    "https://play.itunes.apple.com/WebObjects/MZPlay.woa/wa/acquireWebPlaybackLicense";

/// Dedicated HTTP client for streaming fMP4 media bodies without `crate::http::client()`'s
/// 20-second total request timeout, using a read timeout instead so long tracks don't drop.
static MEDIA_HTTP: OnceLock<reqwest::Client> = OnceLock::new();

pub(crate) fn media_http() -> &'static reqwest::Client {
    MEDIA_HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_secs(300))
            .pool_max_idle_per_host(4)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    })
}

/// Acquires a Widevine license from Apple for `key` and installs it into a `Cdm` handle.
///
/// Runs `Cdm::exchange` on a blocking thread so `widevine`'s internal licensing mutex serializes
/// challenge generation, license POST, and license acceptance on the same `Cdm` handle.
pub async fn acquire_license(
    http: &reqwest::Client,
    bearer: &str,
    user_token: &str,
    key: &KeyInfo,
    is_library: bool,
) -> Result<Cdm, AppError> {
    if !widevine::available() {
        return Err(AppError::Playback(
            "Widevine CDM is not installed; cannot play protected Apple Music tracks".into(),
        ));
    }

    let init_data = widevine::pssh(&key.id);
    let handle = tokio::runtime::Handle::current();
    let http = http.clone();
    let bearer = bearer.to_owned();
    let user_token = user_token.to_owned();
    let uri = key.uri.clone();
    let song_id = key.song_id.clone();

    tokio::task::spawn_blocking(move || -> Result<Cdm, AppError> {
        let cdm = Cdm::open()?;
        cdm.exchange(&init_data, |challenge| {
            handle.block_on(post_license(
                &http, &bearer, &user_token, &uri, &song_id, is_library, &challenge,
            ))
        })?;
        Ok(cdm)
    })
    .await
    .map_err(|e| AppError::Playback(format!("widevine license exchange task failed: {e}")))?
}

async fn post_license(
    http: &reqwest::Client,
    bearer: &str,
    user_token: &str,
    uri: &str,
    song_id: &str,
    is_library: bool,
    challenge: &[u8],
) -> Result<Vec<u8>, AppError> {
    let payload = serde_json::json!({
        "challenge": STANDARD.encode(challenge),
        "key-system": "com.widevine.alpha",
        "uri": uri,
        "adamId": song_id,
        "isLibrary": is_library,
        "user-initiated": true,
    });

    let req = apply_apple_headers(http.post(LICENSE_URL), bearer, user_token).json(&payload);
    let resp = send_reconnecting(req, "acquireWebPlaybackLicense").await?;
    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| {
        AppError::Playback(format!(
            "cannot parse Widevine license response (http {status}): {e}"
        ))
    })?;

    if !status.is_success() {
        return Err(AppError::Playback(format!(
            "apple refused Widevine license: http {status}"
        )));
    }
    if let Some(code) = json
        .get("errorCode")
        .and_then(Value::as_i64)
        .filter(|&c| c != 0)
    {
        return Err(AppError::Playback(format!(
            "apple refused Widevine license: errorCode {code}"
        )));
    }

    let encoded_license = json
        .get("license")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Playback("apple license response carried no license".into()))?;
    STANDARD
        .decode(encoded_license)
        .or_else(|_| STANDARD_NO_PAD.decode(encoded_license))
        .map_err(|e| AppError::Playback(format!("apple license is not valid base64: {e}")))
}

/// Opens the streaming HTTP GET for an Apple Music fMP4 or plain upload asset.
pub async fn open_asset(url: &str, user_token: &str) -> Result<reqwest::Response, AppError> {
    let req = media_http()
        .get(url)
        .header(reqwest::header::USER_AGENT, SAFARI_USER_AGENT)
        .header("x-apple-music-user-token", user_token);
    let resp = send_reconnecting(req, "media asset").await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::Playback(format!(
            "apple media asset request failed: http {status}"
        )));
    }
    Ok(resp)
}
