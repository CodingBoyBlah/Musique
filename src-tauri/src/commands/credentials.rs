use crate::{errors::AppError, state::AppState};
use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;

const SERVICE: &str = "spotify-client";
const ACCOUNT: &str = "client_secret";

fn entry() -> Result<Entry, AppError> {
    Entry::new(SERVICE, ACCOUNT).map_err(AppError::from)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub has_secret: bool,
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub error: Option<String>,
}


pub async fn seed_credentials_from_env(pool: &SqlitePool) {
    if let Ok(id) = std::env::var("SPOTIFY_CLIENT_ID") {
        let id = id.trim();
        if !id.is_empty() {
            // ON CONFLICT DO NOTHING so it only seeds when theres no client_id yet
            let _ = sqlx::query(
                "INSERT INTO settings (key, value, updated_at)
                 VALUES ('spotify_client_id', ?, ?)
                 ON CONFLICT(key) DO NOTHING",
            )
            .bind(id)
            .bind(now_ms())
            .execute(pool)
            .await;
        }
    }

    if let Ok(secret) = std::env::var("SPOTIFY_CLIENT_SECRET") {
        let secret = secret.trim();
        if !secret.is_empty() {
            if let Ok(e) = entry() {
                // only write it if the keyring doesnt already have a secret
                if matches!(e.get_password(), Err(keyring::Error::NoEntry)) {
                    let _ = e.set_password(secret);
                }
            }
        }
    }
}

#[tauri::command]
pub async fn save_credentials(
    client_id: String,
    client_secret: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if client_id.trim().is_empty() {
        return Err(AppError::InvalidInput("Client ID cannot be empty".into()));
    }
    if client_secret.trim().is_empty() {
        return Err(AppError::InvalidInput("Client secret cannot be empty".into()));
    }

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind("spotify_client_id")
    .bind(&client_id)
    .bind(now_ms())
    .execute(&state.db)
    .await?;

    entry()?.set_password(&client_secret)?;

    Ok(())
}

#[tauri::command]
pub async fn get_credentials(
    state: State<'_, AppState>,
) -> Result<Option<Credentials>, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'spotify_client_id'")
        .fetch_optional(&state.db)
        .await?;

    let client_id = match row {
        Some(r) => r.get::<String, _>("value"),
        None => return Ok(None),
    };

    let has_secret = match entry()?.get_password() {
        Ok(_) => true,
        Err(keyring::Error::NoEntry) => false,
        Err(e) => return Err(AppError::Keyring(e.to_string())),
    };

    Ok(Some(Credentials { client_id, has_secret }))
}

#[tauri::command]
pub async fn validate_credentials(
    state: State<'_, AppState>,
) -> Result<ValidationResult, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'spotify_client_id'")
        .fetch_optional(&state.db)
        .await?;

    let client_id = match row {
        Some(r) => r.get::<String, _>("value"),
        None => {
            return Ok(ValidationResult {
                valid: false,
                error: Some("No client ID stored".into()),
            })
        }
    };

    let client_secret = match entry()?.get_password() {
        Ok(s) => s,
        Err(_) => {
            return Ok(ValidationResult {
                valid: false,
                error: Some("No client secret stored".into()),
            })
        }
    };

    let auth = STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let res = reqwest::Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", auth))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("grant_type=client_credentials")
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if res.status().is_success() {
        Ok(ValidationResult { valid: true, error: None })
    } else {
        let status = res.status().as_u16();
        let msg = match status {
            400 => "Invalid request — check your client ID".into(),
            401 => "Unauthorized — invalid client ID or secret".into(),
            _ => format!("Spotify returned HTTP {}", status),
        };
        Ok(ValidationResult { valid: false, error: Some(msg) })
    }
}

#[tauri::command]
pub async fn clear_credentials(state: State<'_, AppState>) -> Result<(), AppError> {
    sqlx::query("DELETE FROM settings WHERE key = 'spotify_client_id'")
        .execute(&state.db)
        .await?;

    match entry()?.delete_password() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(AppError::Keyring(e.to_string())),
    }

    Ok(())
}
