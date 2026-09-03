use crate::{auth, errors::AppError, state::AppState};
use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;

const SERVICE: &str = "spotify-client";
const ACCOUNT_SECRET: &str = "client_secret";

const ACCOUNT_ID: &str = "client_id";

fn entry(account: &str) -> Result<Entry, AppError> {
    Entry::new(SERVICE, account).map_err(AppError::from)
}

// read the client_id keyring mirror, if any (None on NoEntry or any error)
fn keyring_client_id() -> Option<String> {
    entry(ACCOUNT_ID).ok().and_then(|e| e.get_password().ok())
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
    pub is_custom: bool,
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
            if let Ok(e) = entry(ACCOUNT_SECRET) {
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
    client_secret: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let trimmed_id = client_id.trim();

    // If empty or explicitly reset to the default shared ID, clear custom settings
    if trimmed_id.is_empty() || trimmed_id == auth::SHARED_CLIENT_ID {
        return clear_credentials(state).await;
    }

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind("spotify_client_id")
    .bind(trimmed_id)
    .bind(now_ms())
    .execute(&state.db)
    .await?;

    let _ = entry(ACCOUNT_ID).and_then(|e| e.set_password(trimmed_id).map_err(AppError::from));

    if let Some(secret) = client_secret {
        let trimmed_secret = secret.trim();
        if trimmed_secret.is_empty() {
            let _ = entry(ACCOUNT_SECRET).and_then(|e| e.delete_password().map_err(AppError::from));
        } else {
            entry(ACCOUNT_SECRET)?.set_password(trimmed_secret)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_credentials(
    state: State<'_, AppState>,
) -> Result<Option<Credentials>, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'spotify_client_id'")
        .fetch_optional(&state.db)
        .await?;

    let custom_id = match row {
        Some(r) => {
            let v = r.get::<String, _>("value");
            if v.trim().is_empty() {
                None
            } else {
                Some(v)
            }
        }
        None => keyring_client_id(),
    };

    let has_secret = match entry(ACCOUNT_SECRET)?.get_password() {
        Ok(_) => true,
        Err(keyring::Error::NoEntry) => false,
        Err(e) => return Err(AppError::Keyring(e.to_string())),
    };

    match custom_id {
        Some(id) => Ok(Some(Credentials {
            client_id: id,
            has_secret,
            is_custom: true,
        })),
        None => Ok(Some(Credentials {
            client_id: auth::SHARED_CLIENT_ID.to_string(),
            has_secret: false,
            is_custom: false,
        })),
    }
}

#[tauri::command]
pub async fn validate_credentials(
    state: State<'_, AppState>,
) -> Result<ValidationResult, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'spotify_client_id'")
        .fetch_optional(&state.db)
        .await?;

    let client_id = match row.map(|r| r.get::<String, _>("value")).or_else(keyring_client_id) {
        Some(id) if !id.trim().is_empty() => id,
        _ => return Ok(ValidationResult { valid: true, error: None }),
    };

    // If using the shared client ID, it is valid by default
    if client_id == auth::SHARED_CLIENT_ID {
        return Ok(ValidationResult { valid: true, error: None });
    }

    let client_secret = match entry(ACCOUNT_SECRET)?.get_password() {
        Ok(s) if !s.trim().is_empty() => s,
        _ => {
            // PKCE does not require a secret; check if client ID is a 32-character hex ID
            if client_id.len() == 32 && client_id.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(ValidationResult { valid: true, error: None });
            }
            return Ok(ValidationResult {
                valid: false,
                error: Some("Client ID must be a 32-character hex Spotify Application ID".into()),
            });
        }
    };

    let auth_header = STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let res = reqwest::Client::new()
        .post("https://accounts.spotify.com/api/token")
        .header("Authorization", format!("Basic {}", auth_header))
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
            400 => "Invalid request, check your client ID".into(),
            401 => "Unauthorized, invalid client ID or secret".into(),
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

    // clear both keyring mirrors (secret + the id we now stash there)
    for account in [ACCOUNT_SECRET, ACCOUNT_ID] {
        match entry(account)?.delete_password() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(AppError::Keyring(e.to_string())),
        }
    }

    Ok(())
}
