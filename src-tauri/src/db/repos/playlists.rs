use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Playlist {
    pub id:           String,
    pub name:         String,
    pub description:  Option<String>,
    pub owner_id:     Option<String>,
    pub image_url:    Option<String>,
    pub total_tracks: i64,
    pub is_public:    bool,
    pub is_local:     bool,
    pub snapshot_id:  Option<String>,
    pub updated_at:   i64,
}

pub async fn upsert(pool: &SqlitePool, p: &Playlist) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO playlists
             (id, name, description, owner_id, image_url, total_tracks,
              is_public, is_local, snapshot_id, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name         = excluded.name,
             description  = excluded.description,
             owner_id     = excluded.owner_id,
             image_url    = excluded.image_url,
             total_tracks = excluded.total_tracks,
             is_public    = excluded.is_public,
             is_local     = excluded.is_local,
             snapshot_id  = excluded.snapshot_id,
             updated_at   = excluded.updated_at",
    )
    .bind(&p.id)
    .bind(&p.name)
    .bind(&p.description)
    .bind(&p.owner_id)
    .bind(&p.image_url)
    .bind(p.total_tracks)
    .bind(p.is_public)
    .bind(p.is_local)
    .bind(&p.snapshot_id)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<Playlist>, AppError> {
    Ok(
        sqlx::query_as::<_, Playlist>("SELECT * FROM playlists WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_by_owner(pool: &SqlitePool, owner_id: &str) -> Result<Vec<Playlist>, AppError> {
    Ok(
        sqlx::query_as::<_, Playlist>(
            "SELECT * FROM playlists WHERE owner_id = ? ORDER BY name",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

/// tack a track onto a playlist. just ignores it if its already in there
pub async fn add_track(
    pool:        &SqlitePool,
    playlist_id: &str,
    track_id:    &str,
    position:    i64,
    added_by:    Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO playlist_tracks
             (playlist_id, track_id, position, added_at, added_by)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(playlist_id)
    .bind(track_id)
    .bind(position)
    .bind(super::now_ms())
    .bind(added_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_track(
    pool:        &SqlitePool,
    playlist_id: &str,
    track_id:    &str,
) -> Result<bool, AppError> {
    let rows =
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
            .bind(playlist_id)
            .bind(track_id)
            .execute(pool)
            .await?
            .rows_affected();
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Playlist {
        Playlist {
            id:           "pl1".to_string(),
            name:         "Favourites".to_string(),
            description:  None,
            owner_id:     Some("u1".to_string()),
            image_url:    None,
            total_tracks: 0,
            is_public:    false,
            is_local:     true,
            snapshot_id:  None,
            updated_at:   0,
        }
    }

    #[sqlx::test]
    async fn upsert_and_find(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let p = find(&pool, "pl1").await.unwrap().unwrap();
        assert_eq!(p.name, "Favourites");
        assert!(p.is_local);
        assert!(!p.is_public);
    }

    #[sqlx::test]
    async fn list_by_owner_returns_correct(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        upsert(
            &pool,
            &Playlist { id: "pl2".to_string(), owner_id: Some("u2".to_string()), ..sample() },
        )
        .await
        .unwrap();
        let owned = list_by_owner(&pool, "u1").await.unwrap();
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].id, "pl1");
    }

    #[sqlx::test]
    async fn delete_removes_row(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        assert!(delete(&pool, "pl1").await.unwrap());
        assert!(find(&pool, "pl1").await.unwrap().is_none());
    }
}
