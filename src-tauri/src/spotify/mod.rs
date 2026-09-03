pub mod types;

use crate::errors::AppError;

static API_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(8);

async fn send_request(builder: reqwest::RequestBuilder) -> Result<reqwest::Response, AppError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        let _permit = API_SEMAPHORE
            .acquire()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let req = builder
            .try_clone()
            .ok_or_else(|| AppError::Network("Request not cloneable".into()))?;
        let resp = req.send().await?;

        let status = resp.status();
        if status.as_u16() == 429 && attempt <= 3 {
            let retry_after_secs = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1);
            let backoff = std::time::Duration::from_secs(retry_after_secs.clamp(1, 30));
            eprintln!("[spotify-api] 429 rate limit hit, backing off for {retry_after_secs}s (attempt {attempt}/3)");
            drop(_permit);
            tokio::time::sleep(backoff).await;
            continue;
        }

        if status.is_server_error() && attempt <= 2 {
            eprintln!("[spotify-api] 5xx server error ({status}), retrying in 800ms");
            drop(_permit);
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            continue;
        }

        return Ok(resp);
    }
}

pub(crate) async fn spotify_get<T: serde::de::DeserializeOwned>(
    token: &str,
    url:   &str,
) -> Result<T, AppError> {
    let resp = send_request(reqwest::Client::new().get(url).bearer_auth(token)).await?;

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
    let resp = send_request(
        reqwest::Client::new()
            .request(method, url)
            .bearer_auth(token)
            .header("Content-Length", "0"),
    )
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
    let resp = send_request(
        reqwest::Client::new()
            .request(method, url)
            .bearer_auth(token)
            .json(&body),
    )
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
