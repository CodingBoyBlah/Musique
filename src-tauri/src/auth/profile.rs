use super::{get_setting_value, now_ms};
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::{Mutex, OnceLock}, time::{Duration, Instant}};

fn rate_limits() -> &'static Mutex<HashMap<[u8; 32], Instant>> {
    static LIMITS: OnceLock<Mutex<HashMap<[u8; 32], Instant>>> = OnceLock::new();
    LIMITS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize)]
pub struct SpotifyProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
    pub images: Option<Vec<SpotifyImage>>,
    pub followers: Option<Followers>,
    pub external_urls: Option<ExternalUrls>,
    pub explicit_content: Option<ExplicitContent>,
}

impl SpotifyProfile {
    pub fn from_session_view(username: &str, view: &serde_json::Value) -> Self {
        Self {
            id: username.to_string(),
            display_name: view.get("name").and_then(|s| s.as_str()).map(str::to_string),
            images: crate::commands::profile::image(view.get("image_url").and_then(|v| v.as_str()).map(str::to_string))
                .map(|url| vec![SpotifyImage { url }]),
            followers: view.get("followers_count").and_then(|v| v.as_i64()).map(|total| Followers { total }),
            email: None, country: None, product: None, external_urls: None, explicit_content: None,
        }
    }
    pub fn needs_details(&self) -> bool {
        self.product.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) || self.images.is_none()
    }

    pub fn fill_missing(&mut self, extra: Self) {
        if self.id != extra.id { return; }
        if self.product.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) { self.product = extra.product; }
        if self.images.is_none() { self.images = extra.images; }
        if self.display_name.is_none() { self.display_name = extra.display_name; }
        if self.email.is_none() { self.email = extra.email; }
        if self.country.is_none() { self.country = extra.country; }
        if self.followers.is_none() { self.followers = extra.followers; }
        if self.external_urls.is_none() { self.external_urls = extra.external_urls; }
        if self.explicit_content.is_none() { self.explicit_content = extra.explicit_content; }
    }

    pub fn fill_session_plan(&mut self, username: &str, account_type: Option<&str>) {
        // ProductInfo can report Free incorrectly for some grants. Only its
        // positive Premium entitlement is useful when /me omits the plan.
        if self.id == username && self.product.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
            && account_type == Some("premium") {
            self.product = Some("premium".into());
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpotifyImage { pub url: String }
#[derive(Debug, Deserialize)]
pub struct Followers { pub total: i64 }
#[derive(Debug, Deserialize)]
pub struct ExternalUrls { pub spotify: Option<String> }
#[derive(Debug, Deserialize)]
pub struct ExplicitContent { pub filter_enabled: bool, pub filter_locked: bool }

#[derive(Debug, Serialize)]
pub struct Profile {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub product: Option<String>,
    pub followers: i64,
    pub image_url: Option<String>,
    pub spotify_url: Option<String>,
    pub explicit_filter_enabled: bool,
    pub explicit_filter_locked: bool,
}

pub async fn fetch_profile(token: &str) -> Result<SpotifyProfile, AppError> {
    fetch_profile_at(token, "https://api.spotify.com/v1/me").await
}

async fn fetch_profile_at(token: &str, url: &str) -> Result<SpotifyProfile, AppError> {
    let key: [u8; 32] = Sha256::digest(token.as_bytes()).into();
    {
        let mut limits = rate_limits().lock().unwrap_or_else(|e| e.into_inner());
        limits.retain(|_, until| *until > Instant::now());
        if limits.contains_key(&key) {
            return Err(AppError::Network("Spotify profile is rate-limited; waiting before retrying".into()));
        }
    }
    let resp = crate::http::client().get(url)
        .bearer_auth(token).send().await?;
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let wait = resp.headers().get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(60);
        if let Some(until) = Instant::now().checked_add(Duration::from_secs(wait.max(1))) {
            let mut limits = rate_limits().lock().unwrap_or_else(|e| e.into_inner());
            if limits.len() >= 16 { limits.clear(); }
            limits.insert(key, until);
        }
        return Err(AppError::Network(format!("Spotify profile is rate-limited; retry after {wait} seconds")));
    }
    if !resp.status().is_success() {
        return Err(AppError::Auth(format!("Profile fetch failed: {}", resp.status())));
    }
    Ok(resp.json().await?)
}

pub async fn fetch_profile_retrying(token: &str) -> Result<SpotifyProfile, AppError> {
    match fetch_profile(token).await {
        Ok(p) => Ok(p),
        Err(first) => {
            eprintln!("[auth] profile fetch failed ({first}); retrying once");
            tokio::time::sleep(std::time::Duration::from_millis(600)).await;
            fetch_profile(token).await
        }
    }
}

pub const PROFILE_KEYS: &[&str] = &[
    "spotify_user_id", "spotify_display_name", "spotify_email", "spotify_country",
    "spotify_product", "spotify_image_url", "spotify_followers",
    "spotify_explicit_filter", "spotify_explicit_filter_locked", "spotify_profile_url",
];

// A missing deprecated field is unknown. Keep a known value for the same
// account, but never carry it across an account switch. An empty image list
// explicitly removes an avatar; an omitted list does not.
pub async fn cache_profile(pool: &SqlitePool, p: &SpotifyProfile) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let previous: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = 'spotify_user_id'")
        .fetch_optional(&mut *tx).await?;
    if previous.as_ref().map(|r| r.0.as_str()) != Some(p.id.as_str()) {
        for key in PROFILE_KEYS {
            sqlx::query("DELETE FROM settings WHERE key = ?").bind(key).execute(&mut *tx).await?;
        }
    }
    let image = p.images.as_ref().map(|images| images.iter()
        .find(|im| !im.url.trim().is_empty()).map(|im| im.url.as_str()).unwrap_or(""));
    let product = p.product.as_ref().map(|s| s.trim().to_ascii_lowercase());
    let followers = p.followers.as_ref().map(|f| f.total.to_string());
    let enabled = p.explicit_content.as_ref().map(|e| if e.filter_enabled { "1" } else { "0" });
    let locked = p.explicit_content.as_ref().map(|e| if e.filter_locked { "1" } else { "0" });
    for (key, value) in [
        ("spotify_user_id", Some(p.id.as_str())),
        ("spotify_display_name", p.display_name.as_deref()),
        ("spotify_email", p.email.as_deref()),
        ("spotify_country", p.country.as_deref()),
        ("spotify_product", product.as_deref().filter(|v| !v.is_empty())),
        ("spotify_image_url", image),
        ("spotify_followers", followers.as_deref()),
        ("spotify_explicit_filter", enabled),
        ("spotify_explicit_filter_locked", locked),
        ("spotify_profile_url", p.external_urls.as_ref().and_then(|u| u.spotify.as_deref())),
    ] {
        if let Some(value) = value {
            sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at")
                .bind(key).bind(value).bind(now_ms()).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn cached_profile(pool: &SqlitePool) -> Result<Profile, AppError> {
    async fn value(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
        Ok(get_setting_value(pool, key).await?.filter(|s| !s.is_empty()))
    }
    let id = value(pool, "spotify_user_id").await?;
    Ok(Profile {
        spotify_url: value(pool, "spotify_profile_url").await?
            .or_else(|| id.as_ref().map(|id| format!("https://open.spotify.com/user/{id}"))),
        id,
        display_name: value(pool, "spotify_display_name").await?,
        email: value(pool, "spotify_email").await?,
        country: value(pool, "spotify_country").await?,
        product: value(pool, "spotify_product").await?,
        image_url: value(pool, "spotify_image_url").await?,
        followers: value(pool, "spotify_followers").await?.and_then(|v| v.parse().ok()).unwrap_or(0),
        explicit_filter_enabled: value(pool, "spotify_explicit_filter").await?.as_deref() == Some("1"),
        explicit_filter_locked: value(pool, "spotify_explicit_filter_locked").await?.as_deref() == Some("1"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{build_auth_status, init_auth_state_with, upsert_setting};

    async fn db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL)")
            .execute(&pool).await.unwrap();
        pool
    }
    fn profile(json: &str) -> SpotifyProfile { serde_json::from_str(json).unwrap() }

    #[tokio::test]
    async fn profile_recovery_updates_auth_and_playback_together() {
        let pool = db().await;
        cache_profile(&pool, &profile(r#"{"id":"a"}"#)).await.unwrap();
        assert!(build_auth_status(&pool, true).await.unwrap().product.is_none());
        cache_profile(&pool, &profile(r#"{"id":"a","display_name":"Alice","product":"Premium","images":[{"url":""},{"url":"https://example.com/avatar"}]}"#)).await.unwrap();
        let auth = build_auth_status(&pool, true).await.unwrap();
        assert_eq!(auth.product.as_deref(), Some("premium"));
        assert_eq!(auth.image_url.as_deref(), Some("https://example.com/avatar"));
        assert!(crate::commands::playback::spotify_available(&pool).await);
        assert_eq!(cached_profile(&pool).await.unwrap().display_name, auth.display_name);
    }

    #[tokio::test]
    async fn omitted_fields_keep_same_account_values_but_not_another_accounts() {
        let pool = db().await;
        cache_profile(&pool, &profile(r#"{"id":"a","product":"premium","images":[{"url":"https://example.com/a"}]}"#)).await.unwrap();
        cache_profile(&pool, &profile(r#"{"id":"a"}"#)).await.unwrap();
        assert_eq!(cached_profile(&pool).await.unwrap().product.as_deref(), Some("premium"));
        cache_profile(&pool, &profile(r#"{"id":"b"}"#)).await.unwrap();
        let b = cached_profile(&pool).await.unwrap();
        assert!(b.product.is_none());
        assert!(b.image_url.is_none());
        assert!(crate::commands::playback::spotify_available(&pool).await);
    }

    #[tokio::test]
    async fn confirmed_free_and_open_plans_are_distinct_from_unknown() {
        let pool = db().await;
        assert!(crate::commands::playback::spotify_available(&pool).await);
        for product in ["free", "open"] {
            upsert_setting(&pool, "spotify_product", product).await.unwrap();
            assert!(!crate::commands::playback::spotify_available(&pool).await);
        }
    }

    #[tokio::test]
    async fn explicit_empty_images_remove_cached_avatar() {
        let pool = db().await;
        cache_profile(&pool, &profile(r#"{"id":"a","images":[{"url":"https://example.com/a"}]}"#)).await.unwrap();
        cache_profile(&pool, &profile(r#"{"id":"a","images":[]}"#)).await.unwrap();
        assert!(build_auth_status(&pool, true).await.unwrap().image_url.is_none());
    }

    #[test]
    fn fallback_cannot_import_another_accounts_plan_or_avatar() {
        let mut main = profile(r#"{"id":"a"}"#);
        main.fill_missing(profile(r#"{"id":"b","product":"premium","images":[{"url":"https://example.com/b"}]}"#));
        assert!(main.product.is_none());
        assert!(main.images.is_none());
        main.fill_missing(profile(r#"{"id":"a","product":"premium","images":[{"url":"https://example.com/a"}]}"#));
        assert_eq!(main.product.as_deref(), Some("premium"));
        assert_eq!(main.images.unwrap()[0].url, "https://example.com/a");
    }

    #[test]
    fn session_plan_requires_a_matching_account_and_positive_entitlement() {
        let mut p = profile(r#"{"id":"a"}"#);
        p.fill_session_plan("b", Some("premium"));
        assert!(p.product.is_none());
        p.fill_session_plan("a", Some("free"));
        assert!(p.product.is_none());
        p.fill_session_plan("a", Some("premium"));
        assert_eq!(p.product.as_deref(), Some("premium"));
        let mut free = profile(r#"{"id":"a","product":"free"}"#);
        free.fill_session_plan("a", Some("premium"));
        assert_eq!(free.product.as_deref(), Some("free"));
    }

    #[test]
    fn internal_profile_recovers_avatar_and_premium_without_web_api_fields() {
        let view = serde_json::json!({"name":"Alice", "image_url":"spotify:image:abc123", "followers_count":12});
        let mut p = SpotifyProfile::from_session_view("a", &view);
        p.fill_session_plan("a", Some("premium"));
        assert_eq!(p.display_name.as_deref(), Some("Alice"));
        assert_eq!(p.product.as_deref(), Some("premium"));
        assert_eq!(p.images.unwrap()[0].url, "https://i.scdn.co/image/abc123");
        assert_eq!(p.followers.unwrap().total, 12);
    }

    #[tokio::test]
    async fn rate_limit_honors_retry_after_without_blocking_another_grant() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/me", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            for response in [
                "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 120\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                {
                    let body = r#"{"id":"a","product":"premium"}"#;
                    format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
                },
            ] {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = [0u8; 2048];
                socket.read(&mut buffer).await.unwrap();
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let first = fetch_profile_at("rate-limited-test-grant", &url).await.unwrap_err();
        assert!(first.to_string().contains("retry after 120 seconds"));
        let other = fetch_profile_at("first-party-test-grant", &url).await.unwrap();
        assert_eq!(other.product.as_deref(), Some("premium"));
        let retry = fetch_profile_at("rate-limited-test-grant", &url).await.unwrap_err();
        assert!(retry.to_string().contains("waiting before retrying"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn restart_after_logout_does_not_load_undeleted_keyring_tokens() {
        let pool = db().await;
        upsert_setting(&pool, "spotify_signed_out", "1").await.unwrap();
        let state = init_auth_state_with(&pool, Some(("old-access".into(), "old-refresh".into()))).await;
        assert!(state.access_token.is_none());
        assert!(state.refresh_token.is_none());
        upsert_setting(&pool, "spotify_signed_out", "0").await.unwrap();
        let state = init_auth_state_with(&pool, Some(("new-access".into(), "new-refresh".into()))).await;
        assert_eq!(state.access_token.as_deref(), Some("new-access"));
    }
}
