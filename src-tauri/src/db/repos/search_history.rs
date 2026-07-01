use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SearchEntry {
    pub id:          i64,
    pub query:       String,
    pub result_type: Option<String>,
    pub result_id:   Option<String>,
    pub searched_at: i64,
}

pub async fn push(
    pool:        &SqlitePool,
    query:       &str,
    result_type: Option<&str>,
    result_id:   Option<&str>,
) -> Result<i64, AppError> {
    let id = sqlx::query(
        "INSERT INTO search_history (query, result_type, result_id, searched_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(query)
    .bind(result_type)
    .bind(result_id)
    .bind(super::now_ms())
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

pub async fn recent(pool: &SqlitePool, limit: i64) -> Result<Vec<SearchEntry>, AppError> {
    Ok(
        sqlx::query_as::<_, SearchEntry>(
            "SELECT * FROM search_history ORDER BY searched_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM search_history WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn clear(pool: &SqlitePool) -> Result<u64, AppError> {
    Ok(
        sqlx::query("DELETE FROM search_history")
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn push_and_recent(pool: SqlitePool) {
        push(&pool, "beatles", Some("artist"), Some("a1")).await.unwrap();
        push(&pool, "abbey road", Some("album"), None).await.unwrap();
        let entries = recent(&pool, 10).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].query, "abbey road"); // newest one should be on top
    }

    #[sqlx::test]
    async fn recent_respects_limit(pool: SqlitePool) {
        for i in 0..5u8 {
            push(&pool, &format!("query {i}"), None, None).await.unwrap();
        }
        let entries = recent(&pool, 3).await.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[sqlx::test]
    async fn clear_removes_all(pool: SqlitePool) {
        push(&pool, "q1", None, None).await.unwrap();
        push(&pool, "q2", None, None).await.unwrap();
        let removed = clear(&pool).await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(recent(&pool, 100).await.unwrap().len(), 0);
    }
}
