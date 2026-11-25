use crate::db::repos::{albums, artists, tracks};
use crate::errors::AppError;
use crate::spotify::{self, types::*};
use crate::state::AppState;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::SqliteConnection;
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

async fn upsert_artist_full(conn: &mut SqliteConnection, a: &SpArtist) -> Result<(), AppError> {
    artists::upsert(&mut *conn, &artists::Artist {
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

async fn upsert_album_simple(conn: &mut SqliteConnection, al: &SpAlbumSimple) -> Result<(), AppError> {
    albums::upsert(&mut *conn, &albums::Album {
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

async fn upsert_album_full(conn: &mut SqliteConnection, al: &SpAlbum) -> Result<(), AppError> {
    albums::upsert(&mut *conn, &albums::Album {
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
        .execute(&mut *conn)
        .await;
        let _ = albums::add_artist(&mut *conn, &al.id, &artist.id, i as i64).await;
    }

    if let Some(track_page) = &al.tracks {
        for t in &track_page.items {
            let _ = tracks::upsert(&mut *conn, &tracks::Track {
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

// ipc commands

// searches already answered
//
// Typing a query, deleting a letter and typing it again, or going back to the
// results from an artist page, asked Spotify the same question again each time
// and waited on the network for an answer that was already known. Answers are
// kept for a few minutes, so those are instant and cost no request.
//
// Only the newest search is worth writing down: a query that has already been
// replaced by a longer one used to cache its sixty rows and add itself to the
// search history anyway, queuing sqlite work in front of the search the
// listener is actually waiting for.

/// How long an answer is good for. New releases appear on a scale of hours.
const SEARCH_KEPT_FOR: std::time::Duration = std::time::Duration::from_secs(300);
/// How many answers are kept.
const SEARCHES_KEPT: usize = 64;

#[derive(Default)]
struct Searches {
    answers: std::collections::HashMap<String, (std::time::Instant, SearchResults)>,
    /// Keys, oldest first.
    order:   std::collections::VecDeque<String>,
}

impl Searches {
    fn get(&mut self, key: &str, now: std::time::Instant) -> Option<SearchResults> {
        match self.answers.get(key) {
            Some((at, results)) if now.duration_since(*at) < SEARCH_KEPT_FOR => Some(results.clone()),
            Some(_) => {
                self.answers.remove(key);
                self.order.retain(|k| k != key);
                None
            }
            None => None,
        }
    }

    fn put(&mut self, key: String, results: SearchResults, now: std::time::Instant) {
        if self.answers.insert(key.clone(), (now, results)).is_none() {
            self.order.push_back(key);
        }
        while self.order.len() > SEARCHES_KEPT {
            if let Some(oldest) = self.order.pop_front() {
                self.answers.remove(&oldest);
            }
        }
    }
}

fn searches() -> &'static std::sync::Mutex<Searches> {
    static SEARCHES: std::sync::OnceLock<std::sync::Mutex<Searches>> = std::sync::OnceLock::new();
    SEARCHES.get_or_init(Default::default)
}

/// Counts searches, so one can tell whether a newer one has started since.
static SEARCH_SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// What a search is kept under: the account it was made by (results are
/// market- and account-specific), what was asked for, and of which kinds.
fn search_key(epoch: u64, query: &str, types: &str) -> String {
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    format!("{epoch}
{types}
{query}")
}

/// The catalogue (songs, artists, albums) and the rest (playlists, shows,
/// episodes), asked for side by side and put back together. Spotify answers
/// the two halves sooner than all six types in one request: a median of about
/// 600 ms against 700-760 (`search_probe`).
fn split_types(types: &str) -> Option<(String, String)> {
    let (catalogue, rest): (Vec<&str>, Vec<&str>) = types
        .split(',')
        .map(str::trim)
        .filter(|kind| !kind.is_empty())
        .partition(|kind| matches!(*kind, "track" | "artist" | "album"));
    (!catalogue.is_empty() && !rest.is_empty()).then(|| (catalogue.join(","), rest.join(",")))
}

async fn search_once(token: &str, query: &str, types: &str) -> Result<SpSearchResponse, AppError> {
    let mut url = url::Url::parse(&format!("{BASE}/search")).unwrap();
    url.query_pairs_mut()
        .append_pair("q",     query)
        .append_pair("type",  types)
        .append_pair("limit", "20");
    spotify::spotify_get(token, url.as_str()).await
}

async fn search_split(token: &str, query: &str, types: &str) -> Result<SpSearchResponse, AppError> {
    let Some((catalogue, rest)) = split_types(types) else {
        return search_once(token, query, types).await;
    };
    let (first, second) = tokio::join!(search_once(token, query, &catalogue), search_once(token, query, &rest));
    let (first, second) = match (first, second) {
        (Ok(first), Ok(second)) => (first, second),
        // the songs are what a search is for; playlists failing leaves them
        (Ok(first), Err(e)) => {
            eprintln!("[search] playlists, shows and episodes failed: {e}");
            return Ok(first);
        }
        (Err(e), _) => return Err(e),
    };
    Ok(SpSearchResponse {
        tracks:    first.tracks.or(second.tracks),
        artists:   first.artists.or(second.artists),
        albums:    first.albums.or(second.albums),
        playlists: second.playlists.or(first.playlists),
        shows:     second.shows.or(first.shows),
        episodes:  second.episodes.or(first.episodes),
    })
}

#[tauri::command]
pub async fn search(
    app:   AppHandle,
    query: String,
    types: Option<String>,
) -> Result<SearchResults, AppError> {
    let serial = SEARCH_SERIAL.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    let t     = types.as_deref().unwrap_or("track,artist,album,playlist");
    let key   = search_key(crate::auth::auth_epoch(), &query, t);
    let known = searches()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key, std::time::Instant::now());
    if let Some(results) = known {
        return Ok(results);
    }
    let token = tok(&app).await?;

    let raw = search_split(&token, &query, t).await?;

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
        shows: raw.shows.as_ref()
            .and_then(|p| p.get("items"))
            .and_then(|i| i.as_array())
            .map(|a| a.iter().filter_map(crate::commands::podcasts::show_from).collect())
            .unwrap_or_default(),
        episodes: raw.episodes.as_ref()
            .and_then(|p| p.get("items"))
            .and_then(|i| i.as_array())
            .map(|a| a.iter().filter(|e| !e.is_null())
                .filter_map(|e| crate::commands::podcasts::episode_from(e, None)).collect())
            .unwrap_or_default(),
    };

    searches()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .put(key, result.clone(), std::time::Instant::now());

    // a newer search has started: this one's rows and its place in the search
    // history are not worth holding that one's writes up for
    if SEARCH_SERIAL.load(std::sync::atomic::Ordering::SeqCst) != serial {
        return Ok(result);
    }

    // cache the hits after answering. awaiting ~60 row writes here put every
    // keystroke's results behind sqlite - and behind a running sync's writes
    persist_later(&app, move |conn| Box::pin(async move {
        for a  in raw.artists.as_ref().map(|p| p.items.as_slice()).unwrap_or(&[]) { let _ = upsert_artist_full(conn, a).await; }
        for al in raw.albums.as_ref().map(|p|  p.items.as_slice()).unwrap_or(&[]) { let _ = upsert_album_simple(conn, al).await; }
        let mut deps = crate::library::DepWriter::default();
        for tr in raw.tracks.as_ref().map(|p|  p.items.as_slice()).unwrap_or(&[]) { let _ = deps.track(conn, tr).await; }
        let _ = crate::db::repos::search_history::push(&mut *conn, &query, None, None).await;
    }));

    Ok(result)
}

/// `MUSIQUE_ENGINE_PROBE=1 MUSIQUE_SEARCH_PROBE=1 spotify.exe` times the
/// search the interface makes, asked for in one request and split in two run
/// side by side, against the live Web API. Read-only. (Measured 2026-11-14:
/// medians of 700-759 ms in one, 600-611 ms in two. Three requests, or one
/// per type, gained no more and had a slower tail.)
pub async fn search_probe(app: AppHandle) -> i32 {
    use std::time::Instant;
    let token = match tok(&app).await {
        Ok(token) => token,
        Err(e) => { eprintln!("[search-probe] not signed in: {e}"); return 2; }
    };
    let ask = |types: &'static str, query: String| {
        let token = token.clone();
        async move {
            let mut url = url::Url::parse(&format!("{BASE}/search")).unwrap();
            url.query_pairs_mut().append_pair("q", &query).append_pair("type", types).append_pair("limit", "20");
            spotify::spotify_get::<serde_json::Value>(&token, url.as_str()).await.map(drop)
        }
    };
    let queries = [
        "daft punk", "radiohead creep", "lofi beats", "taylor swift", "moonlight sonata", "the weeknd",
        "jazz piano", "kendrick lamar", "fleetwood mac", "ambient", "bad bunny", "chopin nocturne",
    ];
    let (mut whole, mut split) = (Vec::new(), Vec::new());
    for round in 0..2 {
        for (n, query) in queries.iter().enumerate() {
            // a different query each round, so neither form meets a warm cache
            let query = format!("{query}{}", if round == 0 { "" } else { " live" });
            let one = || async {
                let at = Instant::now();
                ask("track,artist,album,playlist,show,episode", query.clone()).await.map(|_| at.elapsed())
            };
            let two = || async {
                let at = Instant::now();
                let (a, b) = tokio::join!(
                    ask("track,artist,album", query.clone()),
                    ask("playlist,show,episode", query.clone()),
                );
                a.and(b).map(|_| at.elapsed())
            };
            let (a, b) = if n % 2 == 0 { (one().await, two().await) } else { let b = two().await; (one().await, b) };
            match (a, b) {
                (Ok(a), Ok(b)) => { whole.push(a.as_millis()); split.push(b.as_millis()); }
                (a, b) => { eprintln!("[search-probe] {query:?} failed: {:?} {:?}", a.err(), b.err()); return 3; }
            }
        }
    }
    let median = |v: &mut Vec<u128>| { v.sort_unstable(); v[v.len() / 2] };
    eprintln!("[search-probe] one request: median {} ms {:?}", median(&mut whole.clone()), whole);
    eprintln!("[search-probe] split in two: median {} ms {:?}", median(&mut split.clone()), split);
    // the merged answer the interface gets has every kind in it
    match search_split(&token, "daft punk", "track,artist,album,playlist,show,episode").await {
        Ok(raw) => {
            let counts = [
                raw.tracks.as_ref().map_or(0, |p| p.items.len()),
                raw.artists.as_ref().map_or(0, |p| p.items.len()),
                raw.albums.as_ref().map_or(0, |p| p.items.len()),
                raw.playlists.as_ref().map_or(0, |p| p.items.len()),
                raw.shows.as_ref().and_then(|p| p["items"].as_array().map(Vec::len)).unwrap_or(0),
                raw.episodes.as_ref().and_then(|p| p["items"].as_array().map(Vec::len)).unwrap_or(0),
            ];
            eprintln!("[search-probe] merged answer: tracks, artists, albums, playlists, shows, episodes = {counts:?}");
            if counts[0] == 0 || counts[3] == 0 {
                return 3;
            }
        }
        Err(e) => { eprintln!("[search-probe] merged search failed: {e}"); return 3; }
    }
    0
}

/// a stored page older than this is refetched behind the copy it answers with.
/// the page's own react-query cache covers repeat opens within a session, so
/// this only stops one open from fetching the same page twice
const PAGE_REVALIDATE_MS: i64 = 60_000;

/// the artist page. a stored copy answers at once (a first open after a cold
/// start no longer waits on three web api calls) and is refetched behind it
#[tauri::command]
pub async fn get_artist(app: AppHandle, id: String) -> Result<ArtistDetail, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:artist:{id}");
    let key = serde_json::json!(["artist", id]);
    let handle = app.clone();
    crate::internal::cache::swr_json(&app, &pool, &uri, "artist-page", PAGE_REVALIDATE_MS, key, move || {
        fetch_artist(handle, id)
    })
    .await
}

async fn fetch_artist(app: AppHandle, id: String) -> Result<ArtistDetail, AppError> {
    let token = tok(&app).await?;

    let mut alb_url = url::Url::parse(&format!("{BASE}/artists/{id}/albums")).unwrap();
    alb_url.query_pairs_mut()
        .append_pair("limit",         "50")
        .append_pair("include_groups", "album,single,compilation");
    let artist_url = format!("{BASE}/artists/{id}");
    let top_url    = format!("{BASE}/artists/{id}/top-tracks?market=from_token");

    // independent requests, so they go out together instead of back to back
    let (artist, album_page, top) = tokio::join!(
        spotify::spotify_get::<SpArtist>(&token, &artist_url),
        spotify::spotify_get::<SpPage<SpAlbumSimple>>(&token, alb_url.as_str()),
        spotify::spotify_get::<SpTopTracks>(&token, &top_url),
    );
    let artist     = artist?;
    let album_page = album_page?;
    // top tracks are best effort, an artist with none still renders the page fine
    let top_tracks: Vec<TrackItem> = top
        .map(|t| t.tracks.iter().map(item_from_track).collect())
        .unwrap_or_default();

    // split up the discography, full albums (incl compilations) vs singles/eps
    let mut albums:  Vec<AlbumItem> = Vec::new();
    let mut singles: Vec<AlbumItem> = Vec::new();
    for al in &album_page.items {
        let item = item_from_album_simple(al);
        if al.album_type == "single" { singles.push(item); } else { albums.push(item); }
    }

    let detail = ArtistDetail {
        id:         artist.id.clone(),
        name:       artist.name.clone(),
        image_url:  artist.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        genres:     artist.genres.clone().unwrap_or_default(),
        popularity: artist.popularity,
        albums,
        singles,
        top_tracks,
    };

    persist_later(&app, move |conn| Box::pin(async move {
        let _ = upsert_artist_full(conn, &artist).await;
        for al in &album_page.items { let _ = upsert_album_simple(conn, al).await; }
    }));

    Ok(detail)
}

/// the album page, answered from its stored copy like `get_artist`
#[tauri::command]
pub async fn get_album(app: AppHandle, id: String) -> Result<AlbumDetail, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:album:{id}");
    let key = serde_json::json!(["album", id]);
    let handle = app.clone();
    crate::internal::cache::swr_json(&app, &pool, &uri, "album-page", PAGE_REVALIDATE_MS, key, move || {
        fetch_album(handle, id)
    })
    .await
}

async fn fetch_album(app: AppHandle, id: String) -> Result<AlbumDetail, AppError> {
    let token = tok(&app).await?;

    let mut url = url::Url::parse(&format!("{BASE}/albums/{id}")).unwrap();
    url.query_pairs_mut().append_pair("market", "from_token");
    let album: SpAlbum = spotify::spotify_get(&token, url.as_str()).await?;

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

    let detail = AlbumDetail {
        id:           album.id.clone(),
        name:         album.name.clone(),
        album_type:   album.album_type.clone(),
        image_url:    album.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone()),
        release_date: album.release_date.clone(),
        total_tracks: album.total_tracks.unwrap_or(0),
        popularity:   album.popularity,
        artists:      album_item.artists.clone(),
        tracks,
    };

    persist_later(&app, move |conn| Box::pin(async move {
        let _ = upsert_album_full(conn, &album).await;
    }));

    Ok(detail)
}

#[tauri::command]
pub async fn get_track(app: AppHandle, id: String) -> Result<TrackDetail, AppError> {
    let token = tok(&app).await?;
    let track: SpTrack = spotify::spotify_get(&token, &format!("{BASE}/tracks/{id}")).await?;

    // awaited, not deferred: the lyrics lookup reads the isrc this stores
    let pool = app.state::<AppState>().db.clone();
    if let Ok(mut tx) = pool.begin().await {
        if crate::library::upsert_track_with_deps(&mut tx, &track).await.is_ok() {
            let _ = tx.commit().await;
        }
    }

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
            // spotify-made playlists (daily mix, discover weekly, editorial)
            // 404 on the public api for most apps; spclient still serves them.
            // a real network outage falls through to whatever we cached
            if let Ok(detail) = crate::commands::internal::internal_playlist(&app, &id).await {
                if !detail.tracks.is_empty() {
                    remember_internal_playlist(&pool, &detail);
                    return Ok(detail);
                }
            }
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

    // apps on spotify's 2026 api schema only get the contents of playlists the
    // user owns or collaborates on; anything else comes back with no tracks.
    // spclient still serves those
    if pl.track_page().map_or(true, |p| p.items.is_empty() && p.total > 0) {
        if let Ok(mut detail) = crate::commands::internal::internal_playlist(&app, &id).await {
            if !detail.tracks.is_empty() {
                if detail.image_url.is_none() {
                    detail.image_url = pl.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone());
                }
                detail.owner_id      = pl.owner.as_ref().and_then(|o| o.id.clone());
                detail.public        = pl.public;
                detail.collaborative = pl.collaborative;
                detail.followers     = pl.followers.as_ref().and_then(|f| f.total);
                remember_internal_playlist(&pool, &detail);
                return Ok(detail);
            }
        }
    }

    let total = pl.track_page().map(|p| p.total).unwrap_or(0);

    // grab every track (spotify caps each page at 100). save em as we go
    let mut items: Vec<(crate::spotify::types::SpTrack, i64)> = Vec::new();
    let mut next: Option<String> = None;

    if let Some(page) = pl.track_page() {
        for i in &page.items {
            if let Some(t) = i.track() {
                let added = i.added_at.as_deref().map(crate::library::parse_iso8601).unwrap_or(0);
                items.push((clone_track(t), added));
            }
        }
        next = page.next.clone();
    }

    // the rest of the pages all at once. `next` is the template (it carries
    // whatever params spotify wants); only its offset changes per page
    if let Some(next) = next.as_deref().and_then(|u| url::Url::parse(u).ok()) {
        let param = |k: &str| next.query_pairs().find(|(n, _)| n == k).and_then(|(_, v)| v.parse::<i64>().ok());
        let limit = param("limit").unwrap_or(100).max(1);
        let first = param("offset").unwrap_or(limit);
        let token = token.as_str();
        let pages: Vec<SpPlaylistTrackPage> = futures_util::stream::iter((first..total).step_by(limit as usize))
            .map(|offset| {
                let url = with_offset(&next, offset);
                async move { spotify::spotify_get::<SpPlaylistTrackPage>(token, &url).await }
            })
            .buffered(8)
            .try_collect()
            .await?;
        for page in &pages {
            for i in &page.items {
                if let Some(t) = i.track() {
                    let added = i.added_at.as_deref().map(crate::library::parse_iso8601).unwrap_or(0);
                    items.push((clone_track(t), added));
                }
            }
        }
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

    // persist in the background (one transaction) so the UI doesn't wait on 300+ row writes
    let pl_id = pl.id.clone();
    let pl_name = pl.name.clone();
    let pl_desc = pl.description.clone();
    let pl_owner = pl.owner.as_ref().and_then(|o| o.display_name.clone());
    let pl_img = pl.images.as_ref().and_then(|v| v.first()).map(|i| i.url.clone());
    let pl_snap = pl.snapshot_id.clone();

    persist_later(&app, move |conn| Box::pin(async move {
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
        .execute(&mut *conn)
        .await;

        let _ = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(&pl_id)
            .execute(&mut *conn)
            .await;

        let mut deps = crate::library::DepWriter::default();
        for (pos, (t, added_at)) in items.iter().enumerate() {
            let _ = deps.track(conn, t).await;
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO playlist_tracks
                     (playlist_id, track_id, position, added_at, added_by)
                 VALUES (?, ?, ?, ?, NULL)",
            )
            .bind(&pl_id)
            .bind(&t.id)
            .bind(pos as i64)
            .bind(added_at)
            .execute(&mut *conn)
            .await;
        }
    }));

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

fn with_offset(base: &url::Url, offset: i64) -> String {
    let pairs: Vec<(String, String)> = base.query_pairs()
        .filter(|(k, _)| k != "offset")
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let mut url = base.clone();
    url.query_pairs_mut().clear().extend_pairs(pairs).append_pair("offset", &offset.to_string());
    url.into()
}

/// what a playlist only spclient serves (spotify-made mixes, radios, other
/// people's lists) is kept as one page, since those lists never land in the
/// library tables. `get_cached_playlist` paints the next open from it, a cold
/// start included, while the live fetch revalidates behind it
pub(crate) const INTERNAL_PLAYLIST_KIND: &str = "playlist-page";

fn remember_internal_playlist(pool: &sqlx::SqlitePool, detail: &PlaylistDetail) {
    let (pool, detail) = (pool.clone(), detail.clone());
    let epoch = crate::auth::auth_epoch();
    tokio::spawn(async move {
        if crate::auth::epoch_is_current(epoch) {
            let uri = format!("spotify:playlist:{}", detail.id);
            crate::internal::cache::put_json(&pool, &uri, INTERNAL_PLAYLIST_KIND, &detail).await;
        }
    });
}

/// Cache what a command fetched without making its caller wait. The writes go
/// in one transaction (one commit instead of one per row); a failure only
/// costs a cache entry, so it is dropped.
fn persist_later<F>(app: &AppHandle, write: F)
where
    F: for<'c> FnOnce(&'c mut SqliteConnection) -> futures_util::future::BoxFuture<'c, ()> + Send + 'static,
{
    let pool = app.state::<AppState>().db.clone();
    tokio::spawn(async move {
        let Ok(mut tx) = pool.begin().await else { return };
        write(&mut tx).await;
        let _ = tx.commit().await;
    });
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_search_splits_the_catalogue_from_the_rest_only_when_it_asks_for_both() {
        assert_eq!(
            split_types("track,artist,album,playlist,show,episode"),
            Some(("track,artist,album".into(), "playlist,show,episode".into()))
        );
        assert_eq!(split_types("track, playlist"), Some(("track".into(), "playlist".into())));
        assert_eq!(split_types("track,artist,album"), None);
        assert_eq!(split_types("playlist"), None);
        assert_eq!(split_types(""), None);
    }

    fn results(name: &str) -> SearchResults {
        SearchResults {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            playlists: vec![PlaylistCard {
                id: name.into(),
                name: name.into(),
                description: None,
                image_url: None,
                owner_name: None,
            }],
            shows: Vec::new(),
            episodes: Vec::new(),
        }
    }

    #[test]
    fn the_same_question_is_the_same_key_however_it_was_typed() {
        assert_eq!(search_key(1, "  Daft   Punk ", "track"), search_key(1, "daft punk", "track"));
        assert_ne!(search_key(1, "daft punk", "track"), search_key(1, "daft punk", "album"));
        // another account's answers are not this one's
        assert_ne!(search_key(1, "daft punk", "track"), search_key(2, "daft punk", "track"));
    }

    #[test]
    fn answers_are_kept_for_a_while_and_only_so_many() {
        let mut searches = Searches::default();
        let start = Instant::now();
        searches.put("a".into(), results("a"), start);
        assert_eq!(searches.get("a", start + Duration::from_secs(60)).unwrap().playlists[0].id, "a");
        assert!(searches.get("a", start + SEARCH_KEPT_FOR).is_none());
        assert!(searches.order.is_empty());

        for i in 0..SEARCHES_KEPT + 5 {
            searches.put(format!("q{i}"), results("x"), start);
        }
        assert_eq!(searches.answers.len(), SEARCHES_KEPT);
        assert!(searches.get("q0", start).is_none());
        assert!(searches.get(&format!("q{}", SEARCHES_KEPT + 4), start).is_some());
    }
}
