//! pathfinder - the graphql api the official clients build their pages on
//! (home feed, artist overview, extracted colours...). the only place the
//! personalised home (daily mixes, daylist, made for you) comes from.
//!
//! requests are "persisted queries": you send an operation name plus the
//! sha256 of the query text, never the query itself. those hashes change
//! whenever spotify ships a new web player, so rather than hardcoding them
//! (they'd rot within weeks) they're read out of the live web player bundle,
//! cached, and refreshed when pathfinder says a hash is unknown. the built-in
//! table is only the last resort.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use super::{cache, spclient};
use crate::{errors::AppError, state::AppState};

const ENDPOINT: &str = "https://api-partner.spotify.com/pathfinder/v2/query";
const WEB_PLAYER: &str = "https://open.spotify.com/";
const BUNDLE_PREFIX: &str = "https://open.spotifycdn.com/cdn/build/web-player/";
/// the value the web player itself sends; pathfinder uses it to pick response shapes
const APP_VERSION: &str = "896000000";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36";
const HASH_TTL: Duration = Duration::from_secs(12 * 3600);

/// last-known hashes (web player da8b87c8, sep 2026). only used when the live
/// bundle can't be read
const FALLBACK: &[(&str, &str)] = &[
    ("home", "76243c78b0e20ecdbe41b794dec8cbe73f75e585b0a7201b8d2e84578412847a"),
    ("queryArtistOverview", "9f8134ef565e78621f1e1793555bd6633c5ac144ae0f89604ed3ae3f80b3c8e6"),
    ("fetchExtractedColors", "36e90fcaea00d47c695fce31874efeb2519b97d4cd0ee1abfb4f8dc9348596ea"),
    ("getAlbum", "6a74b456cd1735c9193d9e8ec8cc5184cad7ce13572210315229db3975964361"),
    ("getTrack", "a8ef9e9f02b836feb0da3003c31dbb30decc6f4b473ef89ca88c882386d668de"),
    ("fetchPlaylist", "243c0ba2736f16da721e3a227004bbcdb8df6c846f198bd478172e00aa1faf42"),
    ("queryWhatsNewFeed", "d889c8c936ab192af8ced595427f5ba2acdf63478fdc0a181c8d477f8322630e"),
    ("queryNpvArtist", "e1ae46a21911a3075c1aa29bf09a6c60f9a45b6a2b1132429f10ad06c299b5d7"),
];

static HASHES: Mutex<Option<(Instant, HashMap<String, String>)>> = Mutex::new(None);

/// every `"<op>","query|mutation","<sha256>"` triple in a bundle. that's how
/// the web player declares its persisted operations.
pub(crate) fn extract_hashes(js: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for marker in ["\",\"query\",\"", "\",\"mutation\",\""] {
        let mut from = 0;
        while let Some(pos) = js[from..].find(marker) {
            let at = from + pos;
            from = at + marker.len();
            let hash = &js[from..js.len().min(from + 64)];
            if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            // the op name is the quoted identifier just before the marker
            let before = &js[..at];
            let Some(start) = before.rfind('"') else { continue };
            let name = &before[start + 1..];
            if !name.is_empty() && name.len() < 80 && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                out.entry(name.to_string()).or_insert_with(|| hash.to_string());
            }
        }
    }
    out
}

/// web-player bundle urls referenced by the landing page html
fn bundle_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut from = 0;
    while let Some(pos) = html[from..].find(BUNDLE_PREFIX) {
        let at = from + pos;
        let rest = &html[at..];
        let end = rest.find(|c: char| c == '"' || c == '\'' || c.is_whitespace()).unwrap_or(rest.len());
        let url = &rest[..end];
        if url.ends_with(".js") && !urls.iter().any(|u| u == url) {
            urls.push(url.to_string());
        }
        from = at + end.max(1);
    }
    urls
}

async fn scrape_hashes() -> Result<HashMap<String, String>, AppError> {
    let http = crate::http::client();
    let html = http.get(WEB_PLAYER).header("user-agent", UA).send().await?.text().await?;
    let mut hashes = HashMap::new();
    // the main web-player bundle carries nearly every operation; vendor/encore
    // chunks carry none, so skip those to save a few MB
    for url in bundle_urls(&html).into_iter().filter(|u| u.contains("/web-player.")) {
        if let Ok(resp) = http.get(&url).header("user-agent", UA).send().await {
            if let Ok(js) = resp.text().await {
                hashes.extend(extract_hashes(&js));
            }
        }
    }
    if hashes.is_empty() {
        return Err(AppError::Network("pathfinder: no operations in the web player bundle".into()));
    }
    Ok(hashes)
}

async fn hash_for(app: &AppHandle, op: &str, force_refresh: bool) -> Result<String, AppError> {
    if !force_refresh {
        if let Some((at, map)) = HASHES.lock().unwrap().as_ref() {
            if at.elapsed() < HASH_TTL {
                if let Some(h) = map.get(op) {
                    return Ok(h.clone());
                }
            }
        }
    }
    let pool = app.state::<AppState>().db.clone();
    let map = if force_refresh {
        scrape_hashes().await
    } else {
        match cache::get_json::<HashMap<String, String>>(&pool, "spotify:pathfinder", "hashes", 12 * cache::HOUR).await {
            Some(m) => Ok(m),
            None => scrape_hashes().await,
        }
    };
    let map = match map {
        Ok(m) => {
            cache::put_json(&pool, "spotify:pathfinder", "hashes", &m).await;
            m
        }
        Err(e) => {
            let mut m = cache::get_json_stale::<HashMap<String, String>>(&pool, "spotify:pathfinder", "hashes")
                .await
                .unwrap_or_default();
            if m.is_empty() {
                eprintln!("[pathfinder] hash scrape failed, using built-in table: {e}");
            }
            for (k, v) in FALLBACK {
                m.entry((*k).to_string()).or_insert_with(|| (*v).to_string());
            }
            m
        }
    };
    let hash = map
        .get(op)
        .cloned()
        .or_else(|| FALLBACK.iter().find(|(k, _)| *k == op).map(|(_, v)| (*v).to_string()))
        .ok_or_else(|| AppError::NotFound(format!("pathfinder: unknown operation {op}")))?;
    *HASHES.lock().unwrap() = Some((Instant::now(), map));
    Ok(hash)
}

async fn post(token: &str, client_token: Option<&str>, op: &str, hash: &str, variables: &Value) -> Result<(u16, Value), AppError> {
    let body = json!({
        "variables": variables,
        "operationName": op,
        "extensions": { "persistedQuery": { "version": 1, "sha256Hash": hash } },
    });
    let mut req = crate::http::client()
        .post(ENDPOINT)
        .bearer_auth(token)
        .header("app-platform", "WebPlayer")
        .header("spotify-app-version", APP_VERSION)
        .header("accept", "application/json")
        .json(&body);
    if let Some(ct) = client_token {
        req = req.header("client-token", ct);
    }
    let resp = req.send().await?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    Ok((status, v))
}

fn unknown_hash(v: &Value) -> bool {
    v.get("errors")
        .and_then(|e| e.as_array())
        .map(|errs| {
            errs.iter().any(|e| {
                let m = e.get("message").and_then(|m| m.as_str()).unwrap_or_default();
                m.contains("PersistedQueryNotFound") || m.contains("persisted query")
            })
        })
        .unwrap_or(false)
}

/// run a persisted pathfinder operation, returning its `data`. tries the
/// session's first-party token (+ client-token) first, then the oauth token.
/// an unknown-hash answer re-reads the web player once and retries.
pub async fn query(app: &AppHandle, op: &str, variables: Value) -> Result<Value, AppError> {
    let first_party = spclient::first_party_tokens(app).await.ok();
    let oauth = crate::commands::spotify::tok(app).await.ok();
    if first_party.is_none() && oauth.is_none() {
        return Err(AppError::Auth("pathfinder: no token".into()));
    }

    let mut hash = hash_for(app, op, false).await?;
    let mut refreshed = false;
    loop {
        let mut last = None;
        let mut attempts: [(&str, Option<&str>); 2] = [("", None), ("", None)];
        let mut count = 0;
        if let Some((t, ct)) = &first_party {
            attempts[count] = (t.as_str(), Some(ct.as_str()));
            count += 1;
        }
        if let Some(t) = &oauth {
            attempts[count] = (t.as_str(), None);
            count += 1;
        }
        for &(token, ct) in &attempts[..count] {
            let (status, v) = post(token, ct, op, &hash, &variables).await?;
            if unknown_hash(&v) {
                last = Some((status, v));
                break;
            }
            if (200..300).contains(&status) {
                if let Some(data) = v.get("data").filter(|d| !d.is_null()) {
                    return Ok(data.clone());
                }
            }
            last = Some((status, v));
            // auth problem with this token -> try the next one; anything else is final
            if status != 401 && status != 403 && status != 400 {
                break;
            }
        }
        let (status, v) = last.unwrap_or((0, Value::Null));
        if unknown_hash(&v) && !refreshed {
            refreshed = true;
            hash = hash_for(app, op, true).await?;
            continue;
        }
        let msg = v
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("no data");
        return Err(AppError::Network(format!("pathfinder {op} ({status}): {msg}")));
    }
}

/// dig through a response by path, `get(v, &["a", "b", "0", "c"])`
pub fn get<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for key in path {
        cur = match key.parse::<usize>() {
            Ok(i) if cur.is_array() => cur.get(i)?,
            _ => cur.get(*key)?,
        };
    }
    Some(cur)
}

pub fn get_str<'a>(v: &'a Value, path: &[&str]) -> Option<&'a str> {
    get(v, path).and_then(|x| x.as_str()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulls_operations_out_of_a_bundle() {
        let js = r#"let a=new i.l("home","query","76243c78b0e20ecdbe41b794dec8cbe73f75e585b0a7201b8d2e84578412847a",null);
            let b=new i.l("addToLibrary","mutation","aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",null);
            x("bad name!","query","76243c78b0e20ecdbe41b794dec8cbe73f75e585b0a7201b8d2e84578412847a");
            y("short","query","abc");"#;
        let h = extract_hashes(js);
        assert_eq!(h.get("home").map(String::as_str), Some("76243c78b0e20ecdbe41b794dec8cbe73f75e585b0a7201b8d2e84578412847a"));
        assert!(h.contains_key("addToLibrary"));
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn finds_bundles() {
        let html = r#"<script src="https://open.spotifycdn.com/cdn/build/web-player/web-player.da8b87c8.js"></script>
            <script src="https://open.spotifycdn.com/cdn/build/web-player/vendor~web-player.4ad2b3e0.js"></script>"#;
        let urls = bundle_urls(html);
        assert_eq!(urls.len(), 2);
        assert!(urls[0].ends_with("web-player.da8b87c8.js"));
    }

    #[test]
    fn paths() {
        let v = json!({"a": {"b": [{"c": "x"}]}});
        assert_eq!(get_str(&v, &["a", "b", "0", "c"]), Some("x"));
        assert_eq!(get_str(&v, &["a", "nope"]), None);
    }
}
