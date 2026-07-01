use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SavedTrack {
    pub track_id: String,
    pub added_at: i64,
}

pub async fn save(pool: &SqlitePool, track_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)",
    )
    .bind(track_id)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unsave(pool: &SqlitePool, track_id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM saved_tracks WHERE track_id = ?")
        .bind(track_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn is_saved(pool: &SqlitePool, track_id: &str) -> Result<bool, AppError> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM saved_tracks WHERE track_id = ?")
            .bind(track_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

pub async fn list(pool: &SqlitePool, limit: i64, offset: i64) -> Result<Vec<SavedTrack>, AppError> {
    Ok(
        sqlx::query_as::<_, SavedTrack>(
            "SELECT * FROM saved_tracks ORDER BY added_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_track(pool: &SqlitePool, id: &str) {
        sqlx::query(
            "INSERT INTO tracks (id, name, duration_ms, track_number, disc_number, explicit, is_local, updated_at)
             VALUES (?, 'Test Track', 0, 1, 1, 0, 0, 0)",
        )
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn save_and_check(pool: SqlitePool) {
        insert_track(&pool, "t1").await;
        save(&pool, "t1").await.unwrap();
        assert!(is_saved(&pool, "t1").await.unwrap());
        assert!(!is_saved(&pool, "t999").await.unwrap());
    }

    #[sqlx::test]
    async fn unsave_removes(pool: SqlitePool) {
        insert_track(&pool, "t1").await;
        save(&pool, "t1").await.unwrap();
        assert!(unsave(&pool, "t1").await.unwrap());
        assert!(!is_saved(&pool, "t1").await.unwrap());
        assert!(!unsave(&pool, "t1").await.unwrap()); // already gone lol
    }

    #[sqlx::test]
    async fn list_paginates(pool: SqlitePool) {
        for id in ["t1", "t2", "t3"] {
            insert_track(&pool, id).await;
            save(&pool, id).await.unwrap();
        }
        let page1 = list(&pool, 2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);
        let page2 = list(&pool, 2, 2).await.unwrap();
        assert_eq!(page2.len(), 1);
    }
}
