// Anonymous visitor identity for InnerTube calls.
//
// YouTube issues every client a `visitorData` token - an opaque base64 protobuf
// that identifies the session without identifying a user. Requests that omit it
// are treated as coming from nothing in particular, and YouTube increasingly
// answers those with:
//
//     playabilityStatus: LOGIN_REQUIRED
//     "Sign in to confirm you're not a bot"
//
// This is not an IP-level block and not a per-account thing. Measured on one
// machine, same moment, same clients: a control video returned OK on all four
// profiles while a challenged video returned LOGIN_REQUIRED on all four.
// Sending a `visitorData` fetched seconds earlier flipped the challenged video
// to OK on both visionOS profiles. The token is the difference.
//
// innertubex threads this through `YouTubeClient.toContext(locale, visitorData,
// dataSyncId)` and the `X-Goog-Visitor-Id` header; this is the same mechanism.
//
// The token is scraped from a YouTube page's `ytcfg` blob, cached process-wide,
// and refreshed on demand when a call comes back challenged.

use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use crate::http;

/// How long a scraped token is trusted before being refetched.
///
/// The token embeds its own issue timestamp and YouTube honours them far
/// longer than this, but a cheap periodic refresh costs one request and avoids
/// carrying a token that has quietly aged out of favour.
const TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// Pages to scrape, in order. `music.` is first because that is the origin the
/// player calls use; `www.` is a fallback for when the music page shape changes.
const SOURCES: &[&str] = &["https://music.youtube.com/", "https://www.youtube.com/"];

const SCRAPE_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) Gecko/20100101 Firefox/140.0";

static CACHE: RwLock<Option<(String, Instant)>> = RwLock::const_new(None);

/// The current visitor token, fetching one if needed.
///
/// Returns `None` rather than erroring when YouTube can't be scraped: a request
/// without a visitor id still succeeds for plenty of videos, so a failure here
/// should degrade the success rate, not block playback outright.
pub async fn get() -> Option<String> {
    if let Some((token, at)) = CACHE.read().await.as_ref() {
        if at.elapsed() < TTL {
            return Some(token.clone());
        }
    }
    refresh().await
}

/// Force a new token, discarding any cached one.
///
/// Called when a player response comes back challenged: the most likely cause
/// is a token YouTube has stopped accepting, and retrying with the same one
/// would just fail identically.
pub async fn refresh() -> Option<String> {
    let mut guard = CACHE.write().await;

    // Another task may have refreshed while this one waited on the lock -
    // scraping again would be pure waste.
    if let Some((token, at)) = guard.as_ref() {
        if at.elapsed() < Duration::from_secs(30) {
            return Some(token.clone());
        }
    }

    for url in SOURCES {
        match scrape(url).await {
            Some(token) => {
                eprintln!("[youtube] visitor id refreshed from {url}");
                *guard = Some((token.clone(), Instant::now()));
                return Some(token);
            }
            None => eprintln!("[youtube] no visitorData in {url}"),
        }
    }

    // Leave any existing token in place; a stale one beats none at all.
    None
}

async fn scrape(url: &str) -> Option<String> {
    let body = http::client()
        .get(url)
        .header("User-Agent", SCRAPE_UA)
        .header("Accept-Language", "en-US,en")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    extract(&body)
}

/// Pull `"visitorData":"..."` out of a YouTube page's inline config.
///
/// Hand-rolled rather than parsed: the token sits inside a `ytcfg.set({...})`
/// JavaScript call, so there is no JSON document to parse without first
/// locating the same substring anyway.
fn extract(html: &str) -> Option<String> {
    const KEY: &str = "\"visitorData\":\"";
    let start = html.find(KEY)? + KEY.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    let raw = &rest[..end];

    if raw.is_empty() {
        return None;
    }
    Some(unescape(raw))
}

/// Undo the JSON/JS string escapes YouTube applies to the token.
///
/// The value is base64url, so `=` padding arrives as `=` and any `&` in a
/// percent-encoded segment as `&`. Passing those through literally yields
/// a token YouTube rejects.
fn unescape(s: &str) -> String {
    s.replace("\\u003d", "=")
        .replace("\\u003D", "=")
        .replace("\\u0026", "&")
        .replace("\\u002F", "/")
        .replace("\\u002f", "/")
        .replace("\\/", "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_from_ytcfg_shape() {
        let html = r#"<script>ytcfg.set({"INNERTUBE_CONTEXT":{"client":{"visitorData":"CgtBQkNERUZHSElKSw%3D%3D","hl":"en"}}});</script>"#;
        assert_eq!(extract(html).unwrap(), "CgtBQkNERUZHSElKSw%3D%3D");
    }

    #[test]
    fn unescapes_padding_and_ampersands() {
        let html = r#"{"visitorData":"Cgt4eXo=&foo"}"#;
        assert_eq!(extract(html).unwrap(), "Cgt4eXo=&foo");
    }

    #[test]
    fn absent_or_empty_is_none() {
        assert!(extract("<html>nothing here</html>").is_none());
        assert!(extract(r#"{"visitorData":""}"#).is_none());
    }

    /// The real token is long base64url; make sure a realistic one survives intact.
    #[test]
    fn keeps_base64url_characters() {
        let token = "Cgs3OXJmUmtDZDFBQSjWnrXVBjIKCgJJThIEGgAgD2-_A";
        let html = format!(r#"{{"visitorData":"{token}","x":1}}"#);
        assert_eq!(extract(&html).unwrap(), token);
    }
}
