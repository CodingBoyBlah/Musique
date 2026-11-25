//! HTTP client and global session management for Apple Music's `amp-api` gateway.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::stream::{self, StreamExt as _, TryStreamExt as _};
use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH, ORIGIN, REFERER, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::apple::auth::{self, SAFARI_USER_AGENT};
use crate::apple::wire;
use crate::errors::AppError;

const AMP_API_BASE: &str = "https://amp-api.music.apple.com/v1";
const ACTIVITY_URL: &str = "https://universal-activity-service.itunes.apple.com/play";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_GET_ATTEMPTS: u32 = 3;
const RETRY_BASE: Duration = Duration::from_millis(400);
const RETRY_AFTER_CAP: Duration = Duration::from_secs(10);
const LISTING_TTL: Duration = Duration::from_secs(30);
const MAX_IN_FLIGHT_PAGES: usize = 10;

static CURRENT_CLIENT: RwLock<Option<AppleClient>> = RwLock::new(None);
static CONNECT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Signed-in Apple Music account summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppleAccount {
    pub storefront: String,
    pub display_name: Option<String>,
    pub handle: Option<String>,
    pub image_url: Option<String>,
}

struct CachedListing {
    fetched_at: Instant,
    pages: Arc<tokio::sync::OnceCell<Arc<Vec<String>>>>,
}

/// Authenticated HTTP client for `amp-api.music.apple.com`.
#[derive(Clone)]
pub struct AppleClient {
    http: reqwest::Client,
    user_token: Arc<str>,
    bearer: Arc<RwLock<String>>,
    bearer_refresh: Arc<tokio::sync::Mutex<()>>,
    storefront: Arc<str>,
    base_url: Arc<str>,
    stations: Arc<Mutex<HashMap<String, String>>>,
    listings: Arc<Mutex<HashMap<String, CachedListing>>>,
    #[cfg(test)]
    test_bearer_origin: Option<String>,
}

impl AppleClient {
    /// Connects to Apple Music with `user_token`, fetches the web bearer JWT if needed,
    /// and resolves the account storefront via `GET /me/storefront`.
    pub async fn connect(user_token: &str) -> Result<AppleClient, AppError> {
        let http = build_http_client()?;
        let bearer = auth::bearer(&http).await?;
        Self::connect_internal(http, user_token, bearer, AMP_API_BASE.to_owned(), None).await
    }

    #[cfg(test)]
    pub(crate) async fn connect_with_base(
        user_token: &str,
        initial_bearer: &str,
        api_base: &str,
        bearer_origin: Option<String>,
    ) -> Result<AppleClient, AppError> {
        let http = build_http_client()?;
        Self::connect_internal(
            http,
            user_token,
            initial_bearer.to_owned(),
            api_base.trim_end_matches('/').to_owned(),
            bearer_origin,
        )
        .await
    }

    async fn connect_internal(
        http: reqwest::Client,
        user_token: &str,
        initial_bearer: String,
        base_url: String,
        _test_bearer_origin: Option<String>,
    ) -> Result<AppleClient, AppError> {
        let clean_token = user_token.trim();
        if clean_token.is_empty() {
            return Err(AppError::Auth("Apple Music user token is empty".into()));
        }

        let mut client = Self {
            http,
            user_token: clean_token.into(),
            bearer: Arc::new(RwLock::new(initial_bearer)),
            bearer_refresh: Arc::new(tokio::sync::Mutex::new(())),
            storefront: "us".into(),
            base_url: base_url.into(),
            stations: Arc::new(Mutex::new(HashMap::new())),
            listings: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(test)]
            test_bearer_origin: _test_bearer_origin,
        };

        let body = client.get("/me/storefront", &[]).await?;
        let page = wire::parse_page(&body);
        let storefront = page
            .data
            .first()
            .and_then(|row| wire::str_field(row, "id"))
            .ok_or_else(|| {
                AppError::Auth("Apple Music returned no storefront for this account".into())
            })?;
        client.storefront = storefront.into();
        Ok(client)
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn bearer(&self) -> String {
        self.bearer
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub fn user_token(&self) -> &str {
        &self.user_token
    }

    pub fn storefront(&self) -> &str {
        &self.storefront
    }

    pub(crate) fn catalog_path(&self, suffix: &str) -> String {
        format!("/catalog/{}{suffix}", encode_segment(&self.storefront))
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<String, AppError> {
        self.request(Method::GET, path, query, None).await
    }

    pub async fn post(
        &self,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<String, AppError> {
        self.request(Method::POST, path, query, body).await
    }

    pub async fn patch(&self, path: &str, body: Value) -> Result<(), AppError> {
        self.request(Method::PATCH, path, &[], Some(body))
            .await
            .map(|_| ())
    }

    pub async fn delete(&self, path: &str, query: &[(&str, &str)]) -> Result<(), AppError> {
        self.request(Method::DELETE, path, query, None)
            .await
            .map(|_| ())
    }

    /// Sends a single play-activity beacon to Apple's activity service (`universal-activity-service`).
    pub(crate) async fn post_activity(&self, body: &Value) -> Result<(), AppError> {
        let resp = self
            .http
            .post(ACTIVITY_URL)
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer()))
            .header("Music-User-Token", self.user_token.as_ref())
            .header(ORIGIN, "https://music.apple.com")
            .header(REFERER, "https://music.apple.com/")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("cannot reach Apple play activity service: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            return Err(AppError::Network(format!(
                "Apple play activity returned {status}: {detail}"
            )));
        }
        Ok(())
    }

    /// Walks a paginated endpoint: fetches the first page to read `meta.total`, then fans out
    /// the remaining pages concurrently (bounded to `MAX_IN_FLIGHT_PAGES`), capped at `max_pages`.
    pub(crate) async fn get_pages(
        &self,
        path: &str,
        page_size: usize,
        max_pages: usize,
        query: &[(&str, &str)],
    ) -> Result<Vec<String>, AppError> {
        if max_pages == 0 {
            return Ok(Vec::new());
        }
        let limit_str = page_size.to_string();
        let mut first_query: Vec<(&str, &str)> = query.to_vec();
        first_query.push(("limit", &limit_str));

        let first_body = self.get(path, &first_query).await?;
        let first_parsed = wire::parse_page(&first_body);
        let got = first_parsed.data.len();
        let has_next = first_parsed.next.is_some();
        let total = first_parsed.total;

        let mut pages = vec![first_body];
        if got < page_size || !has_next || max_pages <= 1 {
            return Ok(pages);
        }

        let remaining_pages = match total {
            Some(total_count) => total_count
                .saturating_sub(got)
                .div_ceil(page_size)
                .min(max_pages - 1),
            None => max_pages - 1,
        };
        if remaining_pages == 0 {
            return Ok(pages);
        }

        let mut rest_stream = stream::iter(1..=remaining_pages)
            .map(|step| {
                let offset_str = (step * page_size).to_string();
                let limit_str = limit_str.clone();
                let base_query: Vec<(String, String)> = query
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect();
                async move {
                    let mut pairs: Vec<(&str, &str)> = base_query
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    pairs.push(("limit", limit_str.as_str()));
                    pairs.push(("offset", offset_str.as_str()));
                    self.get(path, &pairs).await
                }
            })
            .buffered(MAX_IN_FLIGHT_PAGES);

        while let Some(body) = rest_stream.try_next().await? {
            let parsed = wire::parse_page(&body);
            let rows = parsed.data.len();
            let more = parsed.next.is_some();
            pages.push(body);
            if rows < page_size || !more {
                break;
            }
        }

        Ok(pages)
    }

    /// Cached paged listing (30 s TTL, invalidated on any write). Prevents redundant fetches when
    /// callers read the same library listing back-to-back.
    pub(crate) async fn get_listing(
        &self,
        path: &str,
        page_size: usize,
        max_pages: usize,
        query: &[(&str, &str)],
    ) -> Result<Arc<Vec<String>>, AppError> {
        let key = format!("{path}|{page_size}|{max_pages}|{query:?}");
        let cell = {
            let mut guard = self.listings.lock().unwrap_or_else(|p| p.into_inner());
            guard.retain(|_, entry| entry.fetched_at.elapsed() < LISTING_TTL);
            guard
                .entry(key)
                .or_insert_with(|| CachedListing {
                    fetched_at: Instant::now(),
                    pages: Arc::default(),
                })
                .pages
                .clone()
        };
        cell.get_or_try_init(|| async {
            self.get_pages(path, page_size, max_pages, query)
                .await
                .map(Arc::new)
        })
        .await
        .cloned()
    }

    pub(crate) fn invalidate_listings(&self) {
        if let Ok(mut guard) = self.listings.lock() {
            guard.clear();
        }
    }

    pub(crate) fn station_for_seed(&self, seed: &str) -> Option<String> {
        self.stations
            .lock()
            .ok()?
            .get(seed)
            .cloned()
    }

    pub(crate) fn remember_station(&self, seed: &str, station_id: &str) {
        if let Ok(mut guard) = self.stations.lock() {
            if guard.len() >= 64 && !guard.contains_key(seed) {
                guard.clear();
            }
            guard.insert(seed.to_owned(), station_id.to_owned());
        }
    }

    pub(crate) async fn is_subscribed(&self) -> Result<Option<bool>, AppError> {
        let body = self.get("/me/account", &[("meta", "subscription")]).await?;
        let val: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        Ok(val
            .pointer("/meta/subscription/active")
            .and_then(Value::as_bool))
    }

    pub(crate) async fn account_profile(&self) -> Result<AppleAccount, AppError> {
        let body = match self.get("/me/social-profile", &[]).await {
            Ok(b) => Some(b),
            Err(AppError::NotFound(_)) => None,
            Err(e) => {
                eprintln!("[apple] /me/social-profile unavailable, falling back: {e}");
                None
            }
        };
        let (display_name, handle, image_url) = match body.as_deref() {
            Some(text) => {
                let page = wire::parse_page(text);
                let attrs = page.data.first().and_then(wire::attrs_of);
                (
                    attrs
                        .and_then(|a| wire::str_field(a, "name"))
                        .map(str::to_owned),
                    attrs
                        .and_then(|a| wire::str_field(a, "handle"))
                        .map(str::to_owned),
                    attrs.and_then(|a| wire::artwork_url(a, wire::ART_SIZE)),
                )
            }
            None => (None, None, None),
        };
        Ok(AppleAccount {
            storefront: self.storefront.to_string(),
            display_name,
            handle,
            image_url,
        })
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<String, AppError> {
        let is_get = method == Method::GET;
        let max_attempts = if is_get { MAX_GET_ATTEMPTS } else { 1 };
        let mut attempt = 0u32;
        let mut retried_auth = false;

        loop {
            let sent_bearer = self.bearer();
            match self
                .send_once(&method, path, query, body.as_ref(), &sent_bearer)
                .await
            {
                Ok(text) => {
                    if !is_get {
                        self.invalidate_listings();
                    }
                    return Ok(text);
                }
                Err(SendError::Unauthorized(err)) if !retried_auth => {
                    retried_auth = true;
                    if let Err(refresh_err) = self.refresh_bearer(&sent_bearer).await {
                        return Err(AppError::Auth(format!("{err}; bearer refresh failed: {refresh_err}")));
                    }
                }
                Err(SendError::Unauthorized(err)) => {
                    return Err(err);
                }
                Err(SendError::Transient { err, retry_after }) if is_get && attempt + 1 < max_attempts => {
                    attempt += 1;
                    let wait = retry_after.unwrap_or_else(|| backoff_delay(attempt - 1));
                    tokio::time::sleep(wait).await;
                    let _ = err;
                }
                Err(SendError::Transient { err, .. }) | Err(SendError::Fatal(err)) => {
                    if !is_get {
                        self.invalidate_listings();
                    }
                    return Err(err);
                }
            }
        }
    }

    async fn refresh_bearer(&self, failed_bearer: &str) -> Result<(), AppError> {
        let _lock = self.bearer_refresh.lock().await;
        if self.bearer() != failed_bearer {
            return Ok(());
        }
        #[cfg(test)]
        let fresh = if let Some(ref origin) = self.test_bearer_origin {
            auth::scrape_bearer_from_test(&self.http, origin).await?
        } else {
            auth::forget_bearer();
            auth::bearer(&self.http).await?
        };
        #[cfg(not(test))]
        let fresh = {
            auth::forget_bearer();
            auth::bearer(&self.http).await?
        };
        if let Ok(mut guard) = self.bearer.write() {
            *guard = fresh;
        }
        Ok(())
    }

    async fn send_once(
        &self,
        method: &Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&Value>,
        bearer: &str,
    ) -> Result<String, SendError> {
        let url = self.build_url(path);
        let mut req = self
            .http
            .request(method.clone(), &url)
            .header(AUTHORIZATION, format!("Bearer {bearer}"))
            .header("Music-User-Token", self.user_token.as_ref())
            .header(ORIGIN, "https://music.apple.com")
            .header(REFERER, "https://music.apple.com/")
            .query(query);

        req = match body {
            Some(payload) => req.json(payload),
            None => {
                if *method != Method::GET {
                    req.header(CONTENT_LENGTH, "0")
                } else {
                    req
                }
            }
        };

        let resp = req.send().await.map_err(|e| SendError::Transient {
            err: AppError::Network(format!("cannot reach Apple Music ({path}): {e}")),
            retry_after: None,
        })?;

        let status = resp.status();
        let retry_after = parse_retry_after(resp.headers());

        let bytes = resp.bytes().await.map_err(|e| SendError::Transient {
            err: AppError::Network(format!("interrupted Apple Music response ({path}): {e}")),
            retry_after,
        })?;
        let text = String::from_utf8_lossy(&bytes).into_owned();

        if status.is_success() {
            return Ok(text);
        }

        let detail = extract_error_detail(&text);
        let msg = format!("Apple Music {status} on {method} {path}: {detail}");

        if status == StatusCode::UNAUTHORIZED {
            return Err(SendError::Unauthorized(AppError::Auth(msg)));
        }
        if status == StatusCode::FORBIDDEN {
            return Err(SendError::Fatal(AppError::Auth(msg)));
        }
        if status == StatusCode::NOT_FOUND {
            return Err(SendError::Fatal(AppError::NotFound(msg)));
        }
        if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            return Err(SendError::Transient {
                err: AppError::Network(msg),
                retry_after,
            });
        }
        Err(SendError::Fatal(AppError::Network(msg)))
    }

    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_owned();
        }
        let clean = path.strip_prefix("/v1").unwrap_or(path);
        if clean.starts_with('/') {
            format!("{}{clean}", self.base_url)
        } else {
            format!("{}/{clean}", self.base_url)
        }
    }
}

enum SendError {
    Unauthorized(AppError),
    Transient {
        err: AppError,
        retry_after: Option<Duration>,
    },
    Fatal(AppError),
}

fn build_http_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .user_agent(SAFARI_USER_AGENT)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .gzip(true)
        .brotli(true)
        .build()
        .map_err(|e| AppError::Network(format!("cannot build Apple HTTP client: {e}")))
}

/// Percent-encodes a single URL path segment per RFC 3986 unreserved characters.
pub(crate) fn encode_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

fn extract_error_detail(body: &str) -> String {
    let Ok(val) = serde_json::from_str::<Value>(body) else {
        return "no detail".to_owned();
    };
    val.pointer("/errors/0/detail")
        .or_else(|| val.pointer("/errors/0/title"))
        .and_then(Value::as_str)
        .unwrap_or("no detail")
        .to_owned()
}

fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let raw = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    let secs = raw.parse::<u64>().ok()?;
    Some(Duration::from_secs(secs.max(1)).min(RETRY_AFTER_CAP))
}

fn backoff_delay(attempt_idx: u32) -> Duration {
    let base = RETRY_BASE * 2u32.saturating_pow(attempt_idx);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    base + base.mul_f32(nanos as f32 / 1e9)
}

/// Returns the cached global [`AppleClient`], lazily connecting from the stored keyring token.
pub async fn current() -> Result<AppleClient, AppError> {
    if let Some(client) = CURRENT_CLIENT
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
    {
        return Ok(client);
    }

    let _guard = CONNECT_LOCK.lock().await;
    if let Some(client) = CURRENT_CLIENT
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
    {
        return Ok(client);
    }

    let user_token = auth::load_user_token()
        .ok_or_else(|| AppError::Auth("Not signed in to Apple Music".into()))?;
    let client = AppleClient::connect(&user_token).await?;
    *CURRENT_CLIENT.write().unwrap_or_else(|p| p.into_inner()) = Some(client.clone());
    Ok(client)
}

/// Clears the cached global client and removes the stored `media-user-token` (sign-out).
pub fn reset() {
    crate::apple::library::bump_session_epoch();
    if let Ok(mut guard) = CURRENT_CLIENT.write() {
        *guard = None;
    }
    auth::clear_user_token();
}

/// Signs into Apple Music from a cookie header or bare `media-user-token`, verifies an active
/// subscription, persists the token, caches the global client, and returns the account profile.
pub async fn sign_in(cookie_or_token: &str) -> Result<AppleAccount, AppError> {
    let token = auth::user_token_from_cookies(cookie_or_token)?;
    let client = AppleClient::connect(&token).await?;

    if client.is_subscribed().await? == Some(false) {
        return Err(AppError::Auth(
            "This Apple ID has no active Apple Music subscription".into(),
        ));
    }

    let account = client.account_profile().await?;
    auth::store_user_token(&token)?;

    let _guard = CONNECT_LOCK.lock().await;
    crate::apple::library::bump_session_epoch();
    *CURRENT_CLIENT.write().unwrap_or_else(|p| p.into_inner()) = Some(client);
    Ok(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn sample_jwt(tag: &str) -> String {
        let h = URL_SAFE_NO_PAD.encode(r#"{"alg":"ES256"}"#);
        let p = URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":2000000000,"tag":"{tag}","pad":"{}"}}"#, "z".repeat(60)));
        let s = URL_SAFE_NO_PAD.encode("sig");
        format!("{h}.{p}.{s}")
    }

    #[tokio::test]
    async fn retries_401_once_with_refreshed_bearer_and_retries_5xx_429_gets() {
        let stale_jwt = sample_jwt("stale");
        let fresh_jwt = sample_jwt("fresh");
        let fresh_copy = fresh_jwt.clone();

        // Mock music.apple.com for bearer re-scrape
        let bearer_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bearer_addr = bearer_listener.local_addr().unwrap();
        let bearer_server = tokio::spawn(async move {
            let (mut sock, _) = bearer_listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf).await.unwrap();
            let body = format!(r#"<html><script>const t="{fresh_copy}";</script></html>"#);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            sock.write_all(resp.as_bytes()).await.unwrap();
        });

        // Mock amp-api server
        let api_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let api_addr = api_listener.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_clone = hits.clone();
        let fresh_for_api = fresh_jwt.clone();

        let api_server = tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = api_listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).into_owned();
                let line = req.lines().next().unwrap_or("");
                let hit = hits_clone.fetch_add(1, Ordering::SeqCst);

                let (status, extra_hdrs, body) = if line.starts_with("GET /v1/me/storefront") {
                    (
                        "200 OK",
                        "",
                        r#"{"data":[{"id":"us"}]}"#,
                    )
                } else if line.starts_with("GET /v1/me/test-retry") {
                    if !req.contains(&format!("Bearer {fresh_for_api}")) {
                        ("401 Unauthorized", "", r#"{"errors":[{"detail":"expired"}]}"#)
                    } else if hit < 3 {
                        ("429 Too Many Requests", "Retry-After: 1\r\n", "{}")
                    } else {
                        ("200 OK", "", r#"{"data":[{"id":"ok"}]}"#)
                    }
                } else if line.starts_with("POST /v1/me/write-once") {
                    assert!(
                        req.to_ascii_lowercase().contains("content-length: 0"),
                        "bodyless write must carry Content-Length: 0"
                    );
                    ("503 Service Unavailable", "", "{}")
                } else {
                    ("404 Not Found", "", "{}")
                };

                let reply = format!(
                    "HTTP/1.1 {status}\r\n{extra_hdrs}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let client = AppleClient::connect_with_base(
            "user-tok",
            &stale_jwt,
            &format!("http://{api_addr}/v1"),
            Some(format!("http://{bearer_addr}")),
        )
        .await
        .unwrap();
        assert_eq!(client.storefront(), "us");

        let ok_body = client.get("/me/test-retry", &[]).await.unwrap();
        assert!(ok_body.contains("ok"));
        assert_eq!(client.bearer(), fresh_jwt);

        let before_write = hits.load(Ordering::SeqCst);
        let write_err = client.post("/me/write-once", &[], None).await.unwrap_err();
        assert!(matches!(write_err, AppError::Network(_)));
        assert_eq!(
            hits.load(Ordering::SeqCst),
            before_write + 1,
            "writes must be sent exactly once without 5xx retry"
        );

        api_server.abort();
        bearer_server.await.unwrap();
    }

    #[tokio::test]
    async fn fans_out_paginated_requests_using_meta_total() {
        let jwt = sample_jwt("paging");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let seen_offsets = Arc::new(Mutex::new(Vec::<String>::new()));
        let seen_clone = seen_offsets.clone();

        let server = tokio::spawn(async move {
            for _ in 0..4 {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]);
                let path = req.split_whitespace().nth(1).unwrap_or("");
                let body = if path.starts_with("/v1/me/storefront") {
                    r#"{"data":[{"id":"us"}]}"#.to_owned()
                } else if path.contains("offset=2") {
                    seen_clone.lock().unwrap().push("2".into());
                    r#"{"meta":{"total":5},"next":"/v1/me/items?offset=4","data":[{"id":"3"},{"id":"4"}]}"#.to_owned()
                } else if path.contains("offset=4") {
                    seen_clone.lock().unwrap().push("4".into());
                    r#"{"meta":{"total":5},"next":null,"data":[{"id":"5"}]}"#.to_owned()
                } else {
                    seen_clone.lock().unwrap().push("0".into());
                    r#"{"meta":{"total":5},"next":"/v1/me/items?offset=2","data":[{"id":"1"},{"id":"2"}]}"#.to_owned()
                };
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let client = AppleClient::connect_with_base(
            "user-tok",
            &jwt,
            &format!("http://{addr}/v1"),
            None,
        )
        .await
        .unwrap();

        let pages = client.get_pages("/me/items", 2, 10, &[]).await.unwrap();
        assert_eq!(pages.len(), 3);
        let mut offsets = seen_offsets.lock().unwrap().clone();
        offsets.sort();
        assert_eq!(offsets, vec!["0", "2", "4"]);
        server.await.unwrap();
    }
}
