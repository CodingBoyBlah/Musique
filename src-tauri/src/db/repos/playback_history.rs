use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PlaybackEntry {
    pub id:           i64,
    pub track_id:     String,
    pub context_type: Option<String>,
    pub context_id:   Option<String>,
    pub played_at:    i64,
    pub duration_ms:  Option<i64>,
}

pub async fn push(
    pool:         &SqlitePool,
    track_id:     &str,
    context_type: Option<&str>,
    context_id:   Option<&str>,
    duration_ms:  Option<i64>,
) -> Result<i64, AppError> {
    let id = sqlx::query(
        "INSERT INTO playback_history
             (track_id, context_type, context_id, played_at, duration_ms)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(track_id)
    .bind(context_type)
    .bind(context_id)
    .bind(super::now_ms())
    .bind(duration_ms)
    .execute(pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

pub async fn recent(pool: &SqlitePool, limit: i64) -> Result<Vec<PlaybackEntry>, AppError> {
    Ok(
        sqlx::query_as::<_, PlaybackEntry>(
            "SELECT * FROM playback_history ORDER BY played_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?,
    )
}

pub async fn for_track(
    pool:     &SqlitePool,
    track_id: &str,
    limit:    i64,
) -> Result<Vec<PlaybackEntry>, AppError> {
    Ok(
        sqlx::query_as::<_, PlaybackEntry>(
            "SELECT * FROM playback_history WHERE track_id = ? ORDER BY played_at DESC LIMIT ?",
        )
        .bind(track_id)
        .bind(limit)
        .fetch_all(pool)
        .await?,
    )
}

pub async fn clear_before(pool: &SqlitePool, before_ms: i64) -> Result<u64, AppError> {
    Ok(
        sqlx::query("DELETE FROM playback_history WHERE played_at < ?")
            .bind(before_ms)
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
        push(&pool, "t1", Some("album"), Some("al1"), Some(240000)).await.unwrap();
        push(&pool, "t2", None, None, None).await.unwrap();
        let entries = recent(&pool, 10).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].track_id, "t2"); // newest one should be on top
    }

    #[sqlx::test]
    async fn for_track_filters(pool: SqlitePool) {
        push(&pool, "t1", None, None, None).await.unwrap();
        push(&pool, "t2", None, None, None).await.unwrap();
        push(&pool, "t1", None, None, None).await.unwrap();
        let t1_plays = for_track(&pool, "t1", 10).await.unwrap();
        assert_eq!(t1_plays.len(), 2);
    }

    #[sqlx::test]
    async fn clear_before_prunes(pool: SqlitePool) {
        // shove one entry in then nuke everything before some far future time
        push(&pool, "t1", None, None, None).await.unwrap();
        let removed = clear_before(&pool, i64::MAX).await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(recent(&pool, 100).await.unwrap().len(), 0);
    }
}
