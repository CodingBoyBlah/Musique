pub mod types;

use crate::errors::AppError;

use futures_util::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::Semaphore;

// Two lanes. Library sync pulls dozens of pages at once; if it shared one pool
// of permits with the UI, a search or an album open typed during a first sync
// queued behind every one of those pages. Background work runs inside
// `background(..)` and only ever takes the background lane.
static API_SEMAPHORE: Semaphore = Semaphore::const_new(8);
static BG_SEMAPHORE:  Semaphore = Semaphore::const_new(6);

tokio::task_local! {
    static BACKGROUND: ();
}

/// Run `fut` with every Spotify request inside it on the background lane.
pub(crate) async fn background<F: std::future::Future>(fut: F) -> F::Output {
    BACKGROUND.scope((), fut).await
}

fn lane() -> &'static Semaphore {
    if BACKGROUND.try_with(|_| ()).is_ok() { &BG_SEMAPHORE } else { &API_SEMAPHORE }
}

/// Every page of an offset-paginated endpoint, in order. The first page comes
/// alone to learn `total`; the rest go out together instead of one round trip
/// after another (a 3000-song library is 60 pages). The lane semaphore still
/// caps how many are actually in flight.
pub(crate) async fn get_all_pages<P: serde::de::DeserializeOwned>(
    token:   &str,
    limit:   i64,
    url_for: impl Fn(i64) -> String,
    total:   impl Fn(&P) -> i64,
) -> Result<Vec<P>, AppError> {
    let first: P = spotify_get(token, &url_for(0)).await?;
    let total = total(&first);
    // the first page may have taught us this client gets smaller pages
    let limit = limit_cap(&url_for(0)).map_or(limit, |cap| limit.min(cap)).max(1);
    let rest: Vec<P> = futures_util::stream::iter((limit..total).step_by(limit as usize))
        .map(|offset| {
            let url = url_for(offset);
            async move { spotify_get::<P>(token, &url).await }
        })
        .buffered(8)
        .try_collect()
        .await?;
    let mut pages = Vec::with_capacity(rest.len() + 1);
    pages.push(first);
    pages.extend(rest);
    Ok(pages)
}

async fn send_request(builder: reqwest::RequestBuilder) -> Result<reqwest::Response, AppError> {
    let mut attempt = 0;
    loop {
        attempt += 1;
        let _permit = lane()
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

// Largest `limit` each endpoint accepted, learned from "Invalid limit" 400s.
// Spotify caps page sizes per client (search on some client ids rejects
// anything over 10), and which caps apply differs between client ids.
static LIMIT_CAPS: Mutex<Option<HashMap<String, i64>>> = Mutex::new(None);

fn endpoint_key(url: &url::Url) -> String {
    format!("{}{}", url.host_str().unwrap_or_default(), url.path())
}

fn limit_cap(url: &str) -> Option<i64> {
    let url = url::Url::parse(url).ok()?;
    LIMIT_CAPS.lock().ok()?.as_ref()?.get(&endpoint_key(&url)).copied()
}

fn set_limit(url: &url::Url, limit: i64) -> String {
    let pairs: Vec<(String, String)> = url.query_pairs()
        .map(|(k, v)| if k == "limit" { (k.into_owned(), limit.to_string()) } else { (k.into_owned(), v.into_owned()) })
        .collect();
    let mut out = url.clone();
    out.query_pairs_mut().clear().extend_pairs(pairs);
    out.to_string()
}

fn url_limit(url: &url::Url) -> Option<i64> {
    url.query_pairs().find(|(k, _)| k == "limit").and_then(|(_, v)| v.parse().ok())
}

/// Next page size to try after `limit` was rejected.
fn smaller_limit(limit: i64) -> Option<i64> {
    [50, 20, 10, 5, 1].into_iter().find(|&l| l < limit)
}

pub(crate) async fn spotify_get<T: serde::de::DeserializeOwned>(
    token: &str,
    url:   &str,
) -> Result<T, AppError> {
    let mut url = url.to_string();
    if let Ok(parsed) = url::Url::parse(&url) {
        if let (Some(limit), Some(cap)) = (url_limit(&parsed), limit_cap(&url)) {
            if limit > cap { url = set_limit(&parsed, cap); }
        }
    }

    let resp = loop {
        let resp = send_request(crate::http::client().get(&url).bearer_auth(token)).await?;
        if resp.status().is_success() { break resp; }

        let status = resp.status().as_u16();
        let body   = resp.text().await.unwrap_or_default();
        if status == 400 && body.contains("Invalid limit") {
            if let Ok(parsed) = url::Url::parse(&url) {
                if let Some(lower) = url_limit(&parsed).and_then(smaller_limit) {
                    eprintln!("[spotify-api] limit rejected on {}, retrying with {lower}", parsed.path());
                    if let Ok(mut caps) = LIMIT_CAPS.lock() {
                        caps.get_or_insert_with(HashMap::new).insert(endpoint_key(&parsed), lower);
                    }
                    url = set_limit(&parsed, lower);
                    continue;
                }
            }
        }
        return Err(AppError::Network(format!("Spotify {status}: {body}")));
    };
    let url = url.as_str();

    // Deserialize straight off the raw bytes. `text()` would allocate a String
    // and UTF-8 validate the whole payload first (liked-songs/playlist pages run
    // to hundreds of KB) just to hand it to serde, which validates again.
    // `from_slice` skips that entire copy. Schema mismatches still report the
    // real serde field/line, same as before.
    let bytes = resp.bytes().await.map_err(|e| AppError::Network(e.to_string()))?;
    serde_json::from_slice::<T>(&bytes)
        .map_err(|e| AppError::Network(format!("decode {url}: {e}")))
}

/// PUT/DELETE with an empty body, spotifys save/unsave endpoints return 200/204
pub(crate) async fn spotify_write(
    token:  &str,
    method: reqwest::Method,
    url:    &str,
) -> Result<(), AppError> {
    let resp = send_request(
        crate::http::client()
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
        crate::http::client()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_limit_keeps_other_params() {
        let url = url::Url::parse("https://api.spotify.com/v1/search?q=eric+nam&type=track&limit=20").unwrap();
        let out = url::Url::parse(&set_limit(&url, 10)).unwrap();
        assert_eq!(url_limit(&out), Some(10));
        assert_eq!(out.query_pairs().find(|(k, _)| k == "q").unwrap().1, "eric nam");
    }

    #[test]
    fn smaller_limit_steps_down() {
        assert_eq!(smaller_limit(20), Some(10));
        assert_eq!(smaller_limit(50), Some(20));
        assert_eq!(smaller_limit(1), None);
    }
}
