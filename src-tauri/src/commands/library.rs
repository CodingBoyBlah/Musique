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
    let gate = s.sync_gate.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    // held for the whole sync so a sign-out waits for it instead of purging
    // underneath it - see `AppState::sync_gate`
    let _syncing = gate.read().await;
    sync_all(&pool, &token).await
}

fn make_placeholders(count: usize) -> String {
    let mut value = "?,".repeat(count);
    value.pop();
    value
}

pub(crate) async fn fetch_artists_for_tracks(
    pool: &SqlitePool,
    track_ids: &[&str],
) -> Result<std::collections::HashMap<String, Vec<ArtistItem>>, AppError> {
    let mut artists_by_track: std::collections::HashMap<String, Vec<ArtistItem>> =
        std::collections::HashMap::new();
    if track_ids.is_empty() {
        return Ok(artists_by_track);
    }

    let mut deduped: Vec<&str> = Vec::with_capacity(track_ids.len());
    let mut seen = std::collections::HashSet::new();
    for &id in track_ids {
        if seen.insert(id) {
            deduped.push(id);
        }
    }

    for chunk in deduped.chunks(200) {
        let placeholders = make_placeholders(chunk.len());
        let sql = format!(
            "SELECT ta.track_id, a.id, a.name, a.image_url, a.popularity
             FROM track_artists ta
             JOIN artists a ON a.id = ta.artist_id
             WHERE ta.track_id IN ({placeholders})
             ORDER BY ta.position"
        );
        let mut query = sqlx::query_as::<_, (String, String, String, Option<String>, Option<i64>)>(&sql);
        for id in chunk {
            query = query.bind(id);
        }
        let rows = query.fetch_all(pool).await?;
        for (track_id, aid, aname, aimage, apop) in rows {
            artists_by_track.entry(track_id).or_default().push(ArtistItem {
                id: aid,
                name: aname,
                image_url: aimage,
                popularity: apop,
            });
        }
    }

    Ok(artists_by_track)
}

pub(crate) async fn fetch_artists_for_albums(
    pool: &SqlitePool,
    album_ids: &[&str],
) -> Result<std::collections::HashMap<String, Vec<ArtistItem>>, AppError> {
    let mut artists_by_album: std::collections::HashMap<String, Vec<ArtistItem>> =
        std::collections::HashMap::new();
    if album_ids.is_empty() {
        return Ok(artists_by_album);
    }

    let mut deduped: Vec<&str> = Vec::with_capacity(album_ids.len());
    let mut seen = std::collections::HashSet::new();
    for &id in album_ids {
        if seen.insert(id) {
            deduped.push(id);
        }
    }

    for chunk in deduped.chunks(200) {
        let placeholders = make_placeholders(chunk.len());
        let sql = format!(
            "SELECT aa.album_id, a.id, a.name, a.image_url, a.popularity
             FROM album_artists aa
             JOIN artists a ON a.id = aa.artist_id
             WHERE aa.album_id IN ({placeholders})
             ORDER BY aa.position"
        );
        let mut query = sqlx::query_as::<_, (String, String, String, Option<String>, Option<i64>)>(&sql);
        for id in chunk {
            query = query.bind(id);
        }
        let rows = query.fetch_all(pool).await?;
        for (album_id, aid, aname, aimage, apop) in rows {
            artists_by_album.entry(album_id).or_default().push(ArtistItem {
                id: aid,
                name: aname,
                image_url: aimage,
                popularity: apop,
            });
        }
    }

    Ok(artists_by_album)
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

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let track_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let mut artists_by_track = fetch_artists_for_tracks(&pool, &track_ids).await?;

    let result = rows
        .into_iter()
        .map(|row| TrackItem {
            artists: artists_by_track.remove(&row.id).unwrap_or_default(),
            album: row.album_id.map(|aid| AlbumItem {
                id: aid,
                name: row.album_name.unwrap_or_default(),
                album_type: row.album_type.unwrap_or_default(),
                image_url: row.album_image,
                release_date: row.release_date,
                artists: vec![],
                popularity: None,
            }),
            id: row.id,
            name: row.name,
            duration_ms: row.duration_ms,
            explicit: row.explicit,
            popularity: row.popularity,
        })
        .collect();

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

/// save an album to the library, on spotify + in the local mirror
#[tauri::command]
pub async fn save_album(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::PUT,
        &format!("{BASE}/me/albums?ids={id}"),
    )
    .await?;

    // the album row exists from the page that offered the button; if it
    // somehow doesn't, the FK just skips the mirror and the next sync fills it
    let _ = sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
        .bind(&id)
        .bind(now_ms())
        .execute(&pool)
        .await;

    Ok(())
}

/// remove an album from the library
#[tauri::command]
pub async fn unsave_album(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::DELETE,
        &format!("{BASE}/me/albums?ids={id}"),
    )
    .await?;

    let _ = sqlx::query("DELETE FROM saved_albums WHERE album_id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    Ok(())
}

/// is this album in the library? asks spotify (`/me/albums/contains`) so an
/// album saved from the phone since the last sync still shows as saved, and
/// falls back to the local mirror offline
#[tauri::command]
pub async fn is_album_saved(app: AppHandle, id: String) -> Result<bool, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let local = sqlx::query_as::<_, (i32,)>("SELECT 1 FROM saved_albums WHERE album_id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .is_some();

    let Ok(token) = crate::auth::get_valid_token(&pool, &auth).await else {
        return Ok(local);
    };
    match crate::spotify::spotify_get::<Vec<bool>>(&token, &format!("{BASE}/me/albums/contains?ids={id}")).await {
        Ok(v) => {
            let live = v.first().copied().unwrap_or(local);
            if live && !local {
                let _ = sqlx::query("INSERT OR IGNORE INTO saved_albums (album_id, added_at) VALUES (?, ?)")
                    .bind(&id)
                    .bind(now_ms())
                    .execute(&pool)
                    .await;
            } else if !live && local {
                let _ = sqlx::query("DELETE FROM saved_albums WHERE album_id = ?")
                    .bind(&id)
                    .execute(&pool)
                    .await;
            }
            Ok(live)
        }
        Err(_) => Ok(local),
    }
}

/// is this artist followed? straight from the local cache (fast, offline)
#[tauri::command]
pub async fn is_artist_followed(app: AppHandle, id: String) -> Result<bool, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM followed_artists WHERE artist_id = ?")
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

    let placeholders = make_placeholders(ids.len());
    let sql = format!("SELECT track_id FROM saved_tracks WHERE track_id IN ({placeholders})");
    let mut q = sqlx::query_as::<_, (String,)>(&sql);
    for id in &ids {
        q = q.bind(id);
    }

    let rows = q.fetch_all(&pool).await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

// ─── playlist mutations, write through to spotify ────────────────────────────

/// rename / redescribe / change sharing on a playlist you own. every field is
/// optional so the caller only sends what changed
#[tauri::command]
pub async fn update_playlist_details(
    app: AppHandle,
    id: String,
    name: Option<String>,
    description: Option<String>,
    public: Option<bool>,
    collaborative: Option<bool>,
) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let mut body = serde_json::Map::new();
    if let Some(n) = name.as_ref().map(|n| n.trim()).filter(|n| !n.is_empty()) {
        body.insert("name".into(), n.into());
    }
    if let Some(d) = &description {
        body.insert("description".into(), d.trim().into());
    }
    // spotify rejects collaborative + public together, so a collaborative
    // playlist is always sent private
    let collab = collaborative.unwrap_or(false);
    if let Some(p) = public {
        body.insert("public".into(), (p && !collab).into());
    }
    if let Some(c) = collaborative {
        body.insert("collaborative".into(), c.into());
        if c {
            body.insert("public".into(), false.into());
        }
    }
    if body.is_empty() {
        return Ok(());
    }

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::PUT,
        &format!("{BASE}/playlists/{id}"),
        serde_json::Value::Object(body),
    )
    .await?;

    let _ = sqlx::query(
        "UPDATE playlists SET
             name        = COALESCE(?, name),
             description = COALESCE(?, description),
             updated_at  = ?
         WHERE id = ?",
    )
    .bind(name.as_ref().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()))
    .bind(description.as_ref().map(|d| d.trim().to_string()))
    .bind(now_ms())
    .bind(&id)
    .execute(&pool)
    .await;

    Ok(())
}

/// follow someone else's playlist (adds it to your library)
#[tauri::command]
pub async fn follow_playlist(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::PUT,
        &format!("{BASE}/playlists/{id}/followers"),
        serde_json::json!({ "public": true }),
    )
    .await?;
    Ok(())
}

/// unfollow a playlist. on one you own this IS delete - spotify has no hard
/// delete, the owner unfollowing is what removes it from everywhere
#[tauri::command]
pub async fn unfollow_playlist(app: AppHandle, id: String) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    crate::spotify::spotify_write(
        &token,
        reqwest::Method::DELETE,
        &format!("{BASE}/playlists/{id}/followers"),
    )
    .await?;

    // drop it from the local library so the sidebar/list forget it right away
    let _ = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
        .bind(&id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await;
    Ok(())
}

/// does the signed-in user follow this playlist?
#[tauri::command]
pub async fn is_playlist_followed(app: AppHandle, id: String) -> Result<bool, AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    let v: Vec<bool> =
        crate::spotify::spotify_get(&token, &format!("{BASE}/playlists/{id}/followers/contains")).await?;
    Ok(v.first().copied().unwrap_or(false))
}

/// add a track to a playlist on spotify. pass a bare track id, we build the uri
#[tauri::command]
pub async fn add_track_to_playlist(
    app: AppHandle,
    playlist_id: String,
    track_id: String,
    // zero-based insert index. None appends. Used by "Undo" after a remove so
    // the track goes back where it was rather than to the bottom.
    position: Option<u32>,
) -> Result<(), AppError> {
    let s = app.state::<AppState>();
    let pool = s.db.clone();
    let auth = s.auth.clone();
    drop(s);

    let token = crate::auth::get_valid_token(&pool, &auth).await?;
    let mut body = serde_json::json!({ "uris": [format!("spotify:track:{track_id}")] });
    if let Some(pos) = position {
        body["position"] = serde_json::json!(pos);
    }
    crate::spotify::spotify_write_json(
        &token,
        reqwest::Method::POST,
        &format!("{BASE}/playlists/{playlist_id}/tracks"),
        body,
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

    let track_rows = sqlx::query_as::<_, LikedTrackRow>(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.popularity, t.preview_url,
                t.album_id, al.name AS album_name, al.album_type, al.image_url AS album_image,
                al.release_date
         FROM playlist_tracks pt
         JOIN tracks t ON t.id = pt.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         WHERE pt.playlist_id = ?
         ORDER BY pt.position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    if track_rows.is_empty() {
        return Ok(None);
    }

    let track_ids: Vec<&str> = track_rows.iter().map(|r| r.id.as_str()).collect();
    let mut artists_by_track = fetch_artists_for_tracks(pool, &track_ids).await?;

    let tracks: Vec<TrackItem> = track_rows
        .into_iter()
        .map(|row| TrackItem {
            artists: artists_by_track.remove(&row.id).unwrap_or_default(),
            album: row.album_id.map(|aid| AlbumItem {
                id: aid,
                name: row.album_name.unwrap_or_default(),
                album_type: row.album_type.unwrap_or_default(),
                image_url: row.album_image,
                release_date: row.release_date,
                artists: vec![],
                popularity: None,
            }),
            id: row.id,
            name: row.name,
            duration_ms: row.duration_ms,
            explicit: row.explicit,
            popularity: row.popularity,
        })
        .collect();

    Ok(Some(PlaylistDetail {
        id: pid,
        name,
        description: description.filter(|s| !s.is_empty()),
        image_url,
        owner_name: owner_id.filter(|s| !s.is_empty()),
        total_tracks,
        tracks,
        owner_id: None,
        public: None,
        collaborative: None,
        followers: None,
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

    let track_rows = sqlx::query_as::<_, LikedTrackRow>(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.popularity, t.preview_url,
                t.album_id, al.name AS album_name, al.album_type, al.image_url AS album_image,
                al.release_date
         FROM top_tracks tt
         JOIN tracks t ON t.id = tt.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         WHERE tt.time_range = ?
         ORDER BY tt.position",
    )
    .bind(&range)
    .fetch_all(&pool)
    .await?;

    if track_rows.is_empty() {
        return Ok(Vec::new());
    }

    let track_ids: Vec<&str> = track_rows.iter().map(|r| r.id.as_str()).collect();
    let mut artists_by_track = fetch_artists_for_tracks(&pool, &track_ids).await?;

    let out = track_rows
        .into_iter()
        .map(|row| TrackItem {
            artists: artists_by_track.remove(&row.id).unwrap_or_default(),
            album: row.album_id.map(|aid| AlbumItem {
                id: aid,
                name: row.album_name.unwrap_or_default(),
                album_type: row.album_type.unwrap_or_default(),
                image_url: row.album_image,
                release_date: row.release_date,
                artists: vec![],
                popularity: None,
            }),
            id: row.id,
            name: row.name,
            duration_ms: row.duration_ms,
            explicit: row.explicit,
            popularity: row.popularity,
        })
        .collect();

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
    let track_rows = sqlx::query_as::<_, LikedTrackRow>(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.popularity, t.preview_url,
                t.album_id, al.name AS album_name, al.album_type, al.image_url AS album_image,
                al.release_date
         FROM (
             SELECT track_id, MAX(played_at) as max_played
             FROM recently_played
             GROUP BY track_id
             ORDER BY max_played DESC
             LIMIT 50
         ) rp
         JOIN tracks t ON t.id = rp.track_id
         LEFT JOIN albums al ON al.id = t.album_id
         ORDER BY rp.max_played DESC",
    )
    .fetch_all(&pool)
    .await?;

    if track_rows.is_empty() {
        return Ok(Vec::new());
    }

    let track_ids: Vec<&str> = track_rows.iter().map(|r| r.id.as_str()).collect();
    let mut artists_by_track = fetch_artists_for_tracks(&pool, &track_ids).await?;

    let out = track_rows
        .into_iter()
        .map(|row| TrackItem {
            artists: artists_by_track.remove(&row.id).unwrap_or_default(),
            album: row.album_id.map(|aid| AlbumItem {
                id: aid,
                name: row.album_name.unwrap_or_default(),
                album_type: row.album_type.unwrap_or_default(),
                image_url: row.album_image,
                release_date: row.release_date,
                artists: vec![],
                popularity: None,
            }),
            id: row.id,
            name: row.name,
            duration_ms: row.duration_ms,
            explicit: row.explicit,
            popularity: row.popularity,
        })
        .collect();

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

    let album_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let mut artists_by_album = fetch_artists_for_albums(&pool, &album_ids).await?;

    let out = rows
        .into_iter()
        .map(|r| {
            let artists = artists_by_album.remove(&r.id).unwrap_or_default();
            AlbumItem {
                id: r.id,
                name: r.name,
                album_type: r.album_type,
                image_url: r.image_url,
                release_date: r.release_date,
                artists,
                popularity: r.popularity,
            }
        })
        .collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_fetch_artists_for_tracks_empty(pool: SqlitePool) {
        let res = fetch_artists_for_tracks(&pool, &[]).await.unwrap();
        assert!(res.is_empty());
    }

    #[sqlx::test]
    async fn test_fetch_artists_for_albums_empty(pool: SqlitePool) {
        let res = fetch_artists_for_albums(&pool, &[]).await.unwrap();
        assert!(res.is_empty());
    }

    #[sqlx::test]
    async fn test_fetch_artists_for_tracks_batched(pool: SqlitePool) {
        sqlx::query("INSERT INTO artists (id, name, updated_at) VALUES ('a1', 'Artist 1', 0), ('a2', 'Artist 2', 0)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tracks (id, name, duration_ms, track_number, disc_number, explicit, is_local, updated_at) VALUES ('t1', 'Track 1', 1000, 1, 1, 0, 0, 0), ('t2', 'Track 2', 2000, 2, 1, 0, 0, 0)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO track_artists (track_id, artist_id, position) VALUES ('t1', 'a1', 0), ('t1', 'a2', 1), ('t2', 'a2', 0)")
            .execute(&pool)
            .await
            .unwrap();

        let map = fetch_artists_for_tracks(&pool, &["t1", "t2"]).await.unwrap();
        assert_eq!(map.len(), 2);
        let t1_artists = map.get("t1").unwrap();
        assert_eq!(t1_artists.len(), 2);
        assert_eq!(t1_artists[0].name, "Artist 1");
        assert_eq!(t1_artists[1].name, "Artist 2");

        let t2_artists = map.get("t2").unwrap();
        assert_eq!(t2_artists.len(), 1);
        assert_eq!(t2_artists[0].name, "Artist 2");

        sqlx::query("INSERT INTO playlists (id, name, total_tracks, updated_at) VALUES ('p1', 'Mix', 2, 0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at) VALUES ('p1', 't2', 0, 0), ('p1', 't1', 1, 0)")
            .execute(&pool).await.unwrap();
        let playlist = load_cached_playlist(&pool, "p1").await.unwrap().unwrap();
        assert_eq!(playlist.tracks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), vec!["t2", "t1"]);
        assert_eq!(playlist.tracks[0].artists[0].name, "Artist 2");
        assert_eq!(playlist.tracks[1].artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["Artist 1", "Artist 2"]);

    }
}
