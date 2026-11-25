//! High-level Apple Music API operations mapped to musique's models.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::stream::{self, StreamExt as _};
use rand::RngCore as _;
use serde_json::{json, Value};

use crate::apple::client::{self, encode_segment, AppleAccount, AppleClient};
use crate::apple::wire::{self, ParsedAlbum, ParsedArtist, ParsedPlaylist, ParsedTrack};
use crate::commands::artist_extras::{ArtistExtras, ArtistOverview};
use crate::commands::home_feed::{HomeFeed, HomeItem};
use crate::commands::internal::RadioResult;
use crate::commands::library::PlaylistSummary;
use crate::errors::AppError;
use crate::spotify::types::{
    AlbumDetail, AlbumItem, ArtistDetail, ArtistItem, PlaylistDetail, SearchResults, TrackDetail,
    TrackItem,
};

const LIBRARY_PAGE_SIZE: usize = 100;
const LIBRARY_MAX_PAGES: usize = 40;
const SEARCH_LIMIT: usize = 25;
const ARTIST_BATCH_SIZE: usize = 50;
const SONG_BATCH_SIZE: usize = 100;
const STATION_PULL_LIMIT: usize = 10;
const STATION_PULLS: usize = 3;
const CLOCK_SKEW_SECS: i64 = 120;

const BEACON_BUILD: &str = "AppleMusic/1.0 Linux/0.0 model/Linux x86_64 build/2638.11.0-external";
const BEACON_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:156.0) Gecko/20100101 Firefox/156.0";

const LIBRARY_SONGS_QUERY: &[(&str, &str)] = &[
    ("include[library-songs]", "catalog,albums,artists"),
    ("include[songs]", "artists,albums"),
    ("fields[library-albums]", "dateAdded"),
    ("sort", "-dateAdded"),
];

const LIBRARY_ALBUMS_QUERY: &[(&str, &str)] = &[
    ("include", "catalog,artists"),
    ("include[albums]", "artists"),
];

const ARTIST_VIEWS: &str =
    "top-songs,full-albums,singles,compilation-albums,live-albums,appears-on-albums,similar-artists";

// ─── public API (delegates through client::current()) ────────────────────────

pub async fn search(query: &str, types: &str) -> Result<SearchResults, AppError> {
    let c = client::current().await?;
    search_with(&c, query, types).await
}

pub async fn album(id: &str) -> Result<AlbumDetail, AppError> {
    let c = client::current().await?;
    album_with(&c, id).await
}

pub async fn artist(id: &str) -> Result<ArtistDetail, AppError> {
    let c = client::current().await?;
    artist_with(&c, id).await
}

pub async fn artist_extras(id: &str) -> Result<ArtistExtras, AppError> {
    let c = client::current().await?;
    artist_extras_with(&c, id).await
}

pub async fn artist_overview(id: &str) -> Result<ArtistOverview, AppError> {
    let c = client::current().await?;
    artist_overview_with(&c, id).await
}

pub async fn track(id: &str) -> Result<TrackDetail, AppError> {
    let c = client::current().await?;
    track_with(&c, id).await
}

pub async fn track_isrc(id: &str) -> Result<Option<String>, AppError> {
    let c = client::current().await?;
    track_isrc_with(&c, id).await
}

pub async fn tracks(ids: &[String]) -> Result<Vec<TrackItem>, AppError> {
    let c = client::current().await?;
    tracks_with(&c, ids).await
}

pub async fn playlist(id: &str) -> Result<PlaylistDetail, AppError> {
    let c = client::current().await?;
    playlist_with(&c, id).await
}

pub async fn liked_songs() -> Result<Vec<TrackItem>, AppError> {
    let c = client::current().await?;
    Ok(liked_songs_parsed_with(&c)
        .await?
        .into_iter()
        .map(|t| t.item)
        .collect())
}

pub async fn library_playlists() -> Result<Vec<PlaylistSummary>, AppError> {
    let c = client::current().await?;
    Ok(library_playlists_parsed_with(&c)
        .await?
        .into_iter()
        .map(ParsedPlaylist::into_summary)
        .collect())
}

pub async fn library_albums() -> Result<Vec<AlbumItem>, AppError> {
    let c = client::current().await?;
    Ok(library_albums_parsed_with(&c)
        .await?
        .into_iter()
        .map(|a| a.item)
        .collect())
}

pub async fn library_artists() -> Result<Vec<ArtistItem>, AppError> {
    let c = client::current().await?;
    Ok(library_artists_parsed_with(&c)
        .await?
        .into_iter()
        .map(|a| a.item)
        .collect())
}

pub async fn recently_played() -> Result<Vec<TrackItem>, AppError> {
    let c = client::current().await?;
    recently_played_with(&c).await
}

pub async fn heavy_rotation_tracks() -> Result<Vec<TrackItem>, AppError> {
    let c = client::current().await?;
    heavy_rotation_tracks_with(&c).await
}

pub async fn heavy_rotation_artists() -> Result<Vec<ArtistItem>, AppError> {
    let c = client::current().await?;
    heavy_rotation_artists_with(&c).await
}

pub async fn new_releases() -> Result<Vec<AlbumItem>, AppError> {
    let c = client::current().await?;
    new_releases_with(&c).await
}

pub async fn home_feed() -> Result<HomeFeed, AppError> {
    let c = client::current().await?;
    home_feed_with(&c).await
}

pub async fn track_radio(track_id: &str) -> Result<RadioResult, AppError> {
    let c = client::current().await?;
    track_radio_with(&c, track_id).await
}

pub async fn save_track(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    save_track_with(&c, id).await
}

pub async fn unsave_track(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    unsave_track_with(&c, id).await
}

pub async fn saved_track_ids(ids: &[String]) -> Result<Vec<String>, AppError> {
    let c = client::current().await?;
    saved_track_ids_with(&c, ids).await
}

pub async fn save_album(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    save_album_with(&c, id).await
}

pub async fn unsave_album(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    unsave_album_with(&c, id).await
}

pub async fn is_album_saved(id: &str) -> Result<bool, AppError> {
    let c = client::current().await?;
    is_album_saved_with(&c, id).await
}

pub async fn follow_artist(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    follow_artist_with(&c, id).await
}

pub async fn unfollow_artist(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    unfollow_artist_with(&c, id).await
}

pub async fn is_artist_followed(id: &str) -> Result<bool, AppError> {
    let c = client::current().await?;
    is_artist_followed_with(&c, id).await
}

pub async fn create_playlist(
    name: &str,
    description: Option<&str>,
    public: bool,
) -> Result<String, AppError> {
    let c = client::current().await?;
    create_playlist_with(&c, name, description, public).await
}

pub async fn update_playlist(
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    public: Option<bool>,
) -> Result<(), AppError> {
    let c = client::current().await?;
    update_playlist_with(&c, id, name, description, public).await
}

pub async fn delete_playlist(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    delete_playlist_with(&c, id).await
}

pub async fn follow_playlist(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    follow_playlist_with(&c, id).await
}

pub async fn unfollow_playlist(id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    unfollow_playlist_with(&c, id).await
}

pub async fn is_playlist_followed(id: &str) -> Result<bool, AppError> {
    let c = client::current().await?;
    is_playlist_followed_with(&c, id).await
}

pub async fn add_track_to_playlist(pl: &str, track: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    add_track_to_playlist_with(&c, pl, track).await
}

pub async fn remove_track_from_playlist(pl: &str, track: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    remove_track_from_playlist_with(&c, pl, track).await
}

pub async fn report_play(track_id: &str) -> Result<(), AppError> {
    let c = client::current().await?;
    report_play_with(&c, track_id).await
}

pub async fn profile() -> Result<AppleAccount, AppError> {
    let c = client::current().await?;
    c.account_profile().await
}

// ─── client-scoped implementations (used by api, library sync, and tests) ─────

pub(crate) async fn search_with(
    client: &AppleClient,
    query: &str,
    types: &str,
) -> Result<SearchResults, AppError> {
    let query = query.trim();
    let apple_types = map_search_types(types);
    if query.is_empty() || apple_types.is_empty() {
        return Ok(SearchResults {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            playlists: Vec::new(),
            shows: Vec::new(),
            episodes: Vec::new(),
        });
    }

    let limit = SEARCH_LIMIT.to_string();
    let body = client
        .get(
            &client.catalog_path("/search"),
            &[
                ("term", query),
                ("types", &apple_types),
                ("limit", &limit),
                ("include[songs]", "artists,albums"),
                ("include[albums]", "artists"),
            ],
        )
        .await?;
    Ok(wire::parse_search_results(&body))
}

fn map_search_types(types: &str) -> String {
    let mut mapped: Vec<&'static str> = Vec::new();
    for part in types.split(',').map(|s| s.trim().to_ascii_lowercase()) {
        let apple = match part.as_str() {
            "track" | "tracks" | "song" | "songs" => "songs",
            "album" | "albums" => "albums",
            "artist" | "artists" => "artists",
            "playlist" | "playlists" => "playlists",
            _ => continue,
        };
        if !mapped.contains(&apple) {
            mapped.push(apple);
        }
    }
    mapped.join(",")
}

pub(crate) async fn album_with(client: &AppleClient, id: &str) -> Result<AlbumDetail, AppError> {
    let id = require_id(id, "album")?;
    let body = if wire::is_library_id(id) {
        client
            .get(
                &format!("/me/library/albums/{}", encode_segment(id)),
                &[
                    ("include", "tracks,catalog,artists"),
                    ("include[albums]", "artists"),
                    ("include[library-songs]", "catalog,artists,albums"),
                    ("include[songs]", "artists,albums"),
                ],
            )
            .await?
    } else {
        client
            .get(
                &client.catalog_path(&format!("/albums/{}", encode_segment(id))),
                &[
                    ("include", "tracks,artists"),
                    ("include[songs]", "artists,albums"),
                ],
            )
            .await?
    };

    wire::parse_page(&body)
        .data
        .first()
        .and_then(wire::parse_album_detail)
        .ok_or_else(|| AppError::NotFound(format!("Apple Music album {id} not found")))
}

async fn fetch_artist_resource(client: &AppleClient, id: &str) -> Result<Value, AppError> {
    let id = require_id(id, "artist")?;
    if wire::is_synthetic_artist_id(id) {
        return Err(AppError::NotFound(
            "Library-only artist has no catalog page".into(),
        ));
    }

    if wire::is_library_id(id) {
        let body = client
            .get(
                &format!("/me/library/artists/{}", encode_segment(id)),
                &[
                    ("include", "catalog,albums"),
                    ("include[library-albums]", "catalog,artists,tracks"),
                    ("include[albums]", "artists"),
                ],
            )
            .await?;
        let page = wire::parse_page(&body);
        let row = page
            .data
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound(format!("Apple Music artist {id} not found")))?;
        if let Some(cat_id) = wire::catalog_rel(&row)
            .and_then(|c| wire::str_field(c, "id"))
            .map(str::to_owned)
        {
            if let Ok(cat) = fetch_catalog_artist_resource(client, &cat_id).await {
                return Ok(cat);
            }
        }
        return Ok(row);
    }

    fetch_catalog_artist_resource(client, id).await
}

async fn fetch_catalog_artist_resource(
    client: &AppleClient,
    id: &str,
) -> Result<Value, AppError> {
    let body = client
        .get(
            &client.catalog_path(&format!("/artists/{}", encode_segment(id))),
            &[
                ("views", ARTIST_VIEWS),
                ("include[songs]", "artists,albums"),
                ("include[albums]", "artists"),
                ("extend", "artistBio,editorialArtwork"),
            ],
        )
        .await?;
    wire::parse_page(&body)
        .data
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("Apple Music artist {id} not found")))
}

pub(crate) async fn artist_with(client: &AppleClient, id: &str) -> Result<ArtistDetail, AppError> {
    let row = fetch_artist_resource(client, id).await?;
    wire::parse_artist_detail(&row)
        .ok_or_else(|| AppError::NotFound(format!("Apple Music artist {id} could not be parsed")))
}

pub(crate) async fn artist_extras_with(
    client: &AppleClient,
    id: &str,
) -> Result<ArtistExtras, AppError> {
    if wire::is_synthetic_artist_id(id) {
        return Ok(ArtistExtras::default());
    }
    let row = fetch_artist_resource(client, id).await?;
    Ok(wire::parse_artist_extras(&row))
}

pub(crate) async fn artist_overview_with(
    client: &AppleClient,
    id: &str,
) -> Result<ArtistOverview, AppError> {
    if wire::is_synthetic_artist_id(id) {
        return Ok(ArtistOverview::default());
    }
    let row = fetch_artist_resource(client, id).await?;
    Ok(wire::parse_artist_overview(&row))
}

pub(crate) async fn track_with(client: &AppleClient, id: &str) -> Result<TrackDetail, AppError> {
    let id = require_id(id, "track")?;
    let body = if wire::is_library_id(id) {
        client
            .get(
                &format!("/me/library/songs/{}", encode_segment(id)),
                LIBRARY_SONGS_QUERY,
            )
            .await?
    } else {
        client
            .get(
                &client.catalog_path(&format!("/songs/{}", encode_segment(id))),
                &[("include[songs]", "artists,albums")],
            )
            .await?
    };

    wire::parse_page(&body)
        .data
        .first()
        .and_then(wire::parse_track_detail)
        .ok_or_else(|| AppError::NotFound(format!("Apple Music track {id} not found")))
}

pub(crate) async fn track_isrc_with(
    client: &AppleClient,
    id: &str,
) -> Result<Option<String>, AppError> {
    let id = require_id(id, "track")?;
    let resp = if wire::is_library_id(id) {
        client
            .get(
                &format!("/me/library/songs/{}", encode_segment(id)),
                &[
                    ("include[library-songs]", "catalog"),
                    ("fields[songs]", "isrc"),
                ],
            )
            .await
    } else {
        client
            .get(
                &client.catalog_path(&format!("/songs/{}", encode_segment(id))),
                &[("fields[songs]", "isrc")],
            )
            .await
    };
    let body = match resp {
        Ok(b) => b,
        Err(AppError::NotFound(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    Ok(wire::parse_page(&body)
        .data
        .first()
        .and_then(wire::parse_track_isrc))
}

pub(crate) async fn tracks_with(
    client: &AppleClient,
    ids: &[String],
) -> Result<Vec<TrackItem>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut catalog_ids: Vec<&str> = Vec::new();
    let mut lib_ids: Vec<&str> = Vec::new();
    for raw in ids {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if wire::is_library_id(trimmed) {
            if !lib_ids.contains(&trimmed) {
                lib_ids.push(trimmed);
            }
        } else if !catalog_ids.contains(&trimmed) {
            catalog_ids.push(trimmed);
        }
    }

    let mut by_id: HashMap<String, TrackItem> = HashMap::new();

    for chunk in catalog_ids.chunks(SONG_BATCH_SIZE) {
        let joined = chunk.join(",");
        let body = client
            .get(
                &client.catalog_path("/songs"),
                &[("ids", &joined), ("include[songs]", "artists,albums")],
            )
            .await?;
        for row in &wire::parse_page(&body).data {
            if let Some(parsed) = wire::parse_track(row, None) {
                by_id.insert(parsed.item.id.clone(), parsed.item);
            }
        }
    }

    for chunk in lib_ids.chunks(SONG_BATCH_SIZE) {
        let joined = chunk.join(",");
        if let Ok(body) = client
            .get(
                "/me/library/songs",
                &[
                    ("ids", &joined),
                    ("include[library-songs]", "catalog,albums,artists"),
                    ("include[songs]", "artists,albums"),
                ],
            )
            .await
        {
            for row in &wire::parse_page(&body).data {
                if let Some(parsed) = wire::parse_track(row, None) {
                    if let Some(lid) = parsed.library_id.clone() {
                        by_id.insert(lid, parsed.item.clone());
                    }
                    by_id.insert(parsed.item.id.clone(), parsed.item);
                }
            }
        }
    }

    Ok(ids
        .iter()
        .filter_map(|id| by_id.get(id.trim()).cloned())
        .collect())
}

pub(crate) async fn playlist_with(
    client: &AppleClient,
    id: &str,
) -> Result<PlaylistDetail, AppError> {
    let id = require_id(id, "playlist")?;
    let is_lib = wire::is_library_id(id);
    let base_path = if is_lib {
        format!("/me/library/playlists/{}", encode_segment(id))
    } else {
        client.catalog_path(&format!("/playlists/{}", encode_segment(id)))
    };

    let meta_body = if is_lib {
        client.get(&base_path, &[("include", "catalog")]).await?
    } else {
        client.get(&base_path, &[]).await?
    };

    let parsed = wire::parse_page(&meta_body)
        .data
        .first()
        .and_then(wire::parse_playlist)
        .ok_or_else(|| AppError::NotFound(format!("Apple Music playlist {id} not found")))?;

    let tracks_path = format!("{base_path}/tracks");
    let query: &[(&str, &str)] = if is_lib {
        &[
            ("include[songs]", "artists,albums"),
            ("include[library-songs]", "catalog,albums,artists"),
        ]
    } else {
        &[("include[songs]", "artists,albums")]
    };

    let (parsed_tracks, total_from_pages) =
        fetch_all_playlist_tracks(client, &tracks_path, query).await?;
    let tracks: Vec<TrackItem> = parsed_tracks.into_iter().map(|t| t.item).collect();
    let total_tracks = total_from_pages
        .map(|t| t as i64)
        .unwrap_or(tracks.len() as i64)
        .max(tracks.len() as i64);

    Ok(parsed.into_detail(tracks, total_tracks))
}

async fn fetch_all_playlist_tracks(
    client: &AppleClient,
    tracks_path: &str,
    query: &[(&str, &str)],
) -> Result<(Vec<ParsedTrack>, Option<usize>), AppError> {
    let limit_str = LIBRARY_PAGE_SIZE.to_string();
    let mut first_query = query.to_vec();
    first_query.push(("limit", &limit_str));

    let first_body = match client.get(tracks_path, &first_query).await {
        Ok(b) => b,
        Err(AppError::NotFound(_)) => return Ok((Vec::new(), Some(0))),
        Err(e) => return Err(e),
    };

    let first = wire::parse_page(&first_body);
    let total = first.total;
    let first_count = first.data.len();
    let next_url = first.next.clone();
    let mut tracks: Vec<ParsedTrack> = first
        .data
        .iter()
        .filter_map(|row| wire::parse_track(row, None))
        .collect();

    let Some(next) = next_url else {
        return Ok((tracks, total));
    };

    // If Apple gave `meta.total` or an `offset=` link, fan out pages concurrently.
    let uses_offset = total.is_some() || next.contains("offset=");
    if uses_offset {
        let pages = client
            .get_pages(tracks_path, LIBRARY_PAGE_SIZE, LIBRARY_MAX_PAGES, query)
            .await?;
        let mut all = Vec::new();
        for body in &pages {
            for row in &wire::parse_page(body).data {
                if let Some(t) = wire::parse_track(row, None) {
                    all.push(t);
                }
            }
        }
        return Ok((all, total));
    }

    // Otherwise follow cursor `next` links sequentially up to `LIBRARY_MAX_PAGES`.
    let _ = first_count;
    let mut cursor = Some(next);
    for _ in 1..LIBRARY_MAX_PAGES {
        let Some(next_path) = cursor.take() else {
            break;
        };
        let body = match client.get(&next_path, query).await {
            Ok(b) => b,
            Err(AppError::NotFound(_)) => break,
            Err(e) => return Err(e),
        };
        let page = wire::parse_page(&body);
        tracks.extend(page.data.iter().filter_map(|row| wire::parse_track(row, None)));
        cursor = page.next;
    }

    Ok((tracks, total))
}

pub(crate) async fn liked_songs_parsed_with(
    client: &AppleClient,
) -> Result<Vec<ParsedTrack>, AppError> {
    let pages = match client
        .get_listing(
            "/me/library/songs",
            LIBRARY_PAGE_SIZE,
            LIBRARY_MAX_PAGES,
            LIBRARY_SONGS_QUERY,
        )
        .await
    {
        Ok(p) => p,
        Err(AppError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let mut seen_catalog = HashSet::new();
    let mut tracks = Vec::new();
    for body in pages.iter() {
        for row in &wire::parse_page(body).data {
            if let Some(t) = wire::parse_track(row, Some(&mut seen_catalog)) {
                tracks.push(t);
            }
        }
    }
    Ok(tracks)
}

pub(crate) async fn library_playlists_parsed_with(
    client: &AppleClient,
) -> Result<Vec<ParsedPlaylist>, AppError> {
    let pages = match client
        .get_listing(
            "/me/library/playlists",
            LIBRARY_PAGE_SIZE,
            LIBRARY_MAX_PAGES,
            &[("include", "catalog")],
        )
        .await
    {
        Ok(p) => p,
        Err(AppError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let mut playlists = Vec::new();
    for body in pages.iter() {
        for row in &wire::parse_page(body).data {
            if let Some(pl) = wire::parse_playlist(row) {
                playlists.push(pl);
            }
        }
    }
    Ok(playlists)
}

pub(crate) async fn library_albums_parsed_with(
    client: &AppleClient,
) -> Result<Vec<ParsedAlbum>, AppError> {
    let pages = match client
        .get_listing(
            "/me/library/albums",
            LIBRARY_PAGE_SIZE,
            LIBRARY_MAX_PAGES,
            LIBRARY_ALBUMS_QUERY,
        )
        .await
    {
        Ok(p) => p,
        Err(AppError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    let mut albums = Vec::new();
    for body in pages.iter() {
        for row in &wire::parse_page(body).data {
            if let Some(alb) = wire::parse_album(row) {
                albums.push(alb);
            }
        }
    }
    Ok(albums)
}

pub(crate) async fn library_artists_parsed_with(
    client: &AppleClient,
) -> Result<Vec<ParsedArtist>, AppError> {
    let (lib_res, fav_res) = tokio::join!(
        client.get_listing(
            "/me/library/artists",
            LIBRARY_PAGE_SIZE,
            LIBRARY_MAX_PAGES,
            &[("include", "catalog")],
        ),
        client.get("/me/favorites", &[("types", "artists"), ("limit", "100")]),
    );

    let pages = match lib_res {
        Ok(p) => Some(p),
        Err(AppError::NotFound(_)) => None,
        Err(e) => return Err(e),
    };

    let mut out = Vec::new();
    let mut seen = HashSet::new();

    if let Some(pages) = pages {
        for body in pages.iter() {
            for row in &wire::parse_page(body).data {
                if let Some(art) = wire::parse_artist(row) {
                    if seen.insert(art.item.id.clone()) {
                        out.push(art);
                    }
                }
            }
        }
    }

    // Best-effort merge of favorited artists if Apple returns a listing on GET /me/favorites.
    if let Ok(fav_body) = fav_res {
        for row in &wire::parse_page(&fav_body).data {
            if let Some(art) = wire::parse_artist(row) {
                if seen.insert(art.item.id.clone()) {
                    out.push(art);
                }
            }
        }
    }

    hydrate_artist_images(client, &mut out).await;
    Ok(out)
}

async fn hydrate_artist_images(client: &AppleClient, artists: &mut [ParsedArtist]) {
    let missing_ids: Vec<String> = artists
        .iter()
        .filter(|a| a.item.image_url.is_none() && !wire::is_library_id(&a.item.id))
        .map(|a| a.item.id.clone())
        .collect();
    if missing_ids.is_empty() {
        return;
    }

    let images = fetch_catalog_artist_images(client, &missing_ids).await;
    for artist in artists.iter_mut() {
        if artist.item.image_url.is_none() {
            if let Some((img, genres)) = images.get(&artist.item.id) {
                artist.item.image_url = img.clone();
                if artist.genres.is_empty() && !genres.is_empty() {
                    artist.genres = genres.clone();
                }
            }
        }
    }
}

async fn fetch_catalog_artist_images(
    client: &AppleClient,
    ids: &[String],
) -> HashMap<String, (Option<String>, Vec<String>)> {
    let mut map = HashMap::new();
    let chunks: Vec<String> = ids
        .chunks(ARTIST_BATCH_SIZE)
        .map(|c| c.join(","))
        .collect();

    let results = stream::iter(chunks)
        .map(|joined| async move {
            client
                .get(&client.catalog_path("/artists"), &[("ids", &joined)])
                .await
        })
        .buffered(6)
        .collect::<Vec<_>>()
        .await;

    for res in results {
        let Ok(body) = res else { continue };
        for row in &wire::parse_page(&body).data {
            if let Some(parsed) = wire::parse_artist(row) {
                map.insert(parsed.item.id, (parsed.item.image_url, parsed.genres));
            }
        }
    }
    map
}

pub(crate) async fn recently_played_with(client: &AppleClient) -> Result<Vec<TrackItem>, AppError> {
    let body = match client
        .get(
            "/me/recent/played/tracks",
            &[
                ("limit", "30"),
                ("types", "songs"),
                ("include[songs]", "artists,albums"),
            ],
        )
        .await
    {
        Ok(b) => b,
        Err(AppError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    Ok(wire::parse_page(&body)
        .data
        .iter()
        .filter_map(|row| wire::parse_track(row, None).map(|t| t.item))
        .collect())
}

pub(crate) async fn heavy_rotation_tracks_with(
    client: &AppleClient,
) -> Result<Vec<TrackItem>, AppError> {
    let (hr_res, recent_res) = tokio::join!(
        client.get(
            "/me/history/heavy-rotation",
            &[("limit", "25"), ("include[albums]", "artists")],
        ),
        recently_played_with(client),
    );

    let mut tracks = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut album_ids = Vec::new();

    if let Ok(body) = hr_res {
        for row in &wire::parse_page(&body).data {
            match wire::str_field(row, "type") {
                Some("songs" | "library-songs") => {
                    if let Some(t) = wire::parse_track(row, None).map(|p| p.item) {
                        if seen_ids.insert(t.id.clone()) {
                            tracks.push(t);
                        }
                    }
                }
                Some("albums" | "library-albums") => {
                    if let Some(alb) = wire::parse_album(row) {
                        if !wire::is_library_id(&alb.item.id)
                            && !album_ids.contains(&alb.item.id)
                            && album_ids.len() < 8
                        {
                            album_ids.push(alb.item.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if tracks.len() < 20 && !album_ids.is_empty() {
        let fetched = stream::iter(album_ids)
            .map(|aid| async move { album_with(client, &aid).await })
            .buffered(6)
            .collect::<Vec<_>>()
            .await;

        for res in fetched {
            let Ok(detail) = res else { continue };
            for t in detail.tracks.into_iter().take(4) {
                if seen_ids.insert(t.id.clone()) {
                    tracks.push(t);
                }
            }
        }
    }

    if let Ok(recent) = recent_res {
        for t in recent {
            if tracks.len() >= 50 {
                break;
            }
            if seen_ids.insert(t.id.clone()) {
                tracks.push(t);
            }
        }
    }

    tracks.truncate(50);
    Ok(tracks)
}

pub(crate) async fn heavy_rotation_artists_with(
    client: &AppleClient,
) -> Result<Vec<ArtistItem>, AppError> {
    let (hr_res, recent_res) = tokio::join!(
        client.get(
            "/me/history/heavy-rotation",
            &[("limit", "25"), ("include[albums]", "artists")],
        ),
        recently_played_with(client),
    );

    let mut artists = Vec::new();
    let mut seen_ids = HashSet::new();

    if let Ok(body) = hr_res {
        for row in &wire::parse_page(&body).data {
            match wire::str_field(row, "type") {
                Some("artists" | "library-artists") => {
                    if let Some(a) = wire::parse_artist(row).map(|p| p.item) {
                        if !wire::is_synthetic_artist_id(&a.id) && seen_ids.insert(a.id.clone()) {
                            artists.push(a);
                        }
                    }
                }
                Some("albums" | "library-albums") => {
                    if let Some(alb) = wire::parse_album(row) {
                        for a in alb.item.artists {
                            if !wire::is_synthetic_artist_id(&a.id) && seen_ids.insert(a.id.clone())
                            {
                                artists.push(a);
                            }
                        }
                    }
                }
                Some("songs" | "library-songs") => {
                    if let Some(t) = wire::parse_track(row, None) {
                        for a in t.item.artists {
                            if !wire::is_synthetic_artist_id(&a.id) && seen_ids.insert(a.id.clone())
                            {
                                artists.push(a);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if let Ok(recent) = recent_res {
        for t in recent {
            if artists.len() >= 50 {
                break;
            }
            for a in t.artists {
                if !wire::is_synthetic_artist_id(&a.id) && seen_ids.insert(a.id.clone()) {
                    artists.push(a);
                }
            }
        }
    }

    artists.truncate(50);

    // Enrich missing avatars for catalog artists.
    let missing: Vec<String> = artists
        .iter()
        .filter(|a| a.image_url.is_none() && !wire::is_library_id(&a.id))
        .map(|a| a.id.clone())
        .collect();
    if !missing.is_empty() {
        let images = fetch_catalog_artist_images(client, &missing).await;
        for a in artists.iter_mut() {
            if a.image_url.is_none() {
                if let Some((img, _)) = images.get(&a.id) {
                    a.image_url = img.clone();
                }
            }
        }
    }

    Ok(artists)
}

pub(crate) async fn new_releases_with(client: &AppleClient) -> Result<Vec<AlbumItem>, AppError> {
    let charts_body = client
        .get(
            &client.catalog_path("/charts"),
            &[
                ("types", "albums"),
                ("limit", "25"),
                ("include[albums]", "artists"),
            ],
        )
        .await;

    let mut albums = match charts_body {
        Ok(b) => wire::parse_charts_albums(&b),
        Err(_) => Vec::new(),
    };

    if albums.is_empty() {
        if let Ok(rec_body) = client
            .get(
                "/me/recommendations",
                &[("limit", "12"), ("include[albums]", "artists")],
            )
            .await
        {
            let mut seen = HashSet::new();
            for group in &wire::parse_page(&rec_body).data {
                for row in wire::rel_rows(group, "contents") {
                    if wire::str_field(row, "type") == Some("albums") {
                        if let Some(alb) = wire::parse_album(row) {
                            if seen.insert(alb.item.id.clone()) {
                                albums.push(alb.item);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(albums)
}

pub(crate) async fn home_feed_with(client: &AppleClient) -> Result<HomeFeed, AppError> {
    let charts_path = client.catalog_path("/charts");
    let (recs_res, recent_res, charts_res) = tokio::join!(
        client.get(
            "/me/recommendations",
            &[("limit", "12"), ("include[albums]", "artists")],
        ),
        client.get(
            "/me/recent/played",
            &[("limit", "10"), ("include[albums]", "artists")],
        ),
        client.get(
            &charts_path,
            &[
                ("types", "songs,albums,playlists"),
                ("limit", "12"),
                ("include[songs]", "artists,albums"),
                ("include[albums]", "artists"),
            ],
        ),
    );

    if let (Err(e1), Err(_), Err(_)) = (&recs_res, &recent_res, &charts_res) {
        return Err(AppError::Network(e1.to_string()));
    }

    let (mut sections, fallback_recents) = match recs_res {
        Ok(body) => wire::parse_recommendations(&body),
        Err(_) => (Vec::new(), Vec::new()),
    };

    let live_shortcuts: Vec<HomeItem> = match recent_res {
        Ok(body) => wire::parse_page(&body)
            .data
            .iter()
            .filter_map(wire::parse_home_item)
            .collect(),
        Err(_) => Vec::new(),
    };

    let shortcuts = if !live_shortcuts.is_empty() {
        live_shortcuts
    } else {
        fallback_recents
    };

    if let Ok(charts_body) = charts_res {
        let chart_sections = wire::parse_charts_sections(&charts_body);
        sections.extend(chart_sections);
    }

    Ok(HomeFeed {
        greeting: None,
        sections,
        shortcuts,
    })
}

pub(crate) async fn track_radio_with(
    client: &AppleClient,
    track_id: &str,
) -> Result<RadioResult, AppError> {
    let seed_id = resolve_catalog_song_id(client, track_id).await?;
    let Some(seed_id) = seed_id else {
        return Ok(RadioResult {
            title: None,
            playlist_id: None,
            tracks: Vec::new(),
        });
    };

    let limit_str = STATION_PULL_LIMIT.to_string();
    let mut tracks = Vec::new();
    let mut title = None;

    let station_id = match client.station_for_seed(&seed_id) {
        Some(existing) => existing,
        None => {
            let body = client
                .post(
                    "/me/stations/continuous",
                    &[
                        ("with", "tracks"),
                        ("limit[results:tracks]", &limit_str),
                        ("include[songs]", "artists,albums"),
                    ],
                    Some(json!({
                        "data": [{ "id": seed_id, "type": "songs" }]
                    })),
                )
                .await?;
            let (station_id, station_title, first_tracks) = wire::parse_station_seed(&body)
                .ok_or_else(|| {
                    AppError::Network("Apple Music created no station for that track".into())
                })?;
            client.remember_station(&seed_id, &station_id);
            title = station_title;
            tracks.extend(first_tracks);
            station_id
        }
    };

    for _ in 0..STATION_PULLS {
        if tracks.len() >= STATION_PULL_LIMIT * STATION_PULLS {
            break;
        }
        let body = client
            .post(
                &format!("/me/stations/next-tracks/{}", encode_segment(&station_id)),
                &[
                    ("limit", &limit_str),
                    ("include[songs]", "artists,albums"),
                ],
                None,
            )
            .await?;
        let more: Vec<TrackItem> = wire::parse_page(&body)
            .data
            .iter()
            .filter_map(|row| wire::parse_track(row, None).map(|t| t.item))
            .collect();
        if more.is_empty() {
            break;
        }
        tracks.extend(more);
    }

    let mut seen = HashSet::new();
    tracks.retain(|t| t.id != seed_id && seen.insert(t.id.clone()));

    Ok(RadioResult {
        title,
        playlist_id: None,
        tracks,
    })
}

async fn resolve_catalog_song_id(
    client: &AppleClient,
    track_id: &str,
) -> Result<Option<String>, AppError> {
    let trimmed = track_id.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if !wire::is_library_id(trimmed) {
        return Ok(Some(trimmed.to_owned()));
    }
    let body = client
        .get(
            &format!("/me/library/songs/{}", encode_segment(trimmed)),
            &[("include[library-songs]", "catalog")],
        )
        .await?;
    let cat_id = wire::parse_page(&body)
        .data
        .first()
        .and_then(|row| wire::parse_track(row, None))
        .map(|t| t.item.id)
        .filter(|id| !wire::is_library_id(id));
    Ok(cat_id)
}

async fn lookup_library_id(
    client: &AppleClient,
    kind: &str,
    catalog_id: &str,
) -> Result<Option<String>, AppError> {
    let path = client.catalog_path(&format!("/{kind}/{}/library", encode_segment(catalog_id)));
    match client.get(&path, &[]).await {
        Ok(body) => Ok(wire::parse_page(&body)
            .data
            .first()
            .and_then(|row| wire::str_field(row, "id"))
            .map(str::to_owned)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub(crate) async fn save_track_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "track")?;
    if wire::is_library_id(id) {
        return Err(AppError::InvalidInput(
            "Cannot add a library-only track ID to Apple Music library".into(),
        ));
    }
    client.post("/me/library", &[("ids[songs]", id)], None).await?;
    Ok(())
}

pub(crate) async fn unsave_track_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "track")?;
    let lib_id = if wire::is_library_id(id) {
        Some(id.to_owned())
    } else {
        lookup_library_id(client, "songs", id).await?
    };
    if let Some(lid) = lib_id {
        client
            .delete(&format!("/me/library/songs/{}", encode_segment(&lid)), &[])
            .await?;
    }
    Ok(())
}

pub(crate) async fn saved_track_ids_with(
    client: &AppleClient,
    ids: &[String],
) -> Result<Vec<String>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let wanted: Vec<&str> = ids
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }

    // Small batches are resolved directly via `/catalog/{sf}/songs/{id}/library`; larger sets
    // intersect against the 30s-cached `/me/library/songs` listing.
    if wanted.len() <= 20 {
        let checks = stream::iter(wanted.iter().copied())
            .map(|id| async move {
                if wire::is_library_id(id) {
                    return Ok::<Option<String>, AppError>(Some(id.to_owned()));
                }
                match lookup_library_id(client, "songs", id).await {
                    Ok(Some(_)) => Ok(Some(id.to_owned())),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            })
            .buffered(8)
            .collect::<Vec<_>>()
            .await;

        let mut present = Vec::new();
        let mut first_err = None;
        for res in checks {
            match res {
                Ok(Some(id)) => present.push(id),
                Ok(None) => {}
                Err(e) if first_err.is_none() => first_err = Some(e),
                Err(_) => {}
            }
        }
        if present.is_empty() {
            if let Some(e) = first_err {
                return Err(e);
            }
        }
        return Ok(present);
    }

    let lib = liked_songs_parsed_with(client).await?;
    let lib_ids: HashSet<String> = lib
        .into_iter()
        .flat_map(|t| [Some(t.item.id), t.library_id])
        .flatten()
        .collect();
    Ok(wanted
        .into_iter()
        .filter(|id| lib_ids.contains(*id))
        .map(str::to_owned)
        .collect())
}

pub(crate) async fn save_album_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "album")?;
    if wire::is_library_id(id) {
        return Err(AppError::InvalidInput(
            "Cannot add a library-only album ID to Apple Music library".into(),
        ));
    }
    client
        .post("/me/library", &[("ids[albums]", id)], None)
        .await?;
    Ok(())
}

pub(crate) async fn unsave_album_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "album")?;
    let lib_id = if wire::is_library_id(id) {
        Some(id.to_owned())
    } else {
        lookup_library_id(client, "albums", id).await?
    };
    if let Some(lid) = lib_id {
        client
            .delete(&format!("/me/library/albums/{}", encode_segment(&lid)), &[])
            .await?;
    }
    Ok(())
}

pub(crate) async fn is_album_saved_with(client: &AppleClient, id: &str) -> Result<bool, AppError> {
    let id = require_id(id, "album")?;
    if wire::is_library_id(id) {
        return match client
            .get(&format!("/me/library/albums/{}", encode_segment(id)), &[])
            .await
        {
            Ok(_) => Ok(true),
            Err(AppError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        };
    }
    Ok(lookup_library_id(client, "albums", id).await?.is_some())
}

pub(crate) async fn follow_artist_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "artist")?;
    let key = if wire::is_library_id(id) {
        "ids[library-artists]"
    } else {
        "ids[artists]"
    };
    client.post("/me/favorites", &[(key, id)], None).await?;
    Ok(())
}

pub(crate) async fn unfollow_artist_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "artist")?;
    let key = if wire::is_library_id(id) {
        "ids[library-artists]"
    } else {
        "ids[artists]"
    };
    client.delete("/me/favorites", &[(key, id)]).await
}

pub(crate) async fn is_artist_followed_with(
    client: &AppleClient,
    id: &str,
) -> Result<bool, AppError> {
    let id = require_id(id, "artist")?;
    if wire::is_synthetic_artist_id(id) {
        return Ok(false);
    }

    if wire::is_library_id(id) {
        return check_library_personal_rating(client, "library-artists", id).await;
    }

    // Check catalog rating first; if not present/404, fall back to library artist search + rating.
    match client
        .get(&format!("/me/ratings/artists/{}", encode_segment(id)), &[])
        .await
    {
        Ok(body) => {
            let val = wire::parse_page(&body)
                .data
                .first()
                .and_then(wire::attrs_of)
                .and_then(|a| wire::i64_field(a, "value"));
            if val == Some(1) {
                return Ok(true);
            }
        }
        Err(AppError::NotFound(_)) => {}
        Err(_) => {}
    }

    // Resolve library artist ID via name search, then inspect personalRating.
    let Ok(artist_body) = client
        .get(
            &client.catalog_path(&format!("/artists/{}", encode_segment(id))),
            &[("fields[artists]", "name")],
        )
        .await
    else {
        return Ok(false);
    };
    let Some(name) = wire::parse_page(&artist_body)
        .data
        .first()
        .and_then(wire::attrs_of)
        .and_then(|a| wire::str_field(a, "name"))
        .map(str::to_owned)
    else {
        return Ok(false);
    };

    let search_body = client
        .get(
            "/me/library/search",
            &[
                ("term", &name),
                ("types", "library-artists"),
                ("include[library-artists]", "catalog"),
                ("limit", "25"),
            ],
        )
        .await?;
    let lib_artist_id = find_library_artist_id(&search_body, id);
    match lib_artist_id {
        Some(lid) => check_library_personal_rating(client, "library-artists", &lid).await,
        None => Ok(false),
    }
}

fn find_library_artist_id(body: &str, catalog_id: &str) -> Option<String> {
    let page = wire::parse_page(body);
    let inline = page
        .results
        .as_ref()
        .and_then(|r| r.get("library-artists"))
        .and_then(|r| r.get("data"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    let mapped = page
        .resources
        .as_ref()
        .and_then(|r| r.get("library-artists"))
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.values());

    inline
        .chain(mapped)
        .find(|row| {
            wire::catalog_rel(row).and_then(|c| wire::str_field(c, "id")) == Some(catalog_id)
        })
        .and_then(|row| wire::str_field(row, "id"))
        .map(str::to_owned)
}

async fn check_library_personal_rating(
    client: &AppleClient,
    kind: &str,
    lib_id: &str,
) -> Result<bool, AppError> {
    let ids_key = format!("ids[{kind}]");
    let fields_key = format!("fields[{kind}]");
    match client
        .get("/me/library", &[(&ids_key, lib_id), (&fields_key, "personalRating")])
        .await
    {
        Ok(body) => Ok(wire::parse_page(&body).data.iter().any(|row| {
            wire::attrs_of(row).and_then(|a| wire::i64_field(a, "personalRating")) == Some(1)
        })),
        Err(AppError::NotFound(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(crate) async fn create_playlist_with(
    client: &AppleClient,
    name: &str,
    description: Option<&str>,
    public: bool,
) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("Playlist name is required".into()));
    }

    let asked_at_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);

    let mut attrs = serde_json::Map::new();
    attrs.insert("name".into(), json!(name));
    if let Some(desc) = description.map(str::trim).filter(|d| !d.is_empty()) {
        attrs.insert("description".into(), json!(desc));
    }

    let body = client
        .post(
            "/me/library/playlists",
            &[],
            Some(json!({ "attributes": attrs })),
        )
        .await?;

    let id = match wire::parse_page(&body)
        .data
        .first()
        .and_then(|row| wire::str_field(row, "id"))
        .map(str::to_owned)
    {
        Some(id) => id,
        None => find_created_playlist(client, name, asked_at_secs).await?,
    };

    if public {
        let _ = update_playlist_with(client, &id, None, None, Some(true)).await;
    }

    Ok(id)
}

async fn find_created_playlist(
    client: &AppleClient,
    name: &str,
    asked_at_secs: i64,
) -> Result<String, AppError> {
    let pages = client
        .get_pages("/me/library/playlists", LIBRARY_PAGE_SIZE, LIBRARY_MAX_PAGES, &[])
        .await?;

    let mut matches: Vec<(i64, String)> = Vec::new();
    for body in &pages {
        for row in &wire::parse_page(body).data {
            let Some(attrs) = wire::attrs_of(row) else {
                continue;
            };
            if wire::bool_field(attrs, "canEdit") != Some(true) {
                continue;
            }
            if wire::str_field(attrs, "name") != Some(name) {
                continue;
            }
            let Some(added_iso) = wire::str_field(attrs, "dateAdded") else {
                continue;
            };
            let added_secs = crate::library::parse_iso8601(added_iso) / 1000;
            if added_secs >= asked_at_secs - CLOCK_SKEW_SECS {
                if let Some(id) = wire::str_field(row, "id") {
                    matches.push((added_secs, id.to_owned()));
                }
            }
        }
    }

    match matches.len() {
        0 => Err(AppError::Network(
            "Apple Music created a playlist without returning its id".into(),
        )),
        1 => Ok(matches.remove(0).1),
        _ => Err(AppError::Network(
            "Apple Music returned no playlist id and multiple newly-created playlists match".into(),
        )),
    }
}

pub(crate) async fn update_playlist_with(
    client: &AppleClient,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    public: Option<bool>,
) -> Result<(), AppError> {
    let id = require_id(id, "playlist")?;
    let mut attrs = serde_json::Map::new();
    if let Some(n) = name.map(str::trim).filter(|n| !n.is_empty()) {
        attrs.insert("name".into(), json!(n));
    }
    if let Some(d) = description {
        attrs.insert("description".into(), json!(d.trim()));
    }
    if let Some(p) = public {
        attrs.insert("isPublic".into(), json!(p));
    }
    if attrs.is_empty() {
        return Ok(());
    }

    client
        .patch(
            &format!("/me/library/playlists/{}", encode_segment(id)),
            json!({ "attributes": attrs }),
        )
        .await
}

pub(crate) async fn delete_playlist_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "playlist")?;
    client
        .delete(&format!("/me/library/playlists/{}", encode_segment(id)), &[])
        .await
}

pub(crate) async fn follow_playlist_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "playlist")?;
    if wire::is_library_id(id) {
        return Ok(());
    }
    client
        .post("/me/library", &[("ids[playlists]", id)], None)
        .await?;
    Ok(())
}

pub(crate) async fn unfollow_playlist_with(client: &AppleClient, id: &str) -> Result<(), AppError> {
    let id = require_id(id, "playlist")?;
    let lib_id = if wire::is_library_id(id) {
        Some(id.to_owned())
    } else {
        lookup_library_id(client, "playlists", id).await?
    };
    if let Some(lid) = lib_id {
        client
            .delete(
                &format!("/me/library/playlists/{}", encode_segment(&lid)),
                &[],
            )
            .await?;
    }
    Ok(())
}

pub(crate) async fn is_playlist_followed_with(
    client: &AppleClient,
    id: &str,
) -> Result<bool, AppError> {
    let id = require_id(id, "playlist")?;
    if wire::is_library_id(id) {
        return match client
            .get(&format!("/me/library/playlists/{}", encode_segment(id)), &[])
            .await
        {
            Ok(_) => Ok(true),
            Err(AppError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        };
    }
    Ok(lookup_library_id(client, "playlists", id).await?.is_some())
}

pub(crate) async fn add_track_to_playlist_with(
    client: &AppleClient,
    pl: &str,
    track: &str,
) -> Result<(), AppError> {
    let pl = require_id(pl, "playlist")?;
    let track = require_id(track, "track")?;
    let kind = if wire::is_library_id(track) {
        "library-songs"
    } else {
        "songs"
    };
    client
        .post(
            &format!("/me/library/playlists/{}/tracks", encode_segment(pl)),
            &[],
            Some(json!({ "data": [{ "id": track, "type": kind }] })),
        )
        .await?;
    Ok(())
}

pub(crate) async fn remove_track_from_playlist_with(
    client: &AppleClient,
    pl: &str,
    track: &str,
) -> Result<(), AppError> {
    let pl = require_id(pl, "playlist")?;
    let track = require_id(track, "track")?;

    let path = format!("/me/library/playlists/{}/tracks", encode_segment(pl));
    let limit = LIBRARY_PAGE_SIZE.to_string();
    let mut found_row_id: Option<String> = None;

    for page_idx in 0..LIBRARY_MAX_PAGES {
        let offset = (page_idx * LIBRARY_PAGE_SIZE).to_string();
        let body = client
            .get(&path, &[("limit", &limit), ("offset", &offset)])
            .await?;
        let page = wire::parse_page(&body);
        for row in &page.data {
            let row_id = wire::str_field(row, "id");
            let params = wire::attrs_of(row)
                .and_then(|a| a.get("playParams"))
                .filter(|v| v.is_object());
            let matched = row_id == Some(track)
                || params.is_some_and(|p| {
                    wire::str_field(p, "catalogId") == Some(track)
                        || wire::str_field(p, "reportingId") == Some(track)
                        || wire::str_field(p, "id") == Some(track)
                });
            if matched {
                found_row_id = row_id.map(str::to_owned);
                break;
            }
        }
        if found_row_id.is_some() || page.data.is_empty() || page.next.is_none() {
            break;
        }
    }

    let row_id = found_row_id
        .ok_or_else(|| AppError::NotFound("Track is not in that playlist".into()))?;
    client
        .delete(&path, &[("ids[library-songs]", &row_id), ("mode", "all")])
        .await
}

pub(crate) async fn report_play_with(
    client: &AppleClient,
    track_id: &str,
) -> Result<(), AppError> {
    let track_id = track_id.trim();
    if track_id.is_empty() || wire::is_library_id(track_id) || track_id.parse::<u64>().is_err() {
        return Ok(());
    }

    let song_body = client
        .get(
            &client.catalog_path(&format!("/songs/{}", encode_segment(track_id))),
            &[
                ("include[songs]", "albums"),
                ("fields[songs]", "durationInMillis"),
                ("fields[albums]", "url"),
            ],
        )
        .await
        .unwrap_or_default();

    let page = wire::parse_page(&song_body);
    let song = page.data.first();
    let duration_ms = song
        .and_then(wire::attrs_of)
        .and_then(|a| wire::i64_field(a, "durationInMillis"))
        .unwrap_or(0);
    let album_id = song
        .and_then(|s| wire::rel_rows(s, "albums").first())
        .and_then(|a| wire::str_field(a, "id"))
        .map(str::to_owned);

    let persistent_id = format!("{:016x}", rand::thread_rng().next_u64());
    let mut item = json!({
        "build-version": BEACON_BUILD,
        "developer-token": client.bearer(),
        "event-type": 1,
        "event-reason-hint-type": 1,
        "type": 1,
        "ids": { "subscription-adam-id": track_id },
        "internal-build": false,
        "media-type": 0,
        "media-duration-in-milliseconds": duration_ms,
        "milliseconds-since-play": 1,
        "offline": false,
        "persistent-id": persistent_id,
        "play-mode": {
            "auto-play-mode": 1,
            "repeat-play-mode": 1,
            "shuffle-play-mode": 1
        },
        "private-enabled": false,
        "sb-enabled": true,
        "siri-initiated": false,
        "source-type": 16,
        "start-position-in-milliseconds": 0,
        "store-front": client.storefront(),
        "user-agent": BEACON_AGENT,
        "user-token": client.user_token(),
        "utc-offset-in-seconds": 0
    });

    if let Some(alb) = album_id {
        item["container-type"] = json!(3);
        item["container-ids"] = json!({ "album-adam-id": alb });
    }

    let body = json!({
        "client_id": "JSCLIENT",
        "event_type": "JSPLAY",
        "data": [item]
    });
    client.post_activity(&body).await
}

fn require_id<'a>(id: &'a str, kind: &str) -> Result<&'a str, AppError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(format!("Apple Music {kind} id is empty")));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("apple")
            .join("testdata")
            .join(name);
        std::fs::read_to_string(path).unwrap()
    }

    fn jwt() -> String {
        let h = URL_SAFE_NO_PAD.encode(r#"{"alg":"ES256"}"#);
        let p = URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":2000000000,"p":"{}"}}"#, "a".repeat(60)));
        format!("{h}.{p}.sig")
    }

    #[test]
    fn maps_spotify_search_types_and_ignores_podcasts() {
        assert_eq!(
            map_search_types("track,artist,album,playlist,show,episode"),
            "songs,artists,albums,playlists"
        );
        assert_eq!(map_search_types("show,episode"), "");
        assert_eq!(map_search_types("track, track"), "songs");
    }

    #[tokio::test]
    async fn exercises_endpoints_station_reuse_and_playlist_mutations() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let calls_ref = calls.clone();

        let search_json = fixture("search.json");
        let album_json = fixture("catalog_album.json");
        let artist_json = fixture("catalog_artist.json");
        let song_json = fixture("catalog_song.json");
        let recs_json = fixture("recommendations.json");
        let charts_json = fixture("charts.json");
        let station_json = fixture("station.json");
        let pl_json = fixture("library_playlists.json");

        let server = tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).into_owned();
                let first_line = req.lines().next().unwrap_or("");
                let mut parts = first_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let path = parts.next().unwrap_or("");
                calls_ref.lock().unwrap().push(format!("{method} {path}"));

                let (status, body) = if path.starts_with("/v1/me/storefront") {
                    ("200 OK", r#"{"data":[{"id":"us"}]}"#.to_owned())
                } else if path.starts_with("/v1/catalog/us/search") {
                    ("200 OK", search_json.clone())
                } else if path.starts_with("/v1/catalog/us/albums/1691419979") {
                    ("200 OK", album_json.clone())
                } else if path.starts_with("/v1/catalog/us/artists/5468295") {
                    ("200 OK", artist_json.clone())
                } else if path.starts_with("/v1/catalog/us/songs/1440857781") {
                    ("200 OK", song_json.clone())
                } else if path.starts_with("/v1/me/recommendations") {
                    ("200 OK", recs_json.clone())
                } else if path.starts_with("/v1/me/recent/played") {
                    ("200 OK", r#"{"data":[]}"#.to_owned())
                } else if path.starts_with("/v1/catalog/us/charts") {
                    ("200 OK", charts_json.clone())
                } else if path.starts_with("/v1/me/stations/continuous") {
                    ("200 OK", station_json.clone())
                } else if path.starts_with("/v1/me/stations/next-tracks/ra.1440857781") {
                    ("200 OK", r#"{"data":[]}"#.to_owned())
                } else if path == "/v1/me/library/playlists" && method == "POST" {
                    // Simulate Apple 201 with empty body -> lookup fallback
                    ("201 Created", r#"{"data":[]}"#.to_owned())
                } else if path.starts_with("/v1/me/library/playlists") && method == "GET" {
                    if path.contains("/tracks") {
                        (
                            "200 OK",
                            r#"{"data":[{"id":"i.row9","type":"library-songs","attributes":{"name":"Blue Monday","playParams":{"catalogId":"1440857781"}}}]}"#.to_owned(),
                        )
                    } else {
                        // Return a playlist matching name + fresh dateAdded
                        let fresh = r#"{"data":[{"id":"p.created1","type":"library-playlists","attributes":{"name":"Road Trip","canEdit":true,"dateAdded":"2099-01-01T00:00:00Z"}}]}"#;
                        if req.contains("include=catalog") {
                            ("200 OK", pl_json.clone())
                        } else {
                            ("200 OK", fresh.to_owned())
                        }
                    }
                } else if path.starts_with("/v1/me/library/playlists/p.own1/tracks")
                    && method == "DELETE"
                {
                    assert!(path.contains("ids%5Blibrary-songs%5D=i.row9") || path.contains("ids[library-songs]=i.row9"));
                    ("204 No Content", String::new())
                } else if path.starts_with("/v1/catalog/us/playlists/pl.1/library") {
                    ("200 OK", r#"{"data":[{"id":"p.lib1"}]}"#.to_owned())
                } else if path.starts_with("/v1/me/library/playlists/p.lib1") && method == "DELETE"
                {
                    ("204 No Content", String::new())
                } else {
                    ("404 Not Found", "{}".to_owned())
                };

                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
            }
        });

        let client = AppleClient::connect_with_base(
            "user-tok",
            &jwt(),
            &format!("http://{addr}/v1"),
            None,
        )
        .await
        .unwrap();

        let sr = search_with(&client, "new order", "track,show").await.unwrap();
        assert_eq!(sr.tracks.len(), 1);
        assert!(sr.shows.is_empty());

        let alb = album_with(&client, "1691419979").await.unwrap();
        assert_eq!(alb.tracks.len(), 2);

        let art = artist_with(&client, "5468295").await.unwrap();
        assert_eq!(art.name, "Daft Punk");
        let ext = artist_extras_with(&client, "5468295").await.unwrap();
        assert_eq!(ext.related.len(), 1);
        let ov = artist_overview_with(&client, "5468295").await.unwrap();
        assert!(ov.header_image.is_some());

        let isrc = track_isrc_with(&client, "1440857781").await.unwrap();
        assert_eq!(isrc.as_deref(), Some("GBAAP8300001"));

        let feed = home_feed_with(&client).await.unwrap();
        assert!(!feed.sections.is_empty());
        assert_eq!(feed.shortcuts.len(), 1, "falls back to recents group when live is empty");

        // Station created on first call and reused on second call
        let r1 = track_radio_with(&client, "1440857781").await.unwrap();
        assert_eq!(r1.tracks.len(), 1);
        assert_eq!(r1.tracks[0].id, "1001");
        let _r2 = track_radio_with(&client, "1440857781").await.unwrap();
        let continuous_count = calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.contains("/me/stations/continuous"))
            .count();
        assert_eq!(continuous_count, 1, "station remembered per seed");

        // Create playlist with empty 201 fallback lookup
        let created_id = create_playlist_with(&client, "Road Trip", Some("Fun"), false)
            .await
            .unwrap();
        assert_eq!(created_id, "p.created1");

        // Remove track resolves row id i.row9 from catalogId 1440857781
        remove_track_from_playlist_with(&client, "p.own1", "1440857781")
            .await
            .unwrap();

        // Unfollow catalog playlist resolves p.lib1 and deletes it
        unfollow_playlist_with(&client, "pl.1").await.unwrap();

        server.abort();
    }
}
