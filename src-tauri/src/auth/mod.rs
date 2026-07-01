pub mod http;
pub mod pkce;
pub mod token;

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

// private spotify api types dont touch

#[derive(Deserialize)]
struct SpotifyImage {
    url: String,
}

#[derive(Deserialize)]
struct SpotifyProfile {
    id:           String,
    display_name: Option<String>,
    email:        Option<String>,
    product:      Option<String>,
    images:       Option<Vec<SpotifyImage>>,
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

pub async fn init_auth_state(pool: &SqlitePool) -> AuthState {
    let (access_token, refresh_token) = match token::load_tokens() {
        Ok(Some(pair)) => (Some(pair.0), Some(pair.1)),
        _ => return AuthState::default(),
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
    {
        let g = auth.read().await;
        if let (Some(tok), Some(exp)) = (&g.access_token, g.expires_at) {
            if exp - now_ms() > 60_000 {
                return Ok(tok.clone());
            }
        }
    }
    do_refresh(pool, auth).await
}

async fn do_refresh(pool: &SqlitePool, auth: &RwLock<AuthState>) -> Result<String, AppError> {
    let client_id = get_setting_value(pool, "spotify_client_id")
        .await?
        .ok_or_else(|| AppError::Auth("No client_id configured".into()))?;

    let refresh_token = auth
        .read()
        .await
        .refresh_token
        .clone()
        .ok_or_else(|| AppError::Auth("Not logged in".into()))?;

    let resp = match call_token_endpoint(&[
        ("grant_type",    "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id",     &client_id),
    ])
    .await
    {
        Ok(r) => r,
        Err(e) => {
            
            if e.to_string().contains("invalid_grant") {
                let _ = token::clear_tokens();
                *auth.write().await = AuthState::default();
                return Err(AppError::Auth(
                    "Your Spotify session expired. Please sign in again.".into(),
                ));
            }
            return Err(e);
        }
    };

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
    let resp = reqwest::Client::new()
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

async fn fetch_profile(access_token: &str) -> Result<SpotifyProfile, AppError> {
    let resp = reqwest::Client::new()
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(AppError::Auth(format!(
            "Profile fetch failed: {}",
            resp.status()
        )));
    }
    resp.json::<SpotifyProfile>()
        .await
        .map_err(|e| AppError::Network(e.to_string()))
}

// full login, swap the code for tokens then grab the profile then save it all

pub async fn complete_login(
    client_id:    &str,
    code:         &str,
    verifier:     &str,
    redirect_uri: &str,
    pool:         &SqlitePool,
    auth:         &RwLock<AuthState>,
) -> Result<AuthStatus, AppError> {
    let resp = call_token_endpoint(&[
        ("grant_type",    "authorization_code"),
        ("code",          code),
        ("redirect_uri",  redirect_uri),
        ("client_id",     client_id),
        ("code_verifier", verifier),
    ])
    .await?;

    let refresh_token = resp
        .refresh_token
        .ok_or_else(|| AppError::Auth("Spotify did not return a refresh token".into()))?;

    token::store_token("access_token",  &resp.access_token)?;
    token::store_token("refresh_token", &refresh_token)?;

    let expires_at = now_ms() + resp.expires_in as i64 * 1_000;
    upsert_setting(pool, "spotify_token_expires_at", &expires_at.to_string()).await?;

    {
        let mut g    = auth.write().await;
        g.access_token  = Some(resp.access_token.clone());
        g.refresh_token = Some(refresh_token);
        g.expires_at    = Some(expires_at);
    }

    let profile   = fetch_profile(&resp.access_token).await?;
    let image_url = profile
        .images
        .as_ref()
        .and_then(|v| v.first())
        .map(|i| i.url.clone());

    for (key, val) in [
        ("spotify_user_id",      profile.id.as_str()),
        ("spotify_display_name", profile.display_name.as_deref().unwrap_or("")),
        ("spotify_email",        profile.email.as_deref().unwrap_or("")),
        ("spotify_product",      profile.product.as_deref().unwrap_or("")),
        ("spotify_image_url",    image_url.as_deref().unwrap_or("")),
    ] {
        upsert_setting(pool, key, val).await?;
    }

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
