pub mod http;
pub mod pkce;
pub mod token;
pub mod profile;

use profile::{cache_profile, fetch_profile_retrying};

use crate::errors::AppError;
use crate::state::AuthState;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

// public types n stuff

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthStatus {
    pub logged_in:    bool,
    pub user_id:      Option<String>,
    pub display_name: Option<String>,
    pub email:        Option<String>,
    pub product:      Option<String>,
    pub image_url:    Option<String>,
}

// public shared client ID (ncspot / spotify-player public Web API application)
pub const SHARED_CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
// official Spotify desktop client ID (playback grant with full streaming capabilities)
pub const PLAYBACK_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
// persistent device ID derived from device name: hex(&Sha1::digest(device_name.as_bytes()))
pub const DEFAULT_DEVICE_NAME: &str = "Musique";
pub const PLAYBACK_DEVICE_ID: &str = "fbdc061e6e145fffdaac1e916106ec81f4503c13";
pub const PLAYBACK_SCOPES: &str = "app-remote-control streaming user-modify-playback-state user-read-currently-playing user-read-playback-state user-read-private";

// ─── auth epoch ──────────────────────────────────────────────────────────────
//
// Bumped every time the signed-in identity changes (login, logout, account
// switch). Long-running background work that writes account-scoped rows - the
// library sync above all - captures the epoch when it starts and stops writing
// as soon as it changes. Without this, a sync that was already in flight when
// you signed out happily re-inserted the old account's playlists *after*
// logout had purged them, which is why signing out looked like it did nothing.
static SESSION_COMMIT: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static TOKEN_REFRESH: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static ACCOUNT_CLEANUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) async fn account_cleanup() -> tokio::sync::MutexGuard<'static, ()> {
    ACCOUNT_CLEANUP.lock().await
}

pub(crate) async fn session_commit() -> tokio::sync::MutexGuard<'static, ()> {
    SESSION_COMMIT.lock().await
}

pub(crate) fn require_epoch(epoch: u64) -> Result<(), AppError> {
    if epoch_is_current(epoch) { Ok(()) }
    else { Err(AppError::Auth("Account changed; request cancelled".into())) }
}

static AUTH_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn auth_epoch() -> u64 {
    AUTH_EPOCH.load(std::sync::atomic::Ordering::SeqCst)
}

/// Invalidate everything that belongs to the previous identity.
pub fn bump_auth_epoch() -> u64 {
    AUTH_EPOCH.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}

/// True while `epoch` still describes the signed-in identity.
pub fn epoch_is_current(epoch: u64) -> bool {
    auth_epoch() == epoch
}

/// Settings rows that belong to the signed-in Spotify account and must not
/// survive a logout or an account switch.
///
/// Deliberately a denylist: everything else in `settings` describes the *device
/// or the app* - volume, audio quality, cache size, device name, the user's own
/// Spotify application credentials - and wiping those on logout was itself a
/// bug. Clearing `spotify_client_id` in particular left the keyring mirror
/// behind, so the UI kept showing a custom app while logins silently used the
/// shared one, and the resulting refresh tokens came back `invalid_client`.
pub const ACCOUNT_SETTING_KEYS: &[&str] = &[
    "spotify_user_id",
    "spotify_display_name",
    "spotify_email",
    "spotify_product",
    "spotify_image_url",
    "spotify_country",
    "spotify_followers",
    "spotify_explicit_filter",
    "spotify_explicit_filter_locked",
    "spotify_profile_url",
    "spotify_token_expires_at",
    "spotify_playback_token",
    "spotify_auth_client_id",
    "library_last_synced",
];
// last.fm is deliberately absent: it is a separate account the user linked
// themselves, its session key lives in the keyring rather than here, and
// clearing only the settings half of it would leave scrobbling live while the
// UI claimed it was disconnected. It has its own disconnect in Settings.

/// The subset of `ACCOUNT_SETTING_KEYS` that describes the *current* session
/// rather than the account's cached data. Kept when a fresh login has already
/// written the new account's rows and we are only clearing out the old one's.
pub const SESSION_SETTING_KEYS: &[&str] = &[
    "spotify_user_id",
    "spotify_display_name",
    "spotify_email",
    "spotify_product",
    "spotify_image_url",
    "spotify_country",
    "spotify_followers",
    "spotify_explicit_filter",
    "spotify_explicit_filter_locked",
    "spotify_profile_url",
    "spotify_token_expires_at",
    "spotify_auth_client_id",
];

pub fn compute_device_id(device_name: &str) -> String {
    use sha1::{Digest, Sha1};
    let hash = Sha1::digest(device_name.as_bytes());
    let mut out = String::with_capacity(hash.len() * 2);
    for byte in hash {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub async fn get_active_client_id(pool: &SqlitePool) -> String {
    if let Ok(Some(id)) = get_setting_value(pool, "spotify_client_id").await {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    SHARED_CLIENT_ID.to_string()
}

// db helper junk

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub(crate) async fn get_setting_value(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get::<String, _>("value")))
}

pub(crate) async fn upsert_setting(
    pool: &SqlitePool,
    key: &str,
    value: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

// startup init, runs once when app boots

/// Read the token pair out of the OS credential store. Blocking (it talks to
/// the Windows Credential Manager / Keychain / Secret Service), so startup runs
/// it on a blocking thread alongside the sqlite open rather than after it.
pub fn load_stored_tokens() -> Option<(String, String)> {
    token::load_tokens().ok().flatten()
}

pub async fn init_auth_state(pool: &SqlitePool) -> AuthState {
    init_auth_state_with(pool, load_stored_tokens()).await
}

pub async fn init_auth_state_with(
    pool: &SqlitePool,
    tokens: Option<(String, String)>,
) -> AuthState {
    // A failed credential-store delete must not resurrect a signed-out session.
    if get_setting_value(pool, "spotify_signed_out").await.ok().flatten().as_deref() == Some("1") {
        return AuthState::default();
    }
    let (access_token, refresh_token) = match tokens {
        Some(pair) => (Some(pair.0), Some(pair.1)),
        None => return AuthState::default(),
    };

    let expires_at = get_setting_value(pool, "spotify_token_expires_at")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<i64>().ok());

    AuthState { access_token, refresh_token, expires_at }
}

// token access, only refreshes when its actually expired (lazy)

pub async fn get_valid_token(
    pool: &SqlitePool,
    auth: &RwLock<AuthState>,
) -> Result<String, AppError> {
    let _refresh = TOKEN_REFRESH.lock().await;
    {
        let g = auth.read().await;
        if let (Some(tok), Some(exp)) = (&g.access_token, g.expires_at) {
            // 90-second refresh margin to avoid race conditions mid-request
            if exp - now_ms() > 90_000 {
                return Ok(tok.clone());
            }
        }
    }
    do_refresh(pool, auth).await
}

async fn do_refresh(pool: &SqlitePool, auth: &RwLock<AuthState>) -> Result<String, AppError> {
    let (epoch, client_id, refresh_token) = {
        let _commit = session_commit().await;
        let epoch = auth_epoch();
        let refresh_token = auth.read().await.refresh_token.clone()
            .ok_or_else(|| AppError::Auth("Not logged in".into()))?;
        let client_id = match get_setting_value(pool, "spotify_auth_client_id").await? {
            Some(cid) if !cid.trim().is_empty() && cid.trim() != PLAYBACK_CLIENT_ID => cid,
            _ => get_active_client_id(pool).await,
        };
        (epoch, client_id, refresh_token)
    };

    let resp = match call_token_endpoint(&[
        ("grant_type",    "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id",     &client_id),
    ])
    .await
    {
        Ok(r) => r,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("invalid_grant") || err_str.contains("invalid_client") {
                let _commit = session_commit().await;
                require_epoch(epoch)?;
                upsert_setting(pool, "spotify_signed_out", "1").await?;
                bump_auth_epoch();
                let _ = token::clear_tokens();
                *auth.write().await = AuthState::default();
                return Err(AppError::Auth(
                    "Your Spotify session expired or was reset. Please sign in again.".into(),
                ));
            }
            return Err(e);
        }
    };

    commit_refresh(pool, auth, epoch, resp).await
}

async fn commit_refresh(
    pool: &SqlitePool, auth: &RwLock<AuthState>, epoch: u64, resp: token::TokenResponse,
) -> Result<String, AppError> {
    let _commit = session_commit().await;
    require_epoch(epoch)?;
    let expires_at = now_ms() + resp.expires_in as i64 * 1_000;
    token::store_token("access_token", &resp.access_token)?;
    if let Some(ref rt) = resp.refresh_token {
        token::store_token("refresh_token", rt)?;
    }
    upsert_setting(pool, "spotify_token_expires_at", &expires_at.to_string()).await?;

    {
        let mut g = auth.write().await;
        g.access_token = Some(resp.access_token.clone());
        g.expires_at   = Some(expires_at);
        if let Some(rt) = resp.refresh_token {
            g.refresh_token = Some(rt);
        }
    }
    Ok(resp.access_token)
}

// background refresh loop, keeps the token alive while ur using the app

pub async fn refresh_loop(app: AppHandle) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(240));
    loop {
        interval.tick().await;
        let (db, auth) = {
            let s = app.state::<crate::state::AppState>();
            (s.db.clone(), s.auth.clone())
        };
        if auth.read().await.access_token.is_some() {
            if let Err(e) = get_valid_token(&db, &auth).await {
                eprintln!("[auth] background refresh failed: {e}");
            }
        }
    }
}

// http helper stuff

pub(crate) async fn call_token_endpoint(
    params: &[(&str, &str)],
) -> Result<token::TokenResponse, AppError> {
    let resp = crate::http::client()
        .post("https://accounts.spotify.com/api/token")
        .form(params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body   = resp.text().await.unwrap_or_default();
        return Err(AppError::Auth(format!("Token endpoint {status}: {body}")));
    }
    resp.json::<token::TokenResponse>()
        .await
        .map_err(|e| AppError::Network(e.to_string()))
}

// full login, swap the code for tokens then grab the profile then save it all

pub async fn complete_login(
    client_id: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    pool: &SqlitePool,
    auth: &RwLock<AuthState>,
    epoch: u64,
) -> Result<AuthStatus, AppError> {
    require_epoch(epoch)?;
    let resp = call_token_endpoint(&[
        ("grant_type", "authorization_code"), ("code", code),
        ("redirect_uri", redirect_uri), ("client_id", client_id),
        ("code_verifier", verifier),
    ]).await?;
    let refresh_token = resp.refresh_token
        .ok_or_else(|| AppError::Auth("Spotify did not return a refresh token".into()))?;
    let profile = fetch_profile_retrying(&resp.access_token).await;

    // Network work runs outside the commit lock. Logout can invalidate it at
    // any point, and its response may only be stored for the original session.
    let _commit = session_commit().await;
    require_epoch(epoch)?;
    token::store_token("access_token", &resp.access_token)?;
    token::store_token("refresh_token", &refresh_token)?;
    let expires_at = now_ms() + resp.expires_in as i64 * 1_000;
    upsert_setting(pool, "spotify_token_expires_at", &expires_at.to_string()).await?;
    upsert_setting(pool, "spotify_auth_client_id", client_id).await?;
    match profile {
        Ok(p) => cache_profile(pool, &p).await?,
        Err(e) => {
            eprintln!("[auth] signed in but could not read profile: {e}");
            for key in profile::PROFILE_KEYS {
                sqlx::query("DELETE FROM settings WHERE key = ?").bind(key).execute(pool).await?;
            }
        }
    }
    upsert_setting(pool, "spotify_signed_out", "0").await?;
    bump_auth_epoch();
    *auth.write().await = AuthState {
        access_token: Some(resp.access_token), refresh_token: Some(refresh_token),
        expires_at: Some(expires_at),
    };
    build_auth_status(pool, true).await
}

// auth status, the safe stuff we hand to the frontend (no tokens lol)

pub async fn build_auth_status(
    pool:      &SqlitePool,
    logged_in: bool,
) -> Result<AuthStatus, AppError> {
    if !logged_in {
        return Ok(AuthStatus {
            logged_in: false,
            user_id:      None,
            display_name: None,
            email:        None,
            product:      None,
            image_url:    None,
        });
    }

    let user_id      = get_setting_value(pool, "spotify_user_id").await?;
    let display_name = get_setting_value(pool, "spotify_display_name").await?;
    let email        = get_setting_value(pool, "spotify_email").await?;
    let product      = get_setting_value(pool, "spotify_product").await?;
    let image_url    = get_setting_value(pool, "spotify_image_url").await?;

    Ok(AuthStatus {
        logged_in: true,
        user_id:      user_id     .filter(|s| !s.is_empty()),
        display_name: display_name.filter(|s| !s.is_empty()),
        email:        email       .filter(|s| !s.is_empty()),
        product:      product     .filter(|s| !s.is_empty()),
        image_url:    image_url   .filter(|s| !s.is_empty()),
    })
}

#[cfg(test)]
mod session_tests {
    use super::*;

    #[tokio::test]
    async fn late_refresh_and_login_cannot_restore_a_logged_out_session() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL)")
            .execute(&pool).await.unwrap();
        let auth = RwLock::new(AuthState::default());
        let epoch = auth_epoch();
        bump_auth_epoch();
        let response: token::TokenResponse = serde_json::from_str(
            r#"{"access_token":"late-token","refresh_token":"late-refresh","expires_in":3600}"#
        ).unwrap();
        assert!(commit_refresh(&pool, &auth, epoch, response).await.is_err());
        assert!(auth.read().await.access_token.is_none());
        assert!(get_setting_value(&pool, "spotify_token_expires_at").await.unwrap().is_none());
        assert!(complete_login("client", "code", "verifier", "redirect", &pool, &auth, epoch).await.is_err());
        assert!(auth.read().await.access_token.is_none());
    }
}
