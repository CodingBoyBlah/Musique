//! last.fm scrobbling. auth is the normal desktop flow:
//!   1. auth.getToken                -> a request token
//!   2. user authorizes the token in the browser
//!   3. auth.getSession              -> a permanent session key
//! every authed call is md5 signed with the shared secret. the secret
//! and session key live ONLY in the os keyring (same store as spotify creds),
//! the username (not sensitive) lives in the settings table

use std::collections::BTreeMap;

use keyring::Entry;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::errors::AppError;

const SERVICE:     &str = "spotify-client";
const ACC_KEY:     &str = "lastfm_api_key";
const ACC_SECRET:  &str = "lastfm_api_secret";
const ACC_SESSION: &str = "lastfm_session_key";
const API:         &str = "https://ws.audioscrobbler.com/2.0/";

fn keyring_get(account: &str) -> Result<Option<String>, AppError> {
    match Entry::new(SERVICE, account)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

fn keyring_set(account: &str, value: &str) -> Result<(), AppError> {
    Entry::new(SERVICE, account)?.set_password(value)?;
    Ok(())
}

fn keyring_del(account: &str) -> Result<(), AppError> {
    match Entry::new(SERVICE, account)?.delete_password() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

/// last.fm api_sig: md5 of every param (except `format`/`callback`) glued together
/// as `key+value` in ascii sorted key order, then the shared secret on the end
fn sign(params: &BTreeMap<String, String>, secret: &str) -> String {
    let mut buf = String::new();
    for (k, v) in params {
        if k == "format" || k == "callback" {
            continue;
        }
        buf.push_str(k);
        buf.push_str(v);
    }
    buf.push_str(secret);
    format!("{:x}", md5::compute(buf))
}

/// make a signed call. `write` means POST (scrobble/now-playing/getSession),
/// else GET. gives back parsed json. adds `api_sig` + `format=json` for u automatically
async fn call(
    api_key: &str,
    secret:  &str,
    method:  &str,
    mut params: BTreeMap<String, String>,
    write:   bool,
) -> Result<serde_json::Value, AppError> {
    params.insert("method".into(), method.into());
    params.insert("api_key".into(), api_key.into());
    let sig = sign(&params, secret);
    params.insert("api_sig".into(), sig);
    params.insert("format".into(), "json".into());

    let client = reqwest::Client::new();
    let req = if write {
        client.post(API).form(&params)
    } else {
        client.get(API).query(&params)
    };

    let resp = req.send().await.map_err(|e| AppError::Network(e.to_string()))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| AppError::Network(e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Network(format!("last.fm decode: {e}")))?;

    // last.fm returns 200 with an {"error":N,"message":...} body when it fails, sneaky
    if let Some(code) = json.get("error").and_then(|v| v.as_i64()) {
        let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
        return Err(AppError::Auth(format!("last.fm error {code}: {msg}")));
    }
    if !status.is_success() {
        return Err(AppError::Network(format!("last.fm HTTP {status}")));
    }
    Ok(json)
}

// ─── stored credential access ─────────────────────────────────────────────────

struct Creds {
    api_key: String,
    secret:  String,
    session: String,
}

/// all three secrets u need to scrobble, or None if not fully connected
fn full_creds() -> Result<Option<Creds>, AppError> {
    let (Some(api_key), Some(secret), Some(session)) =
        (keyring_get(ACC_KEY)?, keyring_get(ACC_SECRET)?, keyring_get(ACC_SESSION)?)
    else {
        return Ok(None);
    };
    Ok(Some(Creds { api_key, secret, session }))
}

// ─── public surface, called by commands ──────────────────────────────────────

#[derive(Serialize)]
pub struct LastfmStatus {
    pub configured: bool,        // api key + secret are there
    pub connected:  bool,        // session key is there
    pub username:   Option<String>,
}

pub async fn status(pool: &SqlitePool) -> Result<LastfmStatus, AppError> {
    let configured = keyring_get(ACC_KEY)?.is_some() && keyring_get(ACC_SECRET)?.is_some();
    let connected  = keyring_get(ACC_SESSION)?.is_some();
    let username   = crate::auth::get_setting_value(pool, "lastfm_username").await?;
    Ok(LastfmStatus { configured, connected, username })
}

pub fn save_api(api_key: &str, secret: &str) -> Result<(), AppError> {
    let api_key = api_key.trim();
    let secret  = secret.trim();
    if api_key.is_empty() || secret.is_empty() {
        return Err(AppError::InvalidInput("API key and secret are required".into()));
    }
    keyring_set(ACC_KEY, api_key)?;
    keyring_set(ACC_SECRET, secret)?;
    Ok(())
}

/// step 1: grab a request token and hand back the url the user has to open to
/// authorize it. caller opens the browser then calls `finish_auth`
pub async fn start_auth() -> Result<(String, String), AppError> {
    let api_key = keyring_get(ACC_KEY)?
        .ok_or_else(|| AppError::Auth("Save your Last.fm API key + secret first".into()))?;
    let secret  = keyring_get(ACC_SECRET)?
        .ok_or_else(|| AppError::Auth("Save your Last.fm API key + secret first".into()))?;

    let json = call(&api_key, &secret, "auth.getToken", BTreeMap::new(), false).await?;
    let token = json.get("token").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Network("last.fm: no token returned".into()))?
        .to_string();

    let url = format!("https://www.last.fm/api/auth/?api_key={api_key}&token={token}");
    Ok((token, url))
}

/// step 3: swap an authorized token for a permanent session key. we poll
/// cuz the user authorizes out of band in the browser (theres no callback)
pub async fn finish_auth(pool: &SqlitePool, token: &str) -> Result<LastfmStatus, AppError> {
    let api_key = keyring_get(ACC_KEY)?
        .ok_or_else(|| AppError::Auth("Missing API key".into()))?;
    let secret  = keyring_get(ACC_SECRET)?
        .ok_or_else(|| AppError::Auth("Missing API secret".into()))?;

    let mut params = BTreeMap::new();
    params.insert("token".to_string(), token.to_string());

    // poll for like ~60s waiting for the user to authorize in the browser
    let mut last_err = AppError::Auth("Authorization timed out".into());
    for _ in 0..30 {
        match call(&api_key, &secret, "auth.getSession", params.clone(), false).await {
            Ok(json) => {
                let session = json.get("session").cloned().unwrap_or_default();
                let key = session.get("key").and_then(|v| v.as_str())
                    .ok_or_else(|| AppError::Network("last.fm: no session key".into()))?;
                let name = session.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                keyring_set(ACC_SESSION, key)?;
                crate::auth::upsert_setting(pool, "lastfm_username", &name).await?;
                return status(pool).await;
            }
            Err(e) => { last_err = e; }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Err(last_err)
}

pub async fn disconnect(pool: &SqlitePool) -> Result<(), AppError> {
    keyring_del(ACC_SESSION)?;
    let _ = sqlx::query("DELETE FROM settings WHERE key = 'lastfm_username'")
        .execute(pool)
        .await;
    Ok(())
}

/// fully forget last.fm, session + api creds, all of it
pub async fn clear_all(pool: &SqlitePool) -> Result<(), AppError> {
    keyring_del(ACC_SESSION)?;
    keyring_del(ACC_KEY)?;
    keyring_del(ACC_SECRET)?;
    let _ = sqlx::query("DELETE FROM settings WHERE key = 'lastfm_username'")
        .execute(pool)
        .await;
    Ok(())
}

/// set the "now playing" indicator. does nothing (Ok) when not connected
pub async fn now_playing(artist: &str, track: &str, album: &str) -> Result<(), AppError> {
    let Some(c) = full_creds()? else { return Ok(()) };
    let mut p = BTreeMap::new();
    p.insert("artist".into(), artist.to_string());
    p.insert("track".into(), track.to_string());
    if !album.is_empty() { p.insert("album".into(), album.to_string()); }
    p.insert("sk".into(), c.session);
    call(&c.api_key, &c.secret, "track.updateNowPlaying", p, true).await?;
    Ok(())
}

/// scrobble a played track. `timestamp` = unix seconds when the track STARTED.
/// does nothing (Ok) when not connected
pub async fn scrobble(artist: &str, track: &str, album: &str, timestamp: i64) -> Result<(), AppError> {
    let Some(c) = full_creds()? else { return Ok(()) };
    let mut p = BTreeMap::new();
    p.insert("artist".into(), artist.to_string());
    p.insert("track".into(), track.to_string());
    if !album.is_empty() { p.insert("album".into(), album.to_string()); }
    p.insert("timestamp".into(), timestamp.to_string());
    p.insert("sk".into(), c.session);
    call(&c.api_key, &c.secret, "track.scrobble", p, true).await?;
    Ok(())
}
