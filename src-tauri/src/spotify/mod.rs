pub mod types;

use crate::errors::AppError;

pub(crate) async fn spotify_get<T: serde::de::DeserializeOwned>(
    token: &str,
    url:   &str,
) -> Result<T, AppError> {
    let resp = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body   = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!("Spotify {status}: {body}")));
    }

    // read the body as text first then deserialize so a schema mismatch gives us
    // the actual field/line from serde instead of some useless "error decoding
    // response body"
    let text = resp.text().await.map_err(|e| AppError::Network(e.to_string()))?;
    serde_json::from_str::<T>(&text)
        .map_err(|e| AppError::Network(format!("decode {url}: {e}")))
}

/// PUT/DELETE with an empty body, spotifys save/unsave endpoints return 200/204
pub(crate) async fn spotify_write(
    token:  &str,
    method: reqwest::Method,
    url:    &str,
) -> Result<(), AppError> {
    let resp = reqwest::Client::new()
        .request(method, url)
        .bearer_auth(token)
        .header("Content-Length", "0")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body   = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!("Spotify {status}: {body}")));
    }
    Ok(())
}

/// POST/DELETE/PUT with a json body, gives back the raw response json (for
/// endpoints that echo data back e.g create-playlist hands you the new id)
pub(crate) async fn spotify_write_json(
    token:  &str,
    method: reqwest::Method,
    url:    &str,
    body:   serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let resp = reqwest::Client::new()
        .request(method, url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body   = resp.text().await.unwrap_or_default();
        return Err(AppError::Network(format!("Spotify {status}: {body}")));
    }

    // some endpoints (e.g DELETE tracks) return an empty/near empty body so
    // just treat that as json null instead of erroring out
    let text = resp.text().await.unwrap_or_default();
    if text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| AppError::Network(e.to_string()))
}
