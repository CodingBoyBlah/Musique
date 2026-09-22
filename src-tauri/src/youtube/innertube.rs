// InnerTube transport: builds the `context` envelope every youtubei call needs
// and POSTs it.
//
// InnerTube is YouTube's own internal RPC API - the same one the web and mobile
// apps talk to. There is no API key or OAuth here: the request is authenticated
// purely by *looking like* a known client, which is why the `context.client`
// block and the `X-YouTube-Client-*` headers have to agree exactly. A mismatch
// between the two is one of the few ways to get a hard rejection.

use serde_json::{json, Value};

use crate::{errors::AppError, http};

use super::client::YtClient;

const ORIGIN_MUSIC: &str = "https://music.youtube.com";
const ORIGIN_WWW: &str = "https://www.youtube.com";

impl YtClient {
    fn origin(&self) -> &'static str {
        if self.music_origin { ORIGIN_MUSIC } else { ORIGIN_WWW }
    }

    /// The `context.client` object. Fields are omitted rather than sent as null
    /// - InnerTube tolerates missing keys but is picky about nulls on some
    /// clients, and a real client would simply not send them.
    pub fn context(&self, visitor: Option<&str>) -> Value {
        let mut c = json!({
            "clientName":    self.name,
            "clientVersion": self.version,
            "gl":            "US",
            "hl":            "en",
        });
        let o = c.as_object_mut().expect("just built as object");
        // Must agree with the X-Goog-Visitor-Id header - YouTube checks both.
        if let Some(v) = visitor { o.insert("visitorData".into(), json!(v)); }
        if let Some(v) = self.os_name      { o.insert("osName".into(), json!(v)); }
        if let Some(v) = self.os_version   { o.insert("osVersion".into(), json!(v)); }
        if let Some(v) = self.device_make  { o.insert("deviceMake".into(), json!(v)); }
        if let Some(v) = self.device_model { o.insert("deviceModel".into(), json!(v)); }
        if let Some(v) = self.android_sdk  { o.insert("androidSdkVersion".into(), json!(v)); }
        if let Some(v) = self.platform     { o.insert("platform".into(), json!(v)); }
        if self.ua_in_body                 { o.insert("userAgent".into(), json!(self.user_agent)); }
        c
    }
}

/// POST to a youtubei endpoint (`player`, `search`, `browse`, ...).
///
/// `body` is merged with the `context` envelope rather than nested by the
/// caller, so call sites only spell out the fields that are actually specific
/// to their request.
pub async fn post(client: &YtClient, endpoint: &str, body: Value) -> Result<Value, AppError> {
    let url = format!("{}/youtubei/v1/{endpoint}?prettyPrint=false", client.origin());

    // Anonymous calls without a visitor identity get bot-challenged. Best
    // effort: `None` still works for plenty of videos, so a scrape failure
    // lowers the success rate rather than blocking the request. See visitor.rs.
    let visitor = super::visitor::get().await;

    let mut payload =
        json!({ "context": { "client": client.context(visitor.as_deref()), "user": {} } });
    if let (Some(dst), Some(src)) = (payload.as_object_mut(), body.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }

    let res = http::client()
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Goog-Api-Format-Version", "1")
        .header("X-YouTube-Client-Name", client.id)
        .header("X-YouTube-Client-Version", client.version)
        .header("Origin", client.origin())
        .header("X-Origin", client.origin())
        .header("Referer", format!("{}/", client.origin()))
        .header("User-Agent", client.user_agent);

    let res = match visitor.as_deref() {
        Some(v) => res.header("X-Goog-Visitor-Id", v),
        None => res,
    };

    let res = res
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("youtubei {endpoint}: {e}")))?;

    let status = res.status();
    if !status.is_success() {
        // Body often carries a useful InnerTube error; truncate so a giant HTML
        // error page can't flood the log or the IPC error string.
        let body = res.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(200).collect();
        return Err(AppError::Network(format!(
            "youtubei {endpoint} returned {status} [{}]: {snippet}",
            client.name
        )));
    }

    res.json()
        .await
        .map_err(|e| AppError::Network(format!("youtubei {endpoint} decode: {e}")))
}

/// Depth-first search for every value under `key` anywhere in a JSON tree.
///
/// InnerTube responses are deeply nested renderer unions whose exact shape
/// shifts between clients and A/B buckets. Walking for the key we want is far
/// more robust than hardcoding a path like
/// `contents.tabbedSearchResultsRenderer.tabs[0]...` - and those paths are
/// exactly what breaks silently when YouTube reshuffles a response.
pub fn find_all<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                if k == key {
                    out.push(child);
                }
                find_all(child, key, out);
            }
        }
        Value::Array(arr) => {
            for child in arr {
                find_all(child, key, out);
            }
        }
        _ => {}
    }
}

/// First match of [`find_all`].
pub fn find_first<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    let mut out = Vec::new();
    find_all(v, key, &mut out);
    out.into_iter().next()
}

/// Concatenate a `{ "runs": [{ "text": ... }] }` text node.
pub fn runs_text(v: &Value) -> String {
    v.get("runs")
        .and_then(|r| r.as_array())
        .map(|runs| {
            runs.iter()
                .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                .collect::<String>()
        })
        .unwrap_or_default()
}
