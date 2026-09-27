use crate::db::repos::{albums, artists, tracks};
use crate::errors::AppError;
use crate::spotify::{self, types::*};
use crate::state::AppState;
use sqlx::SqlitePool;
use std::collections::HashSet;
use tauri::{AppHandle, Manager};

pub(crate) const BASE: &str = "https://api.spotify.com/v1";

// token helper thing

pub(crate) async fn tok(app: &AppHandle) -> Result<String, AppError> {
    let s    = app.state::<AppState>();
    let db   = s.db.clone();
    let auth = s.auth.clone();
    crate::auth::get_valid_token(&db, &auth).await
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// conversion helper stuff

fn item_from_simple(a: &SpArtistSimple) -> ArtistItem {
    ArtistItem { id: a.id.clone(), name: a.name.clone(), image_url: None, popularity: None }
}

fn item_from_artist(a: &SpArtist) -> ArtistItem {
    ArtistItem {
        id:         a.id.clone(),
        name:       a.name.clone(),
        image_url:  a.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        popularity: a.popularity,
    }
}

pub(crate) fn item_from_album_simple(al: &SpAlbumSimple) -> AlbumItem {
    AlbumItem {
        id:           al.id.clone(),
        name:         al.name.clone(),
        album_type:   al.album_type.clone(),
        image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        artists:      al.artists.as_ref()
                        .map(|v| v.iter().map(item_from_simple).collect())
                        .unwrap_or_default(),
        popularity:   None,
    }
}

pub(crate) fn item_from_track(t: &SpTrack) -> TrackItem {
    TrackItem {
        id:          t.id.clone(),
        name:        t.name.clone(),
        duration_ms: t.duration_ms,
        explicit:    t.explicit,
        artists:     t.artists.iter().map(item_from_simple).collect(),
        album:       t.album.as_ref().map(item_from_album_simple),
        popularity:  t.popularity,
    }
}

pub(crate) fn item_from_album_track(t: &SpAlbumTrack) -> TrackItem {
    TrackItem {
        id:          t.id.clone(),
        name:        t.name.clone(),
        duration_ms: t.duration_ms,
        explicit:    t.explicit,
        artists:     t.artists.iter().map(item_from_simple).collect(),
        album:       None,
        popularity:  None,
    }
}

// db upsert helper stuff

async fn upsert_artist_full(pool: &SqlitePool, a: &SpArtist) -> Result<(), AppError> {
    artists::upsert(pool, &artists::Artist {
        id:         a.id.clone(),
        name:       a.name.clone(),
        image_url:  a.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        genres:     a.genres.as_ref()
                      .filter(|g| !g.is_empty())
                      .map(|g| serde_json::to_string(g).unwrap_or_default()),
        popularity: a.popularity,
        updated_at: 0,
    })
    .await
}

async fn upsert_album_simple(pool: &SqlitePool, al: &SpAlbumSimple) -> Result<(), AppError> {
    albums::upsert(pool, &albums::Album {
        id:           al.id.clone(),
        name:         al.name.clone(),
        album_type:   al.album_type.clone(),
        image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        total_tracks: 0,
        genres:       None,
        popularity:   None,
        updated_at:   0,
    })
    .await
}

async fn upsert_album_full(pool: &SqlitePool, al: &SpAlbum) -> Result<(), AppError> {
    albums::upsert(pool, &albums::Album {
        id:           al.id.clone(),
        name:         al.name.clone(),
        album_type:   al.album_type.clone(),
        image_url:    al.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: al.release_date.clone(),
        total_tracks: al.total_tracks.unwrap_or(0),
        genres:       al.genres.as_ref()
                        .filter(|g| !g.is_empty())
                        .map(|g| serde_json::to_string(g).unwrap_or_default()),
        popularity:   al.popularity,
        updated_at:   0,
    })
    .await?;

    for (i, artist) in al.artists.as_ref().map(|v| v.as_slice()).unwrap_or(&[]).iter().enumerate() {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO artists (id, name, updated_at) VALUES (?, ?, 0)",
        )
        .bind(&artist.id)
        .bind(&artist.name)
        .execute(pool)
        .await;
        let _ = albums::add_artist(pool, &al.id, &artist.id, i as i64).await;
    }

    if let Some(track_page) = &al.tracks {
        for t in &track_page.items {
            let _ = tracks::upsert(pool, &tracks::Track {
                id:           t.id.clone(),
                name:         t.name.clone(),
                album_id:     Some(al.id.clone()),
                duration_ms:  t.duration_ms,
                track_number: t.track_number.unwrap_or(0),
                disc_number:  t.disc_number.unwrap_or(1),
                explicit:     t.explicit,
                popularity:   None,
                preview_url:  t.preview_url.clone(),
                is_local:     t.is_local.unwrap_or(false),
                updated_at:   0,
                // album track objects carry no external_ids; the upsert
                // COALESCEs so this never clears an isrc we already know
                isrc:         None,
            })
            .await;
        }
    }

    Ok(())
}

pub(crate) async fn upsert_track(pool: &SqlitePool, t: &SpTrack) -> Result<(), AppError> {
    tracks::upsert(pool, &tracks::Track {
        id:           t.id.clone(),
        name:         t.name.clone(),
        album_id:     t.album.as_ref().map(|a| a.id.clone()),
        duration_ms:  t.duration_ms,
        track_number: t.track_number.unwrap_or(0),
        disc_number:  t.disc_number.unwrap_or(1),
        explicit:     t.explicit,
        popularity:   t.popularity,
        preview_url:  t.preview_url.clone(),
        is_local:     t.is_local.unwrap_or(false),
        updated_at:   0,
        isrc:         t.external_ids.as_ref().and_then(|e| e.isrc.clone()),
    })
    .await
}

// ipc commands

#[tauri::command]
pub async fn search(
    app:   AppHandle,
    query: String,
    types: Option<String>,
) -> Result<SearchResults, AppError> {
    let token = tok(&app).await?;
    let t     = types.as_deref().unwrap_or("track,artist,album,playlist");

    let mut url = url::Url::parse(&format!("{BASE}/search")).unwrap();
    url.query_pairs_mut()
        .append_pair("q",     &query)
        .append_pair("type",  t)
        .append_pair("limit", "20");

    let raw: SpSearchResponse = spotify::spotify_get(&token, url.as_str()).await?;

    let result = SearchResults {
        artists: raw.artists.as_ref()
            .map(|p| p.items.iter().map(item_from_artist).collect())
            .unwrap_or_default(),
        albums:  raw.albums.as_ref()
            .map(|p| p.items.iter().map(item_from_album_simple).collect())
            .unwrap_or_default(),
        tracks:  raw.tracks.as_ref()
            .map(|p| p.items.iter().map(item_from_track).collect())
            .unwrap_or_default(),
        playlists: raw.playlists.as_ref()
            .map(|p| p.items.iter().flatten().map(|pl| PlaylistCard {
                id:          pl.id.clone(),
                name:        pl.name.clone(),
                description: pl.description.clone().filter(|s| !s.is_empty()),
                image_url:   pl.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
                owner_name:  pl.owner.as_ref().and_then(|o| o.display_name.clone()),
            }).collect())
            .unwrap_or_default(),
    };

    let pool = &app.state::<AppState>().db.clone();
    for a  in raw.artists.as_ref().map(|p| p.items.as_slice()).unwrap_or(&[]) { let _ = upsert_artist_full(pool, a).await; }
    for al in raw.albums.as_ref().map(|p|  p.items.as_slice()).unwrap_or(&[]) { let _ = upsert_album_simple(pool, al).await; }
    for tr in raw.tracks.as_ref().map(|p|  p.items.as_slice()).unwrap_or(&[]) { let _ = upsert_track(pool, tr).await; }
    let _ = crate::db::repos::search_history::push(pool, &query, None, None).await;

    Ok(result)
}

#[tauri::command]
pub async fn get_artist(app: AppHandle, id: String) -> Result<ArtistDetail, AppError> {
    let token = tok(&app).await?;

    let artist: SpArtist = spotify::spotify_get(&token, &format!("{BASE}/artists/{id}")).await?;

    let mut alb_url = url::Url::parse(&format!("{BASE}/artists/{id}/albums")).unwrap();
    alb_url.query_pairs_mut()
        .append_pair("limit",         "50")
        .append_pair("include_groups", "album,single,compilation");
    let album_page: SpPage<SpAlbumSimple> =
        spotify::spotify_get(&token, alb_url.as_str()).await?;

    // top tracks (best effort, an artist with none still renders the page fine)
    let top_tracks: Vec<TrackItem> = match spotify::spotify_get::<SpTopTracks>(
        &token,
        &format!("{BASE}/artists/{id}/top-tracks?market=from_token"),
    ).await {
        Ok(t)  => t.tracks.iter().map(item_from_track).collect(),
        Err(_) => Vec::new(),
    };

    let pool = &app.state::<AppState>().db.clone();
    let _ = upsert_artist_full(pool, &artist).await;
    for al in &album_page.items { let _ = upsert_album_simple(pool, al).await; }

    // split up the discography, full albums (incl compilations) vs singles/eps
    let mut albums:  Vec<AlbumItem> = Vec::new();
    let mut singles: Vec<AlbumItem> = Vec::new();
    for al in &album_page.items {
        let item = item_from_album_simple(al);
        if al.album_type == "single" { singles.push(item); } else { albums.push(item); }
    }

    Ok(ArtistDetail {
        id:         artist.id.clone(),
        name:       artist.name.clone(),
        image_url:  artist.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        genres:     artist.genres.clone().unwrap_or_default(),
        popularity: artist.popularity,
        albums,
        singles,
        top_tracks,
    })
}

#[tauri::command]
pub async fn get_album(app: AppHandle, id: String) -> Result<AlbumDetail, AppError> {
    let token = tok(&app).await?;

    let mut url = url::Url::parse(&format!("{BASE}/albums/{id}")).unwrap();
    url.query_pairs_mut().append_pair("market", "from_token");
    let album: SpAlbum = spotify::spotify_get(&token, url.as_str()).await?;

    let pool = &app.state::<AppState>().db.clone();
    let _ = upsert_album_full(pool, &album).await;

    // album tracks dont embed their own album object so we stamp the parent
    // album onto each one so the player strip / rows have cover art + album name
    let album_item = AlbumItem {
        id:           album.id.clone(),
        name:         album.name.clone(),
        album_type:   album.album_type.clone(),
        image_url:    album.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: album.release_date.clone(),
        artists:      album.artists.as_ref()
                        .map(|v| v.iter().map(item_from_simple).collect())
                        .unwrap_or_default(),
        popularity:   album.popularity,
    };

    let tracks: Vec<TrackItem> = album.tracks.as_ref()
        .map(|p| p.items.iter().map(|t| {
            let mut item = item_from_album_track(t);
            item.album = Some(album_item.clone());
            item
        }).collect())
        .unwrap_or_default();

    Ok(AlbumDetail {
        id:           album.id.clone(),
        name:         album.name.clone(),
        album_type:   album.album_type.clone(),
        image_url:    album.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: album.release_date.clone(),
        total_tracks: album.total_tracks.unwrap_or(0),
        popularity:   album.popularity,
        artists:      album_item.artists.clone(),
        tracks,
    })
}

#[tauri::command]
pub async fn get_track(app: AppHandle, id: String) -> Result<TrackDetail, AppError> {
    let token = tok(&app).await?;
    let track: SpTrack = spotify::spotify_get(&token, &format!("{BASE}/tracks/{id}")).await?;

    let pool = &app.state::<AppState>().db.clone();
    let _ = upsert_track(pool, &track).await;

    Ok(TrackDetail {
        id:          track.id.clone(),
        name:        track.name.clone(),
        duration_ms: track.duration_ms,
        explicit:    track.explicit,
        popularity:  track.popularity,
        preview_url: track.preview_url.clone(),
        artists:     track.artists.iter().map(item_from_simple).collect(),
        album:       track.album.as_ref().map(item_from_album_simple),
    })
}

pub fn clear_spotify_memory_caches() {
    crate::recommend::clear_caches();
}

// persist one listening signal for the self-improving taste engine. the
// frontend fires this on play / full-listen / skip; the engine memoizes nothing
// that survives a signal, so the next rec request already reflects it.
#[tauri::command]
pub async fn record_listen_event(
    app:          AppHandle,
    track_id:     String,
    event_type:   String,
    ms_played:    Option<i64>,
    duration_ms:  Option<i64>,
    context_type: Option<String>,
    context_id:   Option<String>,
) -> Result<(), AppError> {
    let pool = app.state::<AppState>().db.clone();
    crate::recommend::record_event(
        &pool,
        &track_id,
        &event_type,
        ms_played.unwrap_or(0),
        duration_ms.unwrap_or(0),
        context_type.as_deref(),
        context_id.as_deref(),
    )
    .await
}

// personalized recommendations. the heavy lifting lives in the `recommend`
// module (behavioural taste profile + freshness-first MMR ranking); this just
// validates inputs and hands it the auth token + db pool.
#[tauri::command]
pub async fn get_recommendations(
    app:               AppHandle,
    seed_artist_ids:   Option<Vec<String>>,
    exclude_track_ids: Option<Vec<String>>,
    limit:             Option<i64>,
) -> Result<Vec<TrackItem>, AppError> {
    let limit = limit.unwrap_or(30).clamp(1, 100) as usize;
    let seeds: Vec<String> = seed_artist_ids.unwrap_or_default();
    let excludes: HashSet<String> =
        exclude_track_ids.unwrap_or_default().into_iter().collect();
    let is_default_home = seeds.is_empty() && excludes.is_empty();

    let token = tok(&app).await?;
    let pool  = app.state::<AppState>().db.clone();

    crate::recommend::recommend(&token, &pool, seeds, excludes, limit, is_default_home).await
}

// cache first playlist load.
//   - offline -> just serve the cached copy (or error if we never saw it)
//   - cached + snapshot_id unchanged -> serve cache, skip pulling every page
//     (this is what stops the full re-fetch on every launch / open)
//   - changed or never cached -> pull all the pages, save em, return fresh
#[tauri::command]
pub async fn get_playlist(app: AppHandle, id: String) -> Result<PlaylistDetail, AppError> {
    let token = tok(&app).await?;
    let pool  = app.state::<AppState>().db.clone();

    let cached_snapshot: Option<String> = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT snapshot_id FROM playlists WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await?
    .and_then(|r| r.0);

    let cached_count: i64 = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await?
    .0;

    // metadata + first track page. on any network error just fall back to cache
    let pl: SpPlaylist = match spotify::spotify_get(&token, &format!("{BASE}/playlists/{id}")).await {
        Ok(pl) => pl,
        Err(e) => {
            return match crate::commands::library::load_cached_playlist(&pool, &id).await? {
                Some(detail) => Ok(detail),
                None         => Err(e),
            };
        }
    };

    // snapshot unchanged and already cached so serve cache, no extra requests
    if cached_count > 0 && pl.snapshot_id.is_some() && pl.snapshot_id == cached_snapshot {
        if let Some(mut detail) = crate::commands::library::load_cached_playlist(&pool, &id).await? {
            // tracks from cache, but the sharing/ownership bits are live
            detail.owner_id      = pl.owner.as_ref().and_then(|o| o.id.clone());
            detail.public        = pl.public;
            detail.collaborative = pl.collaborative;
            detail.followers     = pl.followers.as_ref().and_then(|f| f.total);
            return Ok(detail);
        }
    }

    let total = pl.tracks.as_ref().map(|p| p.total).unwrap_or(0);

    // grab every track (spotify caps each page at 100). save em as we go
    let mut items: Vec<(crate::spotify::types::SpTrack, i64)> = Vec::new();
    let mut next: Option<String> = None;

    if let Some(page) = &pl.tracks {
        for i in &page.items {
            if let Some(t) = i.track.as_ref() {
                let added = i.added_at.as_deref().map(crate::library::parse_iso8601).unwrap_or(0);
                items.push((clone_track(t), added));
            }
        }
        next = page.next.clone();
    }

    while let Some(url) = next.take() {
        let page: SpPlaylistTrackPage = spotify::spotify_get(&token, &url).await?;
        for i in &page.items {
            if let Some(t) = i.track.as_ref() {
                let added = i.added_at.as_deref().map(crate::library::parse_iso8601).unwrap_or(0);
                items.push((clone_track(t), added));
            }
        }
        next = page.next;
    }

    let tracks: Vec<TrackItem> = items.iter().map(|(t, _)| item_from_track(t)).collect();
    let detail = PlaylistDetail {
        id:           pl.id.clone(),
        name:         pl.name.clone(),
        description:  pl.description.clone().filter(|s| !s.is_empty()),
        image_url:    pl.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        owner_name:   pl.owner.as_ref().and_then(|o| o.display_name.clone()),
        total_tracks: total,
        tracks,
        owner_id:      pl.owner.as_ref().and_then(|o| o.id.clone()),
        public:        pl.public,
        collaborative: pl.collaborative,
        followers:     pl.followers.as_ref().and_then(|f| f.total),
    };

    // persist to SQLite asynchronously in the background so the UI doesn't block on 300+ disk writes
    let pool_clone = pool.clone();
    let pl_id = pl.id.clone();
    let pl_name = pl.name.clone();
    let pl_desc = pl.description.clone();
    let pl_owner = pl.owner.as_ref().and_then(|o| o.display_name.clone());
    let pl_img = pl.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone());
    let pl_snap = pl.snapshot_id.clone();

    tokio::spawn(async move {
        let _ = sqlx::query(
            "INSERT INTO playlists
                 (id, name, description, owner_id, image_url, total_tracks,
                  is_public, is_local, snapshot_id, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, 0, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 name         = excluded.name,
                 description  = excluded.description,
                 owner_id     = excluded.owner_id,
                 image_url    = excluded.image_url,
                 total_tracks = excluded.total_tracks,
                 snapshot_id  = excluded.snapshot_id,
                 updated_at   = excluded.updated_at",
        )
        .bind(&pl_id)
        .bind(&pl_name)
        .bind(&pl_desc)
        .bind(&pl_owner)
        .bind(pl_img.as_deref())
        .bind(total)
        .bind(&pl_snap)
        .bind(now_ms())
        .execute(&pool_clone)
        .await;

        let _ = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(&pl_id)
            .execute(&pool_clone)
            .await;

        for (pos, (t, added_at)) in items.iter().enumerate() {
            let _ = crate::library::upsert_track_with_deps(&pool_clone, t).await;
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO playlist_tracks
                     (playlist_id, track_id, position, added_at, added_by)
                 VALUES (?, ?, ?, ?, NULL)",
            )
            .bind(&pl_id)
            .bind(&t.id)
            .bind(pos as i64)
            .bind(added_at)
            .execute(&pool_clone)
            .await;
        }
    });

    Ok(detail)
}

// deep copy a deserialized track (SpTrack isnt Clone) so we can keep it past
// the borrow of the page it came from
fn clone_track(t: &SpTrack) -> SpTrack {
    SpTrack {
        id:           t.id.clone(),
        name:         t.name.clone(),
        duration_ms:  t.duration_ms,
        explicit:     t.explicit,
        popularity:   t.popularity,
        preview_url:  t.preview_url.clone(),
        artists:      t.artists.iter().map(|a| SpArtistSimple { id: a.id.clone(), name: a.name.clone() }).collect(),
        external_ids: t.external_ids.clone(),
        album:        t.album.as_ref().map(|al| SpAlbumSimple {
            id:           al.id.clone(),
            name:         al.name.clone(),
            album_type:   al.album_type.clone(),
            images:       al.images.as_ref().map(|v| v.iter().map(|i| SpImage { url: i.url.clone() }).collect()),
            release_date: al.release_date.clone(),
            artists:      al.artists.as_ref().map(|v| v.iter().map(|a| SpArtistSimple { id: a.id.clone(), name: a.name.clone() }).collect()),
        }),
        track_number: t.track_number,
        disc_number:  t.disc_number,
        is_local:     t.is_local,
    }
}
