use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Album {
    pub id:           String,
    pub name:         String,
    pub album_type:   String,
    pub image_url:    Option<String>,
    pub release_date: Option<String>,
    pub total_tracks: i64,
    pub genres:       Option<String>, // json array
    pub popularity:   Option<i64>,
    pub updated_at:   i64,
}

pub async fn upsert(pool: &SqlitePool, a: &Album) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO albums
             (id, name, album_type, image_url, release_date, total_tracks, genres, popularity, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
             name         = excluded.name,
             album_type   = excluded.album_type,
             -- track syncs shove albums in with NULL image/genres/popularity and
             -- total_tracks = 0. the COALESCE / CASE guard keeps the better data a
             -- saved album sync put there instead of nuking it
             image_url    = COALESCE(excluded.image_url, albums.image_url),
             release_date = COALESCE(excluded.release_date, albums.release_date),
             total_tracks = CASE WHEN excluded.total_tracks > 0
                                 THEN excluded.total_tracks ELSE albums.total_tracks END,
             genres       = COALESCE(excluded.genres, albums.genres),
             popularity   = COALESCE(excluded.popularity, albums.popularity),
             updated_at   = excluded.updated_at",
    )
    .bind(&a.id)
    .bind(&a.name)
    .bind(&a.album_type)
    .bind(&a.image_url)
    .bind(&a.release_date)
    .bind(a.total_tracks)
    .bind(&a.genres)
    .bind(a.popularity)
    .bind(super::now_ms())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find(pool: &SqlitePool, id: &str) -> Result<Option<Album>, AppError> {
    Ok(
        sqlx::query_as::<_, Album>("SELECT * FROM albums WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let rows = sqlx::query("DELETE FROM albums WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

/// hook an artist up to an album. just ignores dupes quietly
pub async fn add_artist(
    pool:      &SqlitePool,
    album_id:  &str,
    artist_id: &str,
    position:  i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT OR IGNORE INTO album_artists (album_id, artist_id, position) VALUES (?, ?, ?)",
    )
    .bind(album_id)
    .bind(artist_id)
    .bind(position)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Album {
        Album {
            id:           "al1".to_string(),
            name:         "Abbey Road".to_string(),
            album_type:   "album".to_string(),
            image_url:    None,
            release_date: Some("1969-09-26".to_string()),
            total_tracks: 17,
            genres:       None,
            popularity:   Some(95),
            updated_at:   0,
        }
    }

    #[sqlx::test]
    async fn upsert_and_find(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let al = find(&pool, "al1").await.unwrap().unwrap();
        assert_eq!(al.name, "Abbey Road");
        assert_eq!(al.total_tracks, 17);
    }

    #[sqlx::test]
    async fn upsert_updates_total_tracks(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        let updated = Album { total_tracks: 18, ..sample() };
        upsert(&pool, &updated).await.unwrap();
        let al = find(&pool, "al1").await.unwrap().unwrap();
        assert_eq!(al.total_tracks, 18);
    }

    #[sqlx::test]
    async fn delete_removes_row(pool: SqlitePool) {
        upsert(&pool, &sample()).await.unwrap();
        assert!(delete(&pool, "al1").await.unwrap());
        assert!(find(&pool, "al1").await.unwrap().is_none());
    }
}
