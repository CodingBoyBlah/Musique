//! batched extended-metadata -> the app's own item types.
//!
//! `/extended-metadata/v0/extended-metadata` takes any number of entity uris in
//! one protobuf request and answers with a payload per (uri, extension kind).
//! TRACK_V4 / ALBUM_V4 / ARTIST_V4 carry the classic metadata protos, which is
//! enough to render a track row with cover art without touching the web api -
//! so anything internal that hands back bare uris (radio, stations, spotify-made
//! playlists, the home feed) can be turned into real rows here in one go.

use std::collections::HashMap;

use librespot_core::SpotifyId;
use librespot_protocol::{
    extended_metadata::{BatchedEntityRequest, EntityRequest, ExtensionQuery},
    extension_kind::ExtensionKind,
    metadata,
};
use protobuf::{EnumOrUnknown, Message};
use tauri::{AppHandle, Manager};

use super::{cache, spclient};
use crate::{
    errors::AppError,
    spotify::types::{AlbumItem, ArtistItem, TrackItem},
    state::AppState,
};

/// catalogue metadata barely changes, a week is plenty fresh
const TTL: i64 = 7 * cache::DAY;
/// entities per request. the official client sends batches in the low hundreds
const CHUNK: usize = 100;

fn kind_name(kind: ExtensionKind) -> String {
    format!("{kind:?}")
}

/// raw payloads for `uris` under one extension kind, keyed by uri. uris the
/// backend has nothing for are simply absent from the map.
pub async fn fetch_batch(
    app: &AppHandle,
    kind: ExtensionKind,
    uris: &[String],
    ttl_ms: i64,
) -> Result<HashMap<String, Vec<u8>>, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let tag = kind_name(kind);

    let mut out: HashMap<String, Vec<u8>> = HashMap::new();
    let mut misses: Vec<String> = Vec::new();
    for uri in uris {
        if out.contains_key(uri) || misses.contains(uri) {
            continue;
        }
        match cache::get(&pool, uri, &tag, ttl_ms).await {
            Some(bytes) => {
                out.insert(uri.clone(), bytes);
            }
            None => misses.push(uri.clone()),
        }
    }
    if misses.is_empty() {
        return Ok(out);
    }

    let session = spclient::session(app).await;
    for chunk in misses.chunks(CHUNK) {
        let live = match &session {
            Ok(s) => {
                let req = BatchedEntityRequest {
                    entity_request: chunk
                        .iter()
                        .map(|uri| EntityRequest {
                            entity_uri: uri.clone(),
                            query: vec![ExtensionQuery {
                                extension_kind: EnumOrUnknown::new(kind),
                                ..Default::default()
                            }],
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                };
                s.spclient().get_extended_metadata(req).await.map_err(spclient::map_err)
            }
            Err(e) => Err(AppError::Playback(e.to_string())),
        };

        match live {
            Ok(res) => {
                for arr in res.extended_metadata {
                    for data in arr.extension_data {
                        let Some(any) = data.extension_data.as_ref() else { continue };
                        if any.value.is_empty() {
                            continue;
                        }
                        cache::put(&pool, &data.entity_uri, &tag, &any.value).await;
                        out.insert(data.entity_uri.clone(), any.value.clone());
                    }
                }
            }
            Err(e) => {
                // offline / endpoint hiccup: stale beats nothing
                let mut any_stale = false;
                for uri in chunk {
                    if let Some(bytes) = cache::get_stale(&pool, uri, &tag).await {
                        out.insert(uri.clone(), bytes);
                        any_stale = true;
                    }
                }
                if !any_stale && out.is_empty() {
                    return Err(e);
                }
            }
        }
    }
    Ok(out)
}

fn decode_all<M: Message>(raw: HashMap<String, Vec<u8>>) -> HashMap<String, M> {
    raw.into_iter()
        .filter_map(|(uri, bytes)| M::parse_from_bytes(&bytes).ok().map(|m| (uri, m)))
        .collect()
}

pub async fn track_protos(app: &AppHandle, ids: &[String]) -> Result<HashMap<String, metadata::Track>, AppError> {
    let uris: Vec<String> = ids.iter().map(|id| to_uri("track", id)).collect();
    let raw = fetch_batch(app, ExtensionKind::TRACK_V4, &uris, TTL).await?;
    Ok(decode_all(raw))
}

pub async fn album_proto(app: &AppHandle, id: &str) -> Result<metadata::Album, AppError> {
    let uri = to_uri("album", id);
    let raw = fetch_batch(app, ExtensionKind::ALBUM_V4, std::slice::from_ref(&uri), TTL).await?;
    decode_all::<metadata::Album>(raw)
        .remove(&uri)
        .ok_or_else(|| AppError::NotFound(format!("no metadata for {uri}")))
}

pub async fn artist_proto(app: &AppHandle, id: &str, ttl_ms: i64) -> Result<metadata::Artist, AppError> {
    let uri = to_uri("artist", id);
    let raw = fetch_batch(app, ExtensionKind::ARTIST_V4, std::slice::from_ref(&uri), ttl_ms).await?;
    decode_all::<metadata::Artist>(raw)
        .remove(&uri)
        .ok_or_else(|| AppError::NotFound(format!("no metadata for {uri}")))
}

/// hydrate bare track ids (or uris) into full rows, keeping the input order.
/// ids spotify has nothing for are dropped rather than failing the batch.
pub async fn tracks(app: &AppHandle, ids: &[String]) -> Result<Vec<TrackItem>, AppError> {
    let ids: Vec<String> = ids.iter().map(|i| spclient::uri_id(i).to_string()).collect();
    let protos = track_protos(app, &ids).await?;
    Ok(ids
        .iter()
        .filter_map(|id| protos.get(&to_uri("track", id)).and_then(track_item))
        .collect())
}

/// batch-hydrate album ids into cards, order kept
pub async fn albums(app: &AppHandle, ids: &[String]) -> Result<Vec<AlbumItem>, AppError> {
    let uris: Vec<String> = ids.iter().map(|id| to_uri("album", spclient::uri_id(id))).collect();
    let raw = fetch_batch(app, ExtensionKind::ALBUM_V4, &uris, TTL).await?;
    let protos: HashMap<String, metadata::Album> = decode_all(raw);
    Ok(uris.iter().filter_map(|u| protos.get(u).and_then(album_item)).collect())
}

/// batch-hydrate artist ids into cards (with portraits), order kept
pub async fn artists(app: &AppHandle, ids: &[String]) -> Result<Vec<ArtistItem>, AppError> {
    let uris: Vec<String> = ids.iter().map(|id| to_uri("artist", spclient::uri_id(id))).collect();
    let raw = fetch_batch(app, ExtensionKind::ARTIST_V4, &uris, TTL).await?;
    let protos: HashMap<String, metadata::Artist> = decode_all(raw);
    Ok(uris.iter().filter_map(|u| protos.get(u).and_then(artist_item)).collect())
}

// conversions

pub fn to_uri(kind: &str, id: &str) -> String {
    if id.starts_with("spotify:") {
        id.to_string()
    } else {
        format!("spotify:{kind}:{id}")
    }
}

pub fn gid_to_id(gid: &[u8]) -> Option<String> {
    SpotifyId::from_raw(gid).ok()?.to_base62().ok()
}

/// the 640px variant when there is one; spotify's size enum is
/// SMALL(64/160) < DEFAULT(300/320) < LARGE(640) < XLARGE
pub fn best_image(group: Option<&metadata::ImageGroup>, legacy: &[metadata::Image]) -> Option<String> {
    use metadata::image::Size;
    let rank = |s: Size| match s {
        Size::LARGE => 0,
        Size::DEFAULT => 1,
        Size::XLARGE => 2,
        Size::SMALL => 3,
    };
    group
        .map(|g| g.image.as_slice())
        .unwrap_or_default()
        .iter()
        .chain(legacy.iter())
        .filter(|i| !i.file_id().is_empty())
        .min_by_key(|i| rank(i.size()))
        .and_then(|i| spclient::image_url(i.file_id()))
}

pub fn date_string(d: Option<&metadata::Date>) -> Option<String> {
    let d = d?;
    if !d.has_year() || d.year() <= 0 {
        return None;
    }
    Some(match (d.month(), d.day()) {
        (m, day) if m > 0 && day > 0 => format!("{:04}-{:02}-{:02}", d.year(), m, day),
        (m, _) if m > 0 => format!("{:04}-{:02}", d.year(), m),
        _ => format!("{:04}", d.year()),
    })
}

pub fn album_type(a: &metadata::Album) -> String {
    use metadata::album::Type;
    if a.has_type_str() && !a.type_str().is_empty() {
        let t = a.type_str().to_lowercase();
        // the web api folds EPs into "single"; keep the rest of the app's split
        return if t == "ep" { "single".into() } else { t };
    }
    match a.type_() {
        Type::SINGLE | Type::EP => "single",
        Type::COMPILATION => "compilation",
        Type::AUDIOBOOK => "audiobook",
        Type::PODCAST => "podcast",
        Type::ALBUM => "album",
    }
    .into()
}

pub fn artist_item(a: &metadata::Artist) -> Option<ArtistItem> {
    Some(ArtistItem {
        id: gid_to_id(a.gid())?,
        name: a.name().to_string(),
        image_url: best_image(a.portrait_group.as_ref(), &a.portrait),
        popularity: a.has_popularity().then(|| a.popularity() as i64),
    })
}

pub fn album_item(a: &metadata::Album) -> Option<AlbumItem> {
    Some(AlbumItem {
        id: gid_to_id(a.gid())?,
        name: a.name().to_string(),
        album_type: album_type(a),
        image_url: best_image(a.cover_group.as_ref(), &a.cover),
        release_date: date_string(a.date.as_ref()),
        artists: a.artist.iter().filter_map(artist_item).collect(),
        popularity: a.has_popularity().then(|| a.popularity() as i64),
    })
}

pub fn track_item(t: &metadata::Track) -> Option<TrackItem> {
    Some(TrackItem {
        id: gid_to_id(t.gid())?,
        name: t.name().to_string(),
        duration_ms: t.duration() as i64,
        explicit: t.explicit(),
        artists: t.artist.iter().filter_map(artist_item).collect(),
        album: t.album.as_ref().and_then(album_item),
        popularity: t.has_popularity().then(|| t.popularity() as i64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        let mut d = metadata::Date::new();
        assert_eq!(date_string(Some(&d)), None);
        d.set_year(2021);
        assert_eq!(date_string(Some(&d)).as_deref(), Some("2021"));
        d.set_month(3);
        assert_eq!(date_string(Some(&d)).as_deref(), Some("2021-03"));
        d.set_day(9);
        assert_eq!(date_string(Some(&d)).as_deref(), Some("2021-03-09"));
    }

    #[test]
    fn prefers_large_image() {
        use metadata::image::Size;
        let mut small = metadata::Image::new();
        small.set_file_id(vec![1]);
        small.set_size(Size::SMALL);
        let mut large = metadata::Image::new();
        large.set_file_id(vec![2]);
        large.set_size(Size::LARGE);
        let url = best_image(None, &[small, large]);
        assert_eq!(url.as_deref(), Some("https://i.scdn.co/image/02"));
    }

    #[test]
    fn uris() {
        assert_eq!(to_uri("track", "abc"), "spotify:track:abc");
        assert_eq!(to_uri("track", "spotify:episode:x"), "spotify:episode:x");
    }
}
