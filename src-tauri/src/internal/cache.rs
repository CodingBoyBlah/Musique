//! tiny (entity, kind) -> bytes cache over the `extension_cache` table.

use std::collections::HashMap;
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

/// Fresh payloads in batches of 200, keeping SQLite's timestamps authoritative.
pub async fn get_batch(
    pool: &SqlitePool,
    uris: &[String],
    kind: &str,
    max_age_ms: i64,
) -> HashMap<String, Vec<u8>> {
    let mut out = HashMap::new();
    let min_ts = now_ms() - max_age_ms;
    for chunk in uris.chunks(200) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT entity_uri, payload FROM extension_cache WHERE kind = ? AND entity_uri IN ({placeholders}) AND fetched_at >= ?"
        );
        let mut q = sqlx::query_as::<_, (String, Vec<u8>)>(&sql).bind(kind);
        for u in chunk {
            q = q.bind(u);
        }
        q = q.bind(min_ts);

        if let Ok(rows) = q.fetch_all(pool).await {
            out.extend(rows);
        }
    }

    out
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

/// batched fallback for stale entries
pub async fn get_stale_batch(
    pool: &SqlitePool,
    uris: &[String],
    kind: &str,
) -> HashMap<String, Vec<u8>> {
    let mut out: HashMap<String, Vec<u8>> = HashMap::new();
    if uris.is_empty() {
        return out;
    }
    for chunk in uris.chunks(200) {
        let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT entity_uri, payload FROM extension_cache WHERE kind = ? AND entity_uri IN ({placeholders})"
        );
        let mut q = sqlx::query_as::<_, (String, Vec<u8>)>(&sql).bind(kind);
        for u in chunk {
            q = q.bind(u);
        }
        if let Ok(rows) = q.fetch_all(pool).await {
            for (uri, payload) in rows {
                out.insert(uri, payload);
            }
        }
    }
    out
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

/// batched put in a single transaction
pub async fn put_batch(
    pool: &SqlitePool,
    kind: &str,
    entries: &[(String, Vec<u8>)],
) {
    if entries.is_empty() {
        return;
    }
    let now = now_ms();
    let Ok(mut tx) = pool.begin().await else { return; };
    for (uri, payload) in entries {
        let _ = sqlx::query(
            "INSERT INTO extension_cache (entity_uri, kind, payload, fetched_at) VALUES (?, ?, ?, ?)
             ON CONFLICT(entity_uri, kind) DO UPDATE SET
                 payload    = excluded.payload,
                 fetched_at = excluded.fetched_at",
        )
        .bind(uri)
        .bind(kind)
        .bind(payload)
        .bind(now)
        .execute(&mut *tx)
        .await;
    }
    let _ = tx.commit().await;
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

/// whatever is stored for (uri, kind), with its age in ms
async fn get_with_age(pool: &SqlitePool, uri: &str, kind: &str) -> Option<(Vec<u8>, i64)> {
    sqlx::query_as::<_, (Vec<u8>, i64)>(
        "SELECT payload, fetched_at FROM extension_cache WHERE entity_uri = ? AND kind = ?",
    )
    .bind(uri)
    .bind(kind)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|(payload, at)| (payload, now_ms() - at))
}

/// (kind, uri) pairs being refetched behind a stored copy right now, so a page
/// asked for twice (an album header prefetching its artist page, a remount)
/// goes to the network once
static REFRESHING: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(Default::default);

/// what `swr_json` pushes to the page when a refetch comes back different
#[derive(serde::Serialize)]
struct Fresh<'a, T: serde::Serialize> {
    key:  &'a serde_json::Value,
    data: &'a T,
}

// by hand: a derive would demand `T: Clone` for what is only two references
impl<T: serde::Serialize> Clone for Fresh<'_, T> {
    fn clone(&self) -> Self {
        Self { key: self.key, data: self.data }
    }
}

/// stale-while-revalidate over the cache. a stored copy answers at once, so a
/// page opened after a cold start paints from disk instead of waiting on the
/// network; one older than `revalidate_after_ms` is refetched behind it. when
/// the refetch differs from what was answered it is stored and pushed to the
/// page as `query:fresh` under `query_key` (the page's own react-query key), so
/// the page ends up exactly where its own fetch would have left it. with
/// nothing stored this simply waits on `fetch`.
pub async fn swr_json<T, F, Fut>(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    uri: &str,
    kind: &str,
    revalidate_after_ms: i64,
    query_key: serde_json::Value,
    fetch: F,
) -> Result<T, crate::errors::AppError>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, crate::errors::AppError>> + Send + 'static,
{
    if let Some((stored, age)) = get_with_age(pool, uri, kind).await {
        // a copy from before a shape change doesn't parse; that is a miss
        if let Ok(hit) = serde_json::from_slice::<T>(&stored) {
            if age >= revalidate_after_ms {
                refresh_behind(app, pool, uri, kind, stored, query_key, fetch);
            }
            return Ok(hit);
        }
    }
    let v = fetch().await?;
    put_json(pool, uri, kind, &v).await;
    Ok(v)
}

fn refresh_behind<T, F, Fut>(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    uri: &str,
    kind: &str,
    stored: Vec<u8>,
    query_key: serde_json::Value,
    fetch: F,
) where
    T: serde::Serialize + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, crate::errors::AppError>> + Send + 'static,
{
    use tauri::Emitter;
    let slot = format!("{kind} {uri}");
    if !REFRESHING.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(slot.clone()) {
        return;
    }
    let (app, pool, uri, kind) = (app.clone(), pool.clone(), uri.to_string(), kind.to_string());
    // a sign-out while this is in flight purges the cache; the old account's
    // page must not be written back into it
    let epoch = crate::auth::auth_epoch();
    tauri::async_runtime::spawn(async move {
        if let Ok(v) = fetch().await {
            if let Ok(bytes) = serde_json::to_vec(&v) {
                if crate::auth::epoch_is_current(epoch) {
                    put(&pool, &uri, &kind, &bytes).await;
                    if bytes != stored {
                        let _ = app.emit("query:fresh", Fresh { key: &query_key, data: &v });
                    }
                }
            }
        }
        REFRESHING.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&slot);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE extension_cache (entity_uri TEXT, kind TEXT, payload BLOB, fetched_at INTEGER, PRIMARY KEY (entity_uri, kind))")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn batch_preserves_freshness_and_stale_fallback() {
        let pool = test_pool().await;
        put(&pool, "fresh", "track", b"new").await;
        sqlx::query("INSERT INTO extension_cache VALUES ('stale', 'track', ?, ?)")
            .bind(b"old".as_slice()).bind(now_ms() - DAY)
            .execute(&pool).await.unwrap();
        let uris = vec!["fresh".into(), "stale".into(), "missing".into(), "fresh".into()];
        let hits = get_batch(&pool, &uris, "track", HOUR).await;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits["fresh"], b"new");
        let stale = get_stale_batch(&pool, &uris, "track").await;
        assert_eq!(stale.len(), 2);
        assert_eq!(stale["stale"], b"old");
        assert!(get_batch(&pool, &uris, "track", HOUR).await.get("stale").is_none());
    }

    #[tokio::test]
    async fn batches_chunk_and_overwrite_without_crossing_kinds_or_pools() {
        let pool = test_pool().await;
        let entries: Vec<_> = (0..450).map(|i| (format!("track:{i}"), vec![i as u8])).collect();
        put_batch(&pool, "track", &entries).await;
        put_batch(&pool, "track", &[("track:0".into(), b"replacement".to_vec())]).await;
        put(&pool, "track:0", "album", b"album").await;
        let uris: Vec<_> = entries.iter().map(|(uri, _)| uri.clone()).collect();
        let hits = get_batch(&pool, &uris, "track", HOUR).await;
        assert_eq!(hits.len(), 450);
        assert_eq!(hits["track:0"], b"replacement");
        assert_eq!(hits["track:449"], vec![449u16 as u8]);
        assert_eq!(get_batch(&pool, &uris, "album", HOUR).await.len(), 1);
        let other = test_pool().await;
        assert!(get_batch(&other, &uris, "track", HOUR).await.is_empty());
        sqlx::query("DELETE FROM extension_cache").execute(&pool).await.unwrap();
        assert!(get_stale_batch(&pool, &uris, "track").await.is_empty());
    }
}
