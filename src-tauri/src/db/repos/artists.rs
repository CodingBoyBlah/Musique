use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Artist {
    pub id:         String,
    pub name:       String,
    pub image_url:  Option<String>,
    pub genres:     Option<String>, // json array like '["pop","rock"]'
    pub popularity: Option<i64>,
    pub updated_at: i64,
}

pub async fn upsert(pool: &SqlitePool, a: &Artist) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO artists (id, name, image_url, genres, popularity, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name       = excluded.name,
             -- track/album syncs shove artists in with NULL image/genres/popularity.
             -- COALESCE keeps the better data a previous followed-artist sync
             -- saved so the artist avatars dont get wiped out
             image_url  = COALESCE(excluded.image_url, artists.image_url),
             genres     = COALESCE(excluded.genres, artists.genres),
             popularity = COALESCE(excluded.popularity, artists.popularity),
             updated_at = excluded.updated_at",
    )
    .bind(&a.id)
    .bind(&a.name)
    .bind(&a.image_url)
    .bind(&a.genres)
    .bind(a.popularity)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<Artist>, AppError> {
    Ok(
        sqlx::query_as::<_, Artist>("SELECT * FROM artists WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Artist>, AppError> {
    Ok(
        sqlx::query_as::<_, Artist>("SELECT * FROM artists ORDER BY name")
            .fetch_all(pool)
            .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Artist {
        Artist {
            id:         "a1".to_string(),
            name:       "The Beatles".to_string(),
            image_url:  None,
            genres:     Some(r#"["rock","pop"]"#.to_string()),
            popularity: Some(90),
            updated_at: 0,
        }
    }

    #[sqlx::test]
    async fn upsert_and_find(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let a = find(&pool, "a1").await.unwrap().unwrap();
        assert_eq!(a.name, "The Beatles");
        assert_eq!(a.popularity, Some(90));
    }

    #[sqlx::test]
    async fn list_returns_all(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        upsert(&pool, &Artist { id: "a2".to_string(), name: "ABBA".to_string(), ..sample() })
            .await
            .unwrap();
        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "ABBA"); // order by name
    }

    #[sqlx::test]
    async fn delete_removes_row(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        assert!(delete(&pool, "a1").await.unwrap());
        assert!(find(&pool, "a1").await.unwrap().is_none());
    }
}
