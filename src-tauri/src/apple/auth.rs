//! Apple Music credentials: web player bearer JWT scraping/caching and account `media-user-token` storage.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

const KEYRING_SERVICE: &str = "musique-apple";
const KEYRING_ACCOUNT: &str = "media-user-token";
const USER_TOKEN_ENV: &str = "MUSIQUE_APPLE_MEDIA_USER_TOKEN";
const BEARER_TOKEN_ENV: &str = "MUSIQUE_APPLE_BEARER_TOKEN";

const BEARER_CACHE_FILE: &str = "bearer.json";
const MIN_REMAINING_LIFETIME: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const SCRAPE_TIMEOUT: Duration = Duration::from_secs(30);
const WEB_PLAYER_ORIGIN: &str = "https://music.apple.com";

pub(crate) const SAFARI_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15";

static CACHE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
static MEM_BEARER: Mutex<Option<String>> = Mutex::new(None);
static FETCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedBearer {
    token: String,
    #[serde(default)]
    exp: Option<u64>,
}

/// Sets the directory used to cache Apple Music runtime metadata (such as the scraped bearer JWT).
pub fn init(dir: PathBuf) {
    if let Ok(mut guard) = CACHE_DIR.write() {
        *guard = Some(dir);
    }
}

fn cache_dir() -> PathBuf {
    if let Ok(guard) = CACHE_DIR.read() {
        if let Some(ref dir) = *guard {
            return dir.clone();
        }
    }
    fallback_cache_dir()
}

fn fallback_cache_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(base) = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty()) {
        return PathBuf::from(base).join("musique").join("apple");
    }
    #[cfg(not(target_os = "windows"))]
    if let Some(base) = std::env::var_os("XDG_CACHE_HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(base).join("musique").join("apple");
    }
    std::env::temp_dir().join("musique-apple")
}

/// Extracts `media-user-token` from either a `Cookie` header string, a `media-user-token=...`
/// pair, or a bare token string.
pub fn user_token_from_cookies(input: &str) -> Result<String, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Auth("Apple Music user token is empty".into()));
    }

    let token = if trimmed.contains(';') {
        trimmed
            .split(';')
            .map(str::trim)
            .filter_map(|part| part.split_once('='))
            .find(|(name, _)| name.trim().eq_ignore_ascii_case("media-user-token"))
            .map(|(_, val)| val.trim())
            .filter(|val| !val.is_empty())
            .ok_or_else(|| {
                AppError::Auth("Cookies do not contain a media-user-token".into())
            })?
    } else if let Some((name, val)) = trimmed.split_once('=') {
        if name.trim().eq_ignore_ascii_case("media-user-token") {
            let v = val.trim();
            if v.is_empty() {
                return Err(AppError::Auth("Apple Music media-user-token is empty".into()));
            }
            v
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    if token.chars().any(char::is_control) {
        return Err(AppError::Auth(
            "Apple Music user token contains invalid control characters".into(),
        ));
    }

    Ok(token.to_owned())
}

/// Persists the account `media-user-token` in the OS keyring.
pub fn store_user_token(token: &str) -> Result<(), AppError> {
    let clean = token.trim();
    if clean.is_empty() {
        return Err(AppError::Auth("Cannot store an empty Apple Music user token".into()));
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?.set_password(clean)?;
    Ok(())
}

/// Loads the stored account `media-user-token` from environment override or OS keyring.
pub fn load_user_token() -> Option<String> {
    if let Ok(env_val) = std::env::var(USER_TOKEN_ENV) {
        let trimmed = env_val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
    let entry = match keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[apple-auth] keyring init failed: {e}");
            return None;
        }
    };
    match entry.get_password() {
        Ok(val) if val.trim().is_empty() => None,
        Ok(val) => Some(val),
        Err(keyring::Error::NoEntry) => None,
        Err(e) => {
            eprintln!("[apple-auth] could not read user token from keyring: {e}");
            None
        }
    }
}

/// Removes the stored `media-user-token` from the OS keyring (falling back to a blank tombstone
/// if the credential store refuses deletion).
pub fn clear_user_token() {
    let entry = match keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[apple-auth] keyring init failed on clear: {e}");
            return;
        }
    };
    match entry.delete_password() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => {
            eprintln!("[apple-auth] could not delete user token from keyring: {e}");
            if let Err(e2) = entry.set_password("") {
                eprintln!("[apple-auth] could not blank user token in keyring: {e2}");
            }
        }
    }
}

/// Returns the Apple Music web-player developer bearer JWT, reading from env override, in-memory
/// cache, disk cache (if > 7 days remain), or scraping `https://music.apple.com/`.
pub async fn bearer(http: &reqwest::Client) -> Result<String, AppError> {
    if let Ok(env_token) = std::env::var(BEARER_TOKEN_ENV) {
        let trimmed = env_token.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
    }

    if let Some(mem) = load_mem_bearer() {
        if token_has_margin(&mem, None, SystemTime::now()) {
            return Ok(mem);
        }
    }

    let _guard = FETCH_LOCK.lock().await;

    if let Some(mem) = load_mem_bearer() {
        if token_has_margin(&mem, None, SystemTime::now()) {
            return Ok(mem);
        }
    }

    let dir = cache_dir();
    if let Some(cached) = load_disk_bearer(&dir, SystemTime::now()) {
        store_mem_bearer(&cached);
        return Ok(cached);
    }

    let scraped = scrape_bearer_from(http, WEB_PLAYER_ORIGIN).await?;
    save_disk_bearer(&dir, &scraped);
    store_mem_bearer(&scraped);
    Ok(scraped)
}

/// Drops both the in-memory and on-disk bearer JWT so the next [`bearer`] call scrapes a fresh one.
pub fn forget_bearer() {
    if let Ok(mut guard) = MEM_BEARER.lock() {
        *guard = None;
    }
    let path = cache_dir().join(BEARER_CACHE_FILE);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

fn load_mem_bearer() -> Option<String> {
    MEM_BEARER
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
}

fn store_mem_bearer(token: &str) {
    if let Ok(mut guard) = MEM_BEARER.lock() {
        *guard = Some(token.to_owned());
    }
}

fn jwt_exp(token: &str) -> Option<u64> {
    let payload_b64 = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    claims.get("exp")?.as_u64()
}

fn token_has_margin(token: &str, stored_exp: Option<u64>, now: SystemTime) -> bool {
    let Some(exp_secs) = stored_exp.or_else(|| jwt_exp(token)) else {
        return false;
    };
    let exp_time = UNIX_EPOCH + Duration::from_secs(exp_secs);
    exp_time
        .duration_since(now)
        .is_ok_and(|rem| rem > MIN_REMAINING_LIFETIME)
}

fn load_disk_bearer(dir: &Path, now: SystemTime) -> Option<String> {
    let path = dir.join(BEARER_CACHE_FILE);
    let data = std::fs::read(&path).ok()?;
    let cached: CachedBearer = serde_json::from_slice(&data).ok()?;
    if cached.token.trim().is_empty() {
        return None;
    }
    if token_has_margin(&cached.token, cached.exp, now) {
        Some(cached.token)
    } else {
        None
    }
}

fn save_disk_bearer(dir: &Path, token: &str) {
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("[apple-auth] cannot create bearer cache dir: {e}");
        return;
    }
    let payload = CachedBearer {
        token: token.to_owned(),
        exp: jwt_exp(token),
    };
    let Ok(bytes) = serde_json::to_vec(&payload) else {
        return;
    };
    let path = dir.join(BEARER_CACHE_FILE);
    if let Err(e) = std::fs::write(&path, bytes) {
        eprintln!("[apple-auth] cannot write bearer cache file: {e}");
    }
}

#[cfg(test)]
pub(crate) async fn scrape_bearer_from_test(
    http: &reqwest::Client,
    origin: &str,
) -> Result<String, AppError> {
    scrape_bearer_from(http, origin).await
}

async fn scrape_bearer_from(http: &reqwest::Client, origin: &str) -> Result<String, AppError> {
    let origin = origin.trim_end_matches('/');
    let html_resp = http
        .get(format!("{origin}/"))
        .header(reqwest::header::USER_AGENT, SAFARI_USER_AGENT)
        .timeout(SCRAPE_TIMEOUT)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("cannot reach Apple Music web player: {e}")))?;

    if !html_resp.status().is_success() {
        return Err(AppError::Auth(format!(
            "Apple Music web player returned HTTP {}",
            html_resp.status().as_u16()
        )));
    }

    let html = html_resp
        .text()
        .await
        .map_err(|e| AppError::Network(format!("cannot read Apple Music HTML: {e}")))?;

    if let Some(jwt) = extract_jwt(&html) {
        return Ok(jwt.to_owned());
    }

    let bundle_path = extract_bundle_path(&html).ok_or_else(|| {
        AppError::Auth("Apple Music page contains neither a bearer JWT nor an index bundle".into())
    })?;

    let script_url = if bundle_path.starts_with("http://") || bundle_path.starts_with("https://") {
        bundle_path.to_owned()
    } else {
        format!("{origin}{bundle_path}")
    };

    let js_resp = http
        .get(&script_url)
        .header(reqwest::header::USER_AGENT, SAFARI_USER_AGENT)
        .timeout(SCRAPE_TIMEOUT)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("cannot fetch Apple Music bundle: {e}")))?;

    if !js_resp.status().is_success() {
        return Err(AppError::Auth(format!(
            "Apple Music script bundle returned HTTP {}",
            js_resp.status().as_u16()
        )));
    }

    let script = js_resp
        .text()
        .await
        .map_err(|e| AppError::Network(format!("cannot read Apple Music bundle: {e}")))?;

    extract_jwt(&script)
        .map(str::to_owned)
        .ok_or_else(|| AppError::Auth("Apple Music script bundle contains no bearer JWT".into()))
}

fn extract_bundle_path(html: &str) -> Option<&str> {
    for prefix in ["/assets/index~", "/assets/index-"] {
        let mut search_from = 0usize;
        while let Some(rel) = html[search_from..].find(prefix) {
            let start = search_from + rel;
            let rest = &html[start..];
            if let Some(js_rel) = rest.find(".js") {
                let end = js_rel + 3;
                let candidate = &rest[..end];
                if !candidate["/assets/".len()..].contains('/')
                    && !candidate.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'')
                {
                    return Some(candidate);
                }
            }
            search_from = start + prefix.len();
        }
    }
    None
}

fn extract_jwt(body: &str) -> Option<&str> {
    let mut pos = 0usize;
    while let Some(rel) = body[pos..].find("eyJ") {
        let start = pos + rel;
        let len = body[start..]
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            .count();
        let candidate = &body[start..start + len];
        if candidate.len() >= 100 && candidate.bytes().filter(|&b| b == b'.').count() == 2 {
            return Some(candidate);
        }
        pos = start + 3;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn make_jwt(exp_secs: u64) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"ES256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(format!(
            r#"{{"iss":"APPLEMUSIC","iat":1700000000,"exp":{exp_secs},"padding":"{}"}}"#,
            "x".repeat(60)
        ));
        let sig = URL_SAFE_NO_PAD.encode("signature-bytes-for-unit-test-only");
        format!("{header}.{payload}.{sig}")
    }

    #[test]
    fn parses_cookie_headers_and_bare_tokens() {
        assert_eq!(
            user_token_from_cookies("geo=US; media-user-token=AbCd123==; s_vi=1").unwrap(),
            "AbCd123=="
        );
        assert_eq!(
            user_token_from_cookies("media-user-token=XyZ987=").unwrap(),
            "XyZ987="
        );
        assert_eq!(user_token_from_cookies("  BareToken123==  ").unwrap(), "BareToken123==");
        assert!(user_token_from_cookies("geo=US; s_vi=1").is_err());
        assert!(user_token_from_cookies("media-user-token=").is_err());
        assert!(user_token_from_cookies("   ").is_err());
        assert!(user_token_from_cookies("bad\ntoken").is_err());
    }

    #[test]
    fn extracts_bundle_and_jwt() {
        let html = r#"<script type="module" src="/assets/index~9a8b7c6d.js"></script>"#;
        assert_eq!(extract_bundle_path(html), Some("/assets/index~9a8b7c6d.js"));
        assert_eq!(extract_bundle_path("<html></html>"), None);

        let jwt = make_jwt(2_000_000_000);
        assert!(jwt.len() >= 100);
        let snippet = format!(r#"var bad="eyJshort.a.b", ok="{jwt}";"#);
        assert_eq!(extract_jwt(&snippet), Some(jwt.as_str()));
        assert_eq!(jwt_exp(&jwt), Some(2_000_000_000));
    }

    #[test]
    fn disk_cache_respects_seven_day_margin() {
        let dir = std::env::temp_dir().join(format!(
            "musique-apple-auth-test-{}",
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let now = UNIX_EPOCH + Duration::from_secs(1_800_000_000);
        let fresh = make_jwt(1_800_000_000 + 10 * 86_400);
        save_disk_bearer(&dir, &fresh);
        assert_eq!(load_disk_bearer(&dir, now).as_deref(), Some(fresh.as_str()));

        let stale = make_jwt(1_800_000_000 + 3 * 86_400);
        save_disk_bearer(&dir, &stale);
        assert_eq!(load_disk_bearer(&dir, now), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn scrapes_jwt_from_bundle_when_absent_from_html() {
        let expected_jwt = make_jwt(2_000_000_000);
        let jwt_copy = expected_jwt.clone();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            for _ in 0..2 {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]);
                let path = req.split_whitespace().nth(1).unwrap_or("/");
                let (ctype, body) = if path == "/assets/index~abc123.js" {
                    (
                        "application/javascript",
                        format!(r#"const devToken="{jwt_copy}";"#),
                    )
                } else {
                    (
                        "text/html",
                        r#"<!doctype html><script src="/assets/index~abc123.js"></script>"#
                            .to_owned(),
                    )
                };
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let http = reqwest::Client::new();
        let got = scrape_bearer_from(&http, &format!("http://{addr}"))
            .await
            .unwrap();
        assert_eq!(got, expected_jwt);
        server.await.unwrap();
    }
}
