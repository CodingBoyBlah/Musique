use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Track {
    pub id:           String,
    pub name:         String,
    pub album_id:     Option<String>,
    pub duration_ms:  i64,
    pub track_number: i64,
    pub disc_number:  i64,
    pub explicit:     bool,
    pub popularity:   Option<i64>,
    pub preview_url:  Option<String>,
    pub is_local:     bool,
    pub updated_at:   i64,
}

pub async fn upsert(pool: &SqlitePool, t: &Track) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO tracks
             (id, name, album_id, duration_ms, track_number, disc_number,
              explicit, popularity, preview_url, is_local, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name         = excluded.name,
             album_id     = excluded.album_id,
             duration_ms  = excluded.duration_ms,
             track_number = excluded.track_number,
             disc_number  = excluded.disc_number,
             explicit     = excluded.explicit,
             popularity   = excluded.popularity,
             preview_url  = excluded.preview_url,
             is_local     = excluded.is_local,
             updated_at   = excluded.updated_at",
    )
    .bind(&t.id)
    .bind(&t.name)
    .bind(&t.album_id)
    .bind(t.duration_ms)
    .bind(t.track_number)
    .bind(t.disc_number)
    .bind(t.explicit)
    .bind(t.popularity)
    .bind(&t.preview_url)
    .bind(t.is_local)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<Track>, AppError> {
    Ok(
        sqlx::query_as::<_, Track>("SELECT * FROM tracks WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM tracks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn list_by_album(pool: &SqlitePool, album_id: &str) -> Result<Vec<Track>, AppError> {
    Ok(
        sqlx::query_as::<_, Track>(
            "SELECT * FROM tracks WHERE album_id = ? ORDER BY disc_number, track_number",
        )
        .bind(album_id)
        .fetch_all(pool)
        .await?,
    )
}

/// hook an artist up to a track. just ignores dupes quietly
pub async fn add_artist(
    pool:      &SqlitePool,
    track_id:  &str,
    artist_id: &str,
    position:  i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO track_artists (track_id, artist_id, position) VALUES (?, ?, ?)",
    )
    .bind(track_id)
    .bind(artist_id)
    .bind(position)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Track {
        Track {
            id:           "t1".to_string(),
            name:         "Come Together".to_string(),
            album_id:     None,
            duration_ms:  259000,
            track_number: 1,
            disc_number:  1,
            explicit:     false,
            popularity:   Some(80),
            preview_url:  None,
            is_local:     false,
            updated_at:   0,
        }
    }

    #[sqlx::test]
    async fn upsert_and_find(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let t = find(&pool, "t1").await.unwrap().unwrap();
        assert_eq!(t.name, "Come Together");
        assert_eq!(t.duration_ms, 259000);
        assert!(!t.explicit);
    }

    #[sqlx::test]
    async fn explicit_flag_roundtrips(pool: SqlitePool) {
        let mut t = sample();
        t.explicit = true;
        upsert(&pool, &t).await.unwrap();
        let found = find(&pool, "t1").await.unwrap().unwrap();
        assert!(found.explicit);
    }

    #[sqlx::test]
    async fn delete_removes_row(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        assert!(delete(&pool, "t1").await.unwrap());
        assert!(find(&pool, "t1").await.unwrap().is_none());
    }
}
