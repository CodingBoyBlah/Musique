use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id:           String,
    pub display_name: Option<String>,
    pub email:        Option<String>,
    pub product:      Option<String>,
    pub image_url:    Option<String>,
    pub country:      Option<String>,
    pub updated_at:   i64,
}

pub async fn upsert(pool: &SqlitePool, u: &User) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO users (id, display_name, email, product, image_url, country, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             display_name = excluded.display_name,
             email        = excluded.email,
             product      = excluded.product,
             image_url    = excluded.image_url,
             country      = excluded.country,
             updated_at   = excluded.updated_at",
    )
    .bind(&u.id)
    .bind(&u.display_name)
    .bind(&u.email)
    .bind(&u.product)
    .bind(&u.image_url)
    .bind(&u.country)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<User>, AppError> {
    Ok(
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> User {
        User {
            id:           "u1".to_string(),
            display_name: Some("Alice".to_string()),
            email:        Some("alice@example.com".to_string()),
            product:      Some("premium".to_string()),
            image_url:    None,
            country:      Some("US".to_string()),
            updated_at:   0,
        }
    }

    #[sqlx::test]
    async fn upsert_and_find(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let u = find(&pool, "u1").await.unwrap().unwrap();
        assert_eq!(u.display_name.as_deref(), Some("Alice"));
        assert_eq!(u.product.as_deref(), Some("premium"));
    }

    #[sqlx::test]
    async fn upsert_updates_on_conflict(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let updated = User { product: Some("free".to_string()), ..sample() };
        upsert(&pool, &updated).await.unwrap();
        let u = find(&pool, "u1").await.unwrap().unwrap();
        assert_eq!(u.product.as_deref(), Some("free"));
    }

    #[sqlx::test]
    async fn delete_removes_row(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        assert!(delete(&pool, "u1").await.unwrap());
        assert!(find(&pool, "u1").await.unwrap().is_none());
        assert!(!delete(&pool, "u1").await.unwrap()); // already gone lol
    }
}
