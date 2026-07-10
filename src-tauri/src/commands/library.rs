use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    library::{sync_all, SyncResult},
    spotify::types::{AlbumItem, ArtistItem, PlaylistDetail, TrackItem},
    state::AppState,
};

#[derive(Debug, Clone, Serialize)]
pub struct PlaylistSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub total_tracks: i64,
    pub snapshot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryStatus {
    pub last_synced: Option<i64>,
    pub is_syncing: bool,
}

#[tauri::command]
pub async fn sync_library(app: AppHandle) -> Result<SyncResult, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    sync_all(&pool, &token).await
}

#[tauri::command]
pub async fn get_liked_songs(
    app: AppHandle,
    limit: i64,
    offset: i64,
) -> Result<Vec<TrackItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let rows = sqlx::query_as::<_, LikedTrackRow>(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.popularity, t.preview_url,
                t.album_id, al.name AS album_name, al.album_type, al.image_url AS album_image,
                al.release_date
         FROM saved_tracks st
         JOIN tracks t ON t.id = st.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         ORDER BY st.added_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let artist_rows = sqlx::query_as::<_, ArtistRow>(
            "SELECT a.id, a.name, a.image_url, a.popularity
             FROM track_artists ta
             JOIN artists a ON a.id = ta.artist_id
             WHERE ta.track_id = ?
             ORDER BY ta.position",
        )
        .bind(&row.id)
        .fetch_all(&pool)
        .await?;

        result.push(TrackItem {
            id: row.id,
            name: row.name,
            duration_ms: row.duration_ms,
            explicit: row.explicit,
            popularity: row.popularity,
            artists: artist_rows
                .into_iter()
                .map(|a| ArtistItem {
                    id: a.id,
                    name: a.name,
                    image_url: a.image_url,
                    popularity: a.popularity,
                })
                .collect(),
            album: row.album_id.map(|aid| AlbumItem {
                id: aid,
                name: row.album_name.unwrap_or_default(),
                album_type: row.album_type.unwrap_or_default(),
                image_url: row.album_image,
                release_date: row.release_date,
                artists: vec![],
                popularity: None,
            }),
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_liked_songs_count(app: AppHandle) -> Result<i64, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM saved_tracks")
        .fetch_one(&pool)
        .await?;
    Ok(row.0)
}

#[tauri::command]
pub async fn get_my_playlists(app: AppHandle) -> Result<Vec<PlaylistSummary>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let rows = sqlx::query_as::<_, PlaylistRow>(
        "SELECT id, name, description, image_url, total_tracks, snapshot_id
         FROM playlists
         ORDER BY updated_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlaylistSummary {
            id: r.id,
            name: r.name,
            description: r.description,
            image_url: r.image_url,
            total_tracks: r.total_tracks,
            snapshot_id: r.snapshot_id,
        })
        .collect())
}

#[tauri::command]
pub async fn get_saved_albums(app: AppHandle) -> Result<Vec<AlbumItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let rows = sqlx::query_as::<_, SavedAlbumRow>(
        "SELECT al.id, al.name, al.album_type, al.image_url, al.release_date, al.popularity
         FROM saved_albums sa
         JOIN albums al ON al.id = sa.album_id
         ORDER BY sa.added_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AlbumItem {
            id: r.id,
            name: r.name,
            album_type: r.album_type,
            image_url: r.image_url,
            release_date: r.release_date,
            artists: vec![],
            popularity: r.popularity,
        })
        .collect())
}

#[tauri::command]
pub async fn get_followed_artists(app: AppHandle) -> Result<Vec<ArtistItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let rows = sqlx::query_as::<_, ArtistRow>(
        "SELECT a.id, a.name, a.image_url, a.popularity
         FROM followed_artists fa
         JOIN artists a ON a.id = fa.artist_id
         ORDER BY a.name COLLATE NOCASE",
    )
    .fetch_all(&pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ArtistItem {
            id: r.id,
            name: r.name,
            image_url: r.image_url,
            popularity: r.popularity,
        })
        .collect())
}

const BASE: &str = "https://api.spotify.com/v1";

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// like a track, save it on spotify + show it in the local cache right away
#[tauri::command]
pub async fn save_track(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::PUT,
        &format!("{BASE}/me/tracks?ids={id}"),
    )
    .await?;

    // best effort local mirror, the track row should already exist from an earlier fetch
    let _ = sqlx::query("INSERT OR IGNORE INTO saved_tracks (track_id, added_at) VALUES (?, ?)")
        .bind(&id)
        .bind(now_ms())
        .execute(&pool)
        .await;

    Ok(())
}

/// unlike a track.
#[tauri::command]
pub async fn unsave_track(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::DELETE,
        &format!("{BASE}/me/tracks?ids={id}"),
    )
    .await?;

    let _ = sqlx::query("DELETE FROM saved_tracks WHERE track_id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    Ok(())
}

/// follow an artist, follow on spotify + mirror it locally
#[tauri::command]
pub async fn follow_artist(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::PUT,
        &format!("{BASE}/me/following?type=artist&ids={id}"),
    )
    .await?;

    let _ = sqlx::query(
        "INSERT OR IGNORE INTO followed_artists (artist_id, followed_at) VALUES (?, ?)",
    )
    .bind(&id)
    .bind(now_ms())
    .execute(&pool)
    .await;

    Ok(())
}

/// unfollow an artist.
#[tauri::command]
pub async fn unfollow_artist(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::DELETE,
        &format!("{BASE}/me/following?type=artist&ids={id}"),
    )
    .await?;

    let _ = sqlx::query("DELETE FROM followed_artists WHERE artist_id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    Ok(())
}

/// is this artist followed? straight from the local cache (fast, offline)
#[tauri::command]
pub async fn is_artist_followed(app: AppHandle, id: String) -> Result<bool, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let row: Option<(String,)> =
        sqlx::query_as("SELECT artist_id FROM followed_artists WHERE artist_id = ?")
            .bind(&id)
            .fetch_optional(&pool)
            .await?;
    Ok(row.is_some())
}

/// which of these track ids are liked? local cache only (fast, offline)
#[tauri::command]
pub async fn get_saved_track_ids(
    app: AppHandle,
    ids: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT track_id FROM saved_tracks WHERE track_id IN ({placeholders})");
    let mut q = sqlx::query_as::<_, (String,)>(&sql);
    for id in &ids {
        q = q.bind(id);
    }

    let rows = q.fetch_all(&pool).await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

// ─── playlist mutations, write through to spotify ────────────────────────────

/// add a track to a playlist on spotify. pass a bare track id, we build the uri
#[tauri::command]
pub async fn add_track_to_playlist(
    app: AppHandle,
    playlist_id: String,
    track_id: String,
) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::POST,
        &format!("{BASE}/playlists/{playlist_id}/tracks"),
        serde_json::json!({ "uris": [format!("spotify:track:{track_id}")] }),
    )
    .await?;
    Ok(())
}

/// yeet every occurrence of a track from a playlist on spotify
#[tauri::command]
pub async fn remove_track_from_playlist(
    app: AppHandle,
    playlist_id: String,
    track_id: String,
) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::DELETE,
        &format!("{BASE}/playlists/{playlist_id}/tracks"),
        serde_json::json!({ "tracks": [{ "uri": format!("spotify:track:{track_id}") }] }),
    )
    .await?;

    // mirror it locally so the playlist page updates without a whole re-sync
    let _ = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
        .bind(&playlist_id)
        .bind(&track_id)
        .execute(&pool)
        .await;

    Ok(())
}

/// make a new playlist for the current user, hands back its id
#[tauri::command]
pub async fn create_playlist(
    app: AppHandle,
    name: String,
    description: Option<String>,
    public: bool,
) -> Result<String, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("Playlist name is required".into()));
    }

    let user_id = crate::auth::get_setting_value(&pool, "spotify_user_id")
        .await?
        .ok_or_else(|| AppError::Auth("No user id - log in first".into()))?;

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    let resp = crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::POST,
        &format!("{BASE}/users/{user_id}/playlists"),
        serde_json::json!({
            "name": name,
            "description": description.unwrap_or_default(),
            "public": public,
        }),
    )
    .await?;

    resp.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Network("create playlist: no id returned".into()))
}

#[tauri::command]
pub async fn get_library_status(app: AppHandle) -> Result<LibraryStatus, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'library_last_synced'")
            .fetch_optional(&pool)
            .await?;

    let last_synced = row.and_then(|(v,)| v.parse::<i64>().ok());

    Ok(LibraryStatus {
        last_synced,
        is_syncing: false,
    })
}

// ─── cache first helpers, shared with the spotify command module ─────────────

/// build a full `TrackItem` outta the local cache (track + album + artists)
pub(crate) async fn load_track_item(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<TrackItem>, AppError> {
    let row = sqlx::query_as::<_, LikedTrackRow>(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.popularity, t.preview_url,
                t.album_id, al.name AS album_name, al.album_type, al.image_url AS album_image,
                al.release_date
         FROM tracks t
         LEFT JOIN albums al ON al.id = t.album_id
         WHERE t.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };

    let artist_rows = sqlx::query_as::<_, ArtistRow>(
        "SELECT a.id, a.name, a.image_url, a.popularity
         FROM track_artists ta
         JOIN artists a ON a.id = ta.artist_id
         WHERE ta.track_id = ?
         ORDER BY ta.position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(Some(TrackItem {
        id: row.id,
        name: row.name,
        duration_ms: row.duration_ms,
        explicit: row.explicit,
        popularity: row.popularity,
        artists: artist_rows
            .into_iter()
            .map(|a| ArtistItem {
                id: a.id,
                name: a.name,
                image_url: a.image_url,
                popularity: a.popularity,
            })
            .collect(),
        album: row.album_id.map(|aid| AlbumItem {
            id: aid,
            name: row.album_name.unwrap_or_default(),
            album_type: row.album_type.unwrap_or_default(),
            image_url: row.album_image,
            release_date: row.release_date,
            artists: vec![],
            popularity: None,
        }),
    }))
}

/// the whole playlist (meta + ordered tracks) from the local cache, or `None`
pub(crate) async fn load_cached_playlist(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<PlaylistDetail>, AppError> {
    let meta: Option<(
        String,
        String,
        Option<String>,
        Option<String>,
        i64,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, name, description, image_url, total_tracks, owner_id
             FROM playlists WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some((pid, name, description, image_url, total_tracks, owner_id)) = meta else {
        return Ok(None);
    };

    let ids: Vec<(String,)> = sqlx::query_as(
        "SELECT track_id FROM playlist_tracks WHERE playlist_id = ? ORDER BY position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    if ids.is_empty() {
        return Ok(None);
    }

    let mut tracks = Vec::with_capacity(ids.len());
    for (tid,) in ids {
        if let Some(t) = load_track_item(pool, &tid).await? {
            tracks.push(t);
        }
    }

    Ok(Some(PlaylistDetail {
        id: pid,
        name,
        description: description.filter(|s| !s.is_empty()),
        image_url,
        owner_name: owner_id.filter(|s| !s.is_empty()),
        total_tracks,
        tracks,
    }))
}

// ─── discovery read commands, cache first, offline safe ──────────────────────

#[tauri::command]
pub async fn get_top_tracks(
    app: AppHandle,
    time_range: Option<String>,
) -> Result<Vec<TrackItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let range = time_range.unwrap_or_else(|| "medium_term".into());
    let ids: Vec<(String,)> =
        sqlx::query_as("SELECT track_id FROM top_tracks WHERE time_range = ? ORDER BY position")
            .bind(&range)
            .fetch_all(&pool)
            .await?;

    let mut out = Vec::with_capacity(ids.len());
    for (id,) in ids {
        if let Some(t) = load_track_item(&pool, &id).await? {
            out.push(t);
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn get_top_artists(
    app: AppHandle,
    time_range: Option<String>,
) -> Result<Vec<ArtistItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let range = time_range.unwrap_or_else(|| "medium_term".into());
    let rows = sqlx::query_as::<_, ArtistRow>(
        "SELECT a.id, a.name, a.image_url, a.popularity
         FROM top_artists ta
         JOIN artists a ON a.id = ta.artist_id
         WHERE ta.time_range = ?
         ORDER BY ta.position",
    )
    .bind(&range)
    .fetch_all(&pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ArtistItem {
            id: r.id,
            name: r.name,
            image_url: r.image_url,
            popularity: r.popularity,
        })
        .collect())
}

#[tauri::command]
pub async fn get_recently_played(app: AppHandle) -> Result<Vec<TrackItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    // distinct tracks, the most recently played one first
    let ids: Vec<(String,)> = sqlx::query_as(
        "SELECT track_id FROM recently_played
         GROUP BY track_id
         ORDER BY MAX(played_at) DESC
         LIMIT 50",
    )
    .fetch_all(&pool)
    .await?;

    let mut out = Vec::with_capacity(ids.len());
    for (id,) in ids {
        if let Some(t) = load_track_item(&pool, &id).await? {
            out.push(t);
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn get_new_releases(app: AppHandle) -> Result<Vec<AlbumItem>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let rows = sqlx::query_as::<_, SavedAlbumRow>(
        "SELECT al.id, al.name, al.album_type, al.image_url, al.release_date, al.popularity
         FROM new_releases nr
         JOIN albums al ON al.id = nr.album_id
         ORDER BY nr.position",
    )
    .fetch_all(&pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let artist_rows = sqlx::query_as::<_, ArtistRow>(
            "SELECT a.id, a.name, a.image_url, a.popularity
             FROM album_artists aa
             JOIN artists a ON a.id = aa.artist_id
             WHERE aa.album_id = ?
             ORDER BY aa.position",
        )
        .bind(&r.id)
        .fetch_all(&pool)
        .await?;

        out.push(AlbumItem {
            id: r.id,
            name: r.name,
            album_type: r.album_type,
            image_url: r.image_url,
            release_date: r.release_date,
            artists: artist_rows
                .into_iter()
                .map(|a| ArtistItem {
                    id: a.id,
                    name: a.name,
                    image_url: a.image_url,
                    popularity: a.popularity,
                })
                .collect(),
            popularity: r.popularity,
        });
    }
    Ok(out)
}

/// playlist straight from cache, instant first paint, returns `None` if we never saw it
#[tauri::command]
pub async fn get_cached_playlist(
    app: AppHandle,
    id: String,
) -> Result<Option<PlaylistDetail>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    load_cached_playlist(&pool, &id).await
}

#[derive(sqlx::FromRow)]
struct LikedTrackRow {
    id: String,
    name: String,
    duration_ms: i64,
    explicit: bool,
    popularity: Option<i64>,
    #[allow(dead_code)]
    preview_url: Option<String>,
    album_id: Option<String>,
    album_name: Option<String>,
    album_type: Option<String>,
    album_image: Option<String>,
    release_date: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ArtistRow {
    id: String,
    name: String,
    image_url: Option<String>,
    popularity: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct SavedAlbumRow {
    id: String,
    name: String,
    album_type: String,
    image_url: Option<String>,
    release_date: Option<String>,
    popularity: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct PlaylistRow {
    id: String,
    name: String,
    description: Option<String>,
    image_url: Option<String>,
    total_tracks: i64,
    snapshot_id: Option<String>,
}
