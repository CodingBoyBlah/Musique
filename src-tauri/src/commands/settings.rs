use crate::{errors::AppError, state::AppState};
use sqlx::Row;
use std::collections::HashMap;
use tauri::State;

// grab a single settings value by key. None when the key isnt there
#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(&key)
        .fetch_optional(&state.db)
        .await?;

    Ok(row.map(|r| r.get::<String, _>("value")))
}

// upsert a settings key/value pair and bump updated_at to now (epoch ms)
#[tauri::command]
pub async fn set_setting(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(&key)
    .bind(&value)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(())
}

// dump every setting as a flat key -> value map. frontend uses this to hydrate
// its prefs store without doing one call per key
#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, AppError> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(&state.db)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| (r.get::<String, _>("key"), r.get::<String, _>("value")))
        .collect())
}
