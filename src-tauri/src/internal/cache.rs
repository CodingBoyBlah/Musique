//! tiny (entity, kind) -> bytes cache over the `extension_cache` table.

use sqlx::SqlitePool;

pub const HOUR: i64 = 3_600_000;
pub const DAY: i64 = 24 * HOUR;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// cached payload if it's younger than `max_age_ms`. `None` for a miss or a
/// stale row - use `get_stale` for the offline fallback.
pub async fn get(pool: &SqlitePool, uri: &str, kind: &str, max_age_ms: i64) -> Option<Vec<u8>> {
    sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT payload FROM extension_cache WHERE entity_uri = ? AND kind = ? AND fetched_at >= ?",
    )
    .bind(uri)
    .bind(kind)
    .bind(now_ms() - max_age_ms)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|r| r.0)
}

/// whatever we have, however old. for when the live fetch just failed
pub async fn get_stale(pool: &SqlitePool, uri: &str, kind: &str) -> Option<Vec<u8>> {
    sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT payload FROM extension_cache WHERE entity_uri = ? AND kind = ?",
    )
    .bind(uri)
    .bind(kind)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|r| r.0)
}

pub async fn put(pool: &SqlitePool, uri: &str, kind: &str, payload: &[u8]) {
    let _ = sqlx::query(
        "INSERT INTO extension_cache (entity_uri, kind, payload, fetched_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(entity_uri, kind) DO UPDATE SET
             payload    = excluded.payload,
             fetched_at = excluded.fetched_at",
    )
    .bind(uri)
    .bind(kind)
    .bind(payload)
    .bind(now_ms())
    .execute(pool)
    .await;
}

/// json convenience over get/put, most internal endpoints speak json
pub async fn get_json<T: serde::de::DeserializeOwned>(
    pool: &SqlitePool,
    uri: &str,
    kind: &str,
    max_age_ms: i64,
) -> Option<T> {
    let bytes = get(pool, uri, kind, max_age_ms).await?;
    serde_json::from_slice(&bytes).ok()
}

pub async fn get_json_stale<T: serde::de::DeserializeOwned>(
    pool: &SqlitePool,
    uri: &str,
    kind: &str,
) -> Option<T> {
    let bytes = get_stale(pool, uri, kind).await?;
    serde_json::from_slice(&bytes).ok()
}

pub async fn put_json<T: serde::Serialize>(pool: &SqlitePool, uri: &str, kind: &str, value: &T) {
    if let Ok(bytes) = serde_json::to_vec(value) {
        put(pool, uri, kind, &bytes).await;
    }
}

/// the usual shape of an internal fetch: fresh cache -> live -> stale cache.
/// `fetch` only runs on a miss; its result is cached on success.
pub async fn cached_json<T, F, Fut>(
    pool: &SqlitePool,
    uri: &str,
    kind: &str,
    max_age_ms: i64,
    fetch: F,
) -> Result<T, crate::errors::AppError>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, crate::errors::AppError>>,
{
    if let Some(hit) = get_json::<T>(pool, uri, kind, max_age_ms).await {
        return Ok(hit);
    }
    match fetch().await {
        Ok(v) => {
            put_json(pool, uri, kind, &v).await;
            Ok(v)
        }
        Err(e) => get_json_stale::<T>(pool, uri, kind).await.ok_or(e),
    }
}
