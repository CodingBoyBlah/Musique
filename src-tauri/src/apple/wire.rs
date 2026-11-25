//! Tolerant Apple Music JSON wire parsers and mappers into musique's domain models.

use std::collections::HashSet;
use std::fmt::Write as _;

use serde::de::{Deserializer, IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::Value;

use crate::commands::artist_extras::{strip_html, ArtistExtras, ArtistOverview};
use crate::commands::home_feed::{HomeItem, HomeSection};
use crate::commands::library::PlaylistSummary;
use crate::spotify::types::{
    AlbumDetail, AlbumItem, ArtistDetail, ArtistItem, PlaylistCard, PlaylistDetail, SearchResults,
    TrackDetail, TrackItem,
};

pub const ART_SIZE: u32 = 640;
pub const HERO_SIZE: u32 = 1200;
const BANNER_MAX_WIDTH: u32 = 2400;

/// Returns `true` when `id` is an Apple Music library resource ID (`i.`, `l.`, `p.`, `r.`).
pub fn is_library_id(id: &str) -> bool {
    matches!(id.as_bytes(), [b'i' | b'l' | b'p' | b'r', b'.', ..])
}

/// Fills an Apple artwork template (`.../{w}x{h}bb.{f}`) with concrete dimensions and `jpg`.
pub fn format_artwork(template: &str, width: u32, height: u32) -> Option<String> {
    let trimmed = template.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len() + 12);
    let mut rest = trimmed;
    while let Some(idx) = rest.find('{') {
        out.push_str(&rest[..idx]);
        rest = &rest[idx..];
        if let Some(rest_after) = rest.strip_prefix("{w}") {
            let _ = write!(out, "{width}");
            rest = rest_after;
        } else if let Some(rest_after) = rest.strip_prefix("{h}") {
            let _ = write!(out, "{height}");
            rest = rest_after;
        } else if let Some(rest_after) = rest.strip_prefix("{f}") {
            out.push_str("jpg");
            rest = rest_after;
        } else if let Some(rest_after) = rest.strip_prefix("{c}") {
            rest = rest_after;
        } else {
            out.push('{');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    Some(out)
}

pub fn artwork_url(attrs: &Value, size: u32) -> Option<String> {
    let tmpl = attrs.get("artwork")?.get("url")?.as_str()?;
    format_artwork(tmpl, size, size)
}

pub fn banner_url(attrs: &Value) -> Option<String> {
    let editorial = attrs.get("editorialArtwork")?;
    for key in [
        "bannerUber",
        "superHeroWide",
        "centeredFullscreenBackground",
        "subscriptionHero",
        "storeFlowcase",
    ] {
        if let Some(url) = format_editorial_entry(editorial.get(key)) {
            return Some(url);
        }
    }
    None
}

pub fn gallery_urls(attrs: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(editorial) = attrs.get("editorialArtwork").and_then(Value::as_object) {
        for key in [
            "bannerUber",
            "superHeroWide",
            "centeredFullscreenBackground",
            "subscriptionHero",
            "storeFlowcase",
            "staticDetailSquare",
            "staticDetailTall",
        ] {
            if let Some(url) = format_editorial_entry(editorial.get(key)) {
                if !out.contains(&url) {
                    out.push(url);
                }
            }
        }
        for (key, val) in editorial {
            if matches!(
                key.as_str(),
                "bannerUber"
                    | "superHeroWide"
                    | "centeredFullscreenBackground"
                    | "subscriptionHero"
                    | "storeFlowcase"
                    | "staticDetailSquare"
                    | "staticDetailTall"
            ) {
                continue;
            }
            if let Some(url) = format_editorial_entry(Some(val)) {
                if !out.contains(&url) {
                    out.push(url);
                }
            }
        }
    }
    if let Some(art) = artwork_url(attrs, HERO_SIZE) {
        if !out.contains(&art) {
            out.push(art);
        }
    }
    out
}

fn format_editorial_entry(entry: Option<&Value>) -> Option<String> {
    let obj = entry?.as_object()?;
    let tmpl = obj.get("url")?.as_str()?;
    let w = obj.get("width").and_then(Value::as_u64).unwrap_or(0);
    let h = obj.get("height").and_then(Value::as_u64).unwrap_or(0);
    if w > 0 && h > 0 {
        let clamped_w = u64::from(BANNER_MAX_WIDTH).min(w);
        let scaled_h = (h * clamped_w / w).max(1);
        format_artwork(tmpl, clamped_w as u32, scaled_h as u32)
    } else {
        format_artwork(tmpl, HERO_SIZE, HERO_SIZE)
    }
}

/// Top-level tolerant response envelope.
#[derive(Debug, Default)]
pub struct ParsedPage {
    pub data: Vec<Value>,
    pub next: Option<String>,
    pub total: Option<usize>,
    pub results: Option<Value>,
    pub resources: Option<Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawEnvelope {
    #[serde(deserialize_with = "lossy_object_array")]
    data: Vec<Value>,
    #[serde(deserialize_with = "lossy_opt_string")]
    next: Option<String>,
    #[serde(deserialize_with = "lossy_meta")]
    meta: RawMeta,
    results: Option<Value>,
    resources: Option<Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawMeta {
    #[serde(deserialize_with = "lossy_opt_usize")]
    total: Option<usize>,
}

fn lossy_object_array<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Value>, D::Error> {
    struct ArrVisitor;
    impl<'de> Visitor<'de> for ArrVisitor {
        type Value = Vec<Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("array of objects")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(elem) = seq.next_element::<Value>()? {
                if elem.is_object() {
                    out.push(elem);
                }
            }
            Ok(out)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
            Ok(Vec::new())
        }
    }
    deserializer.deserialize_any(ArrVisitor)
}

fn lossy_opt_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let val = Value::deserialize(deserializer)?;
    Ok(val
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned))
}

fn lossy_opt_usize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<usize>, D::Error> {
    let val = Value::deserialize(deserializer)?;
    Ok(val.as_u64().and_then(|n| usize::try_from(n).ok()))
}

fn lossy_meta<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RawMeta, D::Error> {
    let val = Value::deserialize(deserializer)?;
    Ok(serde_json::from_value::<RawMeta>(val).unwrap_or_default())
}

/// Parses an Apple Music JSON response tolerantly. Never fails on malformed rows or wrong field types.
pub fn parse_page(body: &str) -> ParsedPage {
    let Ok(env) = serde_json::from_str::<RawEnvelope>(body) else {
        return ParsedPage::default();
    };
    ParsedPage {
        data: env.data,
        next: env.next,
        total: env.meta.total,
        results: env.results,
        resources: env.resources,
    }
}

pub fn str_field<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub fn bool_field(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}

pub fn i64_field(v: &Value, key: &str) -> Option<i64> {
    let x = v.get(key)?;
    x.as_i64()
        .or_else(|| x.as_u64().and_then(|u| i64::try_from(u).ok()))
}

pub fn attrs_of(res: &Value) -> Option<&Value> {
    res.get("attributes").filter(|v| v.is_object())
}

pub fn rel_rows<'a>(res: &'a Value, rel: &str) -> &'a [Value] {
    res.get("relationships")
        .and_then(|r| r.get(rel))
        .and_then(|r| r.get("data"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn view_rows<'a>(res: &'a Value, view: &str) -> &'a [Value] {
    res.get("views")
        .and_then(|v| v.get(view))
        .and_then(|v| v.get("data"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn catalog_rel(res: &Value) -> Option<&Value> {
    rel_rows(res, "catalog").first().filter(|v| v.is_object())
}

pub fn album_type_from(attrs: &Value, name: &str) -> String {
    if bool_field(attrs, "isCompilation") == Some(true) {
        return "compilation".to_owned();
    }
    if bool_field(attrs, "isSingle") == Some(true)
        || name.ends_with(" - Single")
        || name.ends_with(" - EP")
    {
        return "single".to_owned();
    }
    "album".to_owned()
}

pub fn normalize_release_date(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let date_part = s.split_once('T').map(|(d, _)| d).unwrap_or(s);
    (!date_part.is_empty()).then(|| date_part.to_owned())
}

fn synthetic_artist_id(name: &str) -> String {
    format!("apple-name:{name}")
}

pub fn is_synthetic_artist_id(id: &str) -> bool {
    id.starts_with("apple-name:")
}

/// Parses artist references on a track or album, falling back to `attributes.artistName`.
pub fn parse_artist_refs(res: &Value, attrs: &Value) -> Vec<ArtistItem> {
    let rows = rel_rows(res, "artists");
    let mut out = Vec::new();
    for row in rows {
        let target = catalog_rel(row).unwrap_or(row);
        let Some(id) = str_field(target, "id").or_else(|| str_field(row, "id")) else {
            continue;
        };
        let target_attrs = attrs_of(target);
        let row_attrs = attrs_of(row);
        let name = target_attrs
            .and_then(|a| str_field(a, "name"))
            .or_else(|| row_attrs.and_then(|a| str_field(a, "name")))
            .or_else(|| {
                if rows.len() == 1 {
                    str_field(attrs, "artistName")
                } else {
                    None
                }
            });
        let Some(name) = name else { continue };
        let image_url = target_attrs
            .and_then(|a| artwork_url(a, ART_SIZE))
            .or_else(|| row_attrs.and_then(|a| artwork_url(a, ART_SIZE)));
        out.push(ArtistItem {
            id: id.to_owned(),
            name: name.to_owned(),
            image_url,
            popularity: None,
        });
    }
    if out.is_empty() {
        if let Some(name) = str_field(attrs, "artistName") {
            out.push(ArtistItem {
                id: synthetic_artist_id(name),
                name: name.to_owned(),
                image_url: None,
                popularity: None,
            });
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct ParsedTrack {
    pub item: TrackItem,
    pub isrc: Option<String>,
    pub preview_url: Option<String>,
    pub track_number: i64,
    pub disc_number: i64,
    pub date_added: Option<String>,
    pub library_id: Option<String>,
    pub personal_rating: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ParsedAlbum {
    pub item: AlbumItem,
    pub total_tracks: i64,
    pub genres: Vec<String>,
    pub date_added: Option<String>,
    pub library_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedArtist {
    pub item: ArtistItem,
    pub genres: Vec<String>,
    pub date_added: Option<String>,
    pub library_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedPlaylist {
    pub id: String,
    pub library_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub owner_name: Option<String>,
    pub owner_id: Option<String>,
    pub public: Option<bool>,
    pub collaborative: Option<bool>,
    pub can_edit: bool,
    pub total_tracks: i64,
    pub snapshot_id: Option<String>,
    pub tracks: Vec<ParsedTrack>,
}

impl ParsedPlaylist {
    pub fn into_card(self) -> PlaylistCard {
        PlaylistCard {
            id: self.id,
            name: self.name,
            description: self.description,
            image_url: self.image_url,
            owner_name: self.owner_name,
        }
    }

    pub fn into_summary(self) -> PlaylistSummary {
        PlaylistSummary {
            id: self.id,
            name: self.name,
            description: self.description,
            image_url: self.image_url,
            total_tracks: self.total_tracks,
            snapshot_id: self.snapshot_id,
        }
    }

    pub fn into_detail(self, tracks: Vec<TrackItem>, total_tracks: i64) -> PlaylistDetail {
        PlaylistDetail {
            id: self.id,
            name: self.name,
            description: self.description,
            image_url: self.image_url,
            owner_name: self.owner_name,
            total_tracks,
            tracks,
            owner_id: self.owner_id,
            public: self.public,
            collaborative: self.collaborative,
            followers: None,
        }
    }
}

/// Parses a catalog `songs` or library `library-songs` resource.
pub fn parse_track(res: &Value, mut dedup_catalog: Option<&mut HashSet<String>>) -> Option<ParsedTrack> {
    let raw_id = str_field(res, "id")?;
    let res_type = str_field(res, "type");
    let own_attrs = attrs_of(res);
    let is_lib = res_type == Some("library-songs") || is_library_id(raw_id);

    if !is_lib {
        return parse_catalog_song(res, raw_id.to_owned(), None, None);
    }

    let play_params = own_attrs
        .and_then(|a| a.get("playParams"))
        .filter(|v| v.is_object());
    let own_streams = play_params.is_some();
    let cat = catalog_rel(res);
    let cat_id = play_params
        .and_then(|p| str_field(p, "catalogId"))
        .or_else(|| cat.and_then(|c| str_field(c, "id")));
    let reporting_id = play_params.and_then(|p| str_field(p, "reportingId"));

    let cat_streams = match cat {
        Some(c) => match attrs_of(c) {
            Some(ca) => ca.get("playParams").is_some_and(Value::is_object),
            None => true,
        },
        None => true,
    };

    let mut chosen_id = match (cat_id, cat_streams) {
        (Some(cid), true) => cid.to_owned(),
        (Some(cid), false) => match reporting_id.filter(|rep| *rep != cid) {
            Some(rep) => rep.to_owned(),
            None if own_streams => raw_id.to_owned(),
            None => cid.to_owned(),
        },
        (None, _) => raw_id.to_owned(),
    };

    if let Some(seen) = dedup_catalog.as_deref_mut() {
        if chosen_id != raw_id && own_streams && !seen.insert(chosen_id.clone()) {
            chosen_id = raw_id.to_owned();
        }
    }

    let date_added = own_attrs
        .and_then(|a| str_field(a, "dateAdded"))
        .map(str::to_owned);
    let personal_rating = own_attrs.and_then(|a| i64_field(a, "personalRating"));

    let mut parsed = if let Some(cat_res) = cat.filter(|c| attrs_of(c).is_some()) {
        if let Some(mut track) = parse_catalog_song(cat_res, chosen_id.clone(), Some(raw_id), own_attrs) {
            // Preserve library-specific overrides if present.
            if let Some(own) = own_attrs {
                if let Some(name) = str_field(own, "name") {
                    track.item.name = name.to_owned();
                }
                if let Some(art) = artwork_url(own, ART_SIZE) {
                    track.item.image_url_override(&art);
                }
                if let Some(tn) = i64_field(own, "trackNumber") {
                    track.track_number = tn;
                }
                if let Some(dn) = i64_field(own, "discNumber") {
                    track.disc_number = dn;
                }
                if track.item.artists.is_empty() {
                    track.item.artists = parse_artist_refs(res, own);
                }
                let lib_album = rel_rows(res, "albums").first().filter(|v| v.is_object());
                if let Some(lib_alb) = lib_album {
                    let lib_alb_id = str_field(lib_alb, "id");
                    let own_alb_name = str_field(own, "albumName");
                    let cat_alb_name = track.item.album.as_ref().map(|a| a.name.as_str());
                    if let Some(lid) = lib_alb_id {
                        if track.item.album.is_none()
                            || own_alb_name.is_some_and(|n| cat_alb_name != Some(n))
                        {
                            track.item.album = Some(AlbumItem {
                                id: lid.to_owned(),
                                name: own_alb_name.unwrap_or_default().to_owned(),
                                album_type: album_type_from(own, own_alb_name.unwrap_or_default()),
                                image_url: artwork_url(own, ART_SIZE),
                                release_date: normalize_release_date(str_field(own, "releaseDate")),
                                artists: track.item.artists.clone(),
                                popularity: None,
                            });
                        }
                    }
                }
            }
            track
        } else {
            parse_catalog_song(res, chosen_id, Some(raw_id), None)?
        }
    } else {
        parse_catalog_song(res, chosen_id, Some(raw_id), None)?
    };

    parsed.date_added = date_added.or(parsed.date_added);
    parsed.library_id = Some(raw_id.to_owned());
    parsed.personal_rating = personal_rating.or(parsed.personal_rating);
    Some(parsed)
}

trait TrackAlbumArtExt {
    fn image_url_override(&mut self, url: &str);
}

impl TrackAlbumArtExt for TrackItem {
    fn image_url_override(&mut self, url: &str) {
        if let Some(ref mut alb) = self.album {
            if alb.image_url.is_none() {
                alb.image_url = Some(url.to_owned());
            }
        }
    }
}

fn parse_catalog_song(
    res: &Value,
    id: String,
    library_id: Option<&str>,
    fallback_attrs: Option<&Value>,
) -> Option<ParsedTrack> {
    let attrs = attrs_of(res).or(fallback_attrs)?;
    let name = str_field(attrs, "name")
        .or_else(|| fallback_attrs.and_then(|a| str_field(a, "name")))?;
    let duration_ms = i64_field(attrs, "durationInMillis")
        .or_else(|| fallback_attrs.and_then(|a| i64_field(a, "durationInMillis")))
        .unwrap_or(0);
    let explicit = str_field(attrs, "contentRating")
        .is_some_and(|r| r.eq_ignore_ascii_case("explicit"));
    let isrc = str_field(attrs, "isrc").map(str::to_owned);
    let preview_url = attrs
        .get("previews")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|p| str_field(p, "url"))
        .map(str::to_owned);
    let track_number = i64_field(attrs, "trackNumber").unwrap_or(0);
    let disc_number = i64_field(attrs, "discNumber").unwrap_or(1);
    let date_added = str_field(attrs, "dateAdded").map(str::to_owned);
    let personal_rating = i64_field(attrs, "personalRating");
    let artists = parse_artist_refs(res, attrs);

    let album = rel_rows(res, "albums")
        .first()
        .filter(|a| a.is_object())
        .and_then(|alb_res| {
            let alb_id = str_field(alb_res, "id")?;
            let alb_attrs = attrs_of(alb_res);
            let alb_name = alb_attrs
                .and_then(|a| str_field(a, "name"))
                .or_else(|| str_field(attrs, "albumName"))
                .unwrap_or_default()
                .to_owned();
            let alb_type = alb_attrs
                .map(|a| album_type_from(a, &alb_name))
                .unwrap_or_else(|| album_type_from(attrs, &alb_name));
            let alb_img = alb_attrs
                .and_then(|a| artwork_url(a, ART_SIZE))
                .or_else(|| artwork_url(attrs, ART_SIZE));
            let release_date = alb_attrs
                .and_then(|a| normalize_release_date(str_field(a, "releaseDate")))
                .or_else(|| normalize_release_date(str_field(attrs, "releaseDate")));
            let alb_artists = alb_attrs
                .map(|a| parse_artist_refs(alb_res, a))
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| artists.clone());
            Some(AlbumItem {
                id: alb_id.to_owned(),
                name: alb_name,
                album_type: alb_type,
                image_url: alb_img,
                release_date,
                artists: alb_artists,
                popularity: None,
            })
        })
        // no album relationship came back (stations and some shelves omit it): the song's
        // own fields still name its album, and its artwork is the album's
        .or_else(|| {
            let alb_name = str_field(attrs, "albumName")?;
            let alb_id = str_field(attrs, "url").and_then(album_id_from_url)?;
            Some(AlbumItem {
                id: alb_id,
                name: alb_name.to_owned(),
                album_type: album_type_from(attrs, alb_name),
                image_url: artwork_url(attrs, ART_SIZE),
                release_date: normalize_release_date(str_field(attrs, "releaseDate")),
                artists: artists.clone(),
                popularity: None,
            })
        });

    Some(ParsedTrack {
        item: TrackItem {
            id,
            name: name.to_owned(),
            duration_ms,
            explicit,
            artists,
            album,
            popularity: None,
        },
        isrc,
        preview_url,
        track_number,
        disc_number,
        date_added,
        library_id: library_id.map(str::to_owned),
        personal_rating,
    })
}

/// The catalog album id in a song's web url: `https://music.apple.com/us/album/<slug>/<id>?i=<song>`.
fn album_id_from_url(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next()?;
    let mut parts = path.split('/').filter(|p| !p.is_empty());
    parts.position(|p| p == "album")?;
    let id = parts.last()?;
    id.chars().all(|c| c.is_ascii_digit()).then(|| id.to_owned())
}

pub fn parse_track_detail(res: &Value) -> Option<TrackDetail> {
    let parsed = parse_track(res, None)?;
    Some(TrackDetail {
        id: parsed.item.id,
        name: parsed.item.name,
        duration_ms: parsed.item.duration_ms,
        explicit: parsed.item.explicit,
        popularity: parsed.item.popularity,
        preview_url: parsed.preview_url,
        artists: parsed.item.artists,
        album: parsed.item.album,
    })
}

pub fn parse_track_isrc(res: &Value) -> Option<String> {
    if let Some(isrc) = attrs_of(res).and_then(|a| str_field(a, "isrc")) {
        return Some(isrc.to_owned());
    }
    catalog_rel(res)
        .and_then(attrs_of)
        .and_then(|a| str_field(a, "isrc"))
        .map(str::to_owned)
}

/// Parses a catalog `albums` or library `library-albums` resource.
pub fn parse_album(res: &Value) -> Option<ParsedAlbum> {
    let raw_id = str_field(res, "id")?;
    let res_type = str_field(res, "type");
    let own_attrs = attrs_of(res);
    let cat = catalog_rel(res);

    let is_lib = res_type == Some("library-albums") || is_library_id(raw_id);
    if is_lib {
        let empty_lib = own_attrs.is_some_and(|a| i64_field(a, "trackCount") == Some(0));
        if empty_lib && cat.is_none() {
            return None;
        }
    }

    let primary = cat.filter(|c| attrs_of(c).is_some()).unwrap_or(res);
    let primary_attrs = attrs_of(primary).or(own_attrs)?;
    let id = cat
        .and_then(|c| str_field(c, "id"))
        .unwrap_or(raw_id)
        .to_owned();
    let name = str_field(primary_attrs, "name")
        .or_else(|| own_attrs.and_then(|a| str_field(a, "name")))?
        .to_owned();

    let album_type = album_type_from(primary_attrs, &name);
    let image_url = artwork_url(primary_attrs, ART_SIZE)
        .or_else(|| own_attrs.and_then(|a| artwork_url(a, ART_SIZE)));
    let release_date = normalize_release_date(str_field(primary_attrs, "releaseDate"))
        .or_else(|| own_attrs.and_then(|a| normalize_release_date(str_field(a, "releaseDate"))));
    let total_tracks = i64_field(primary_attrs, "trackCount")
        .or_else(|| own_attrs.and_then(|a| i64_field(a, "trackCount")))
        .unwrap_or(0);
    let date_added = own_attrs
        .and_then(|a| str_field(a, "dateAdded"))
        .map(str::to_owned);
    let genres = parse_genres(primary_attrs);

    let mut artists = parse_artist_refs(primary, primary_attrs);
    if artists.is_empty() {
        if let Some(oa) = own_attrs {
            artists = parse_artist_refs(res, oa);
        }
    }

    Some(ParsedAlbum {
        item: AlbumItem {
            id,
            name,
            album_type,
            image_url,
            release_date,
            artists,
            popularity: None,
        },
        total_tracks,
        genres,
        date_added,
        library_id: is_lib.then(|| raw_id.to_owned()),
    })
}

pub fn parse_album_detail(res: &Value) -> Option<AlbumDetail> {
    let parsed = parse_album(res)?;
    let attrs = catalog_rel(res).and_then(attrs_of).or_else(|| attrs_of(res));
    let image_url = attrs
        .and_then(|a| artwork_url(a, HERO_SIZE))
        .or_else(|| parsed.item.image_url.clone());

    let album_stub = AlbumItem {
        image_url: image_url.clone(),
        ..parsed.item.clone()
    };

    let mut tracks: Vec<TrackItem> = rel_rows(res, "tracks")
        .iter()
        .filter_map(|row| parse_track(row, None))
        .map(|mut t| {
            if t.item.album.is_none() {
                t.item.album = Some(album_stub.clone());
            }
            t.item
        })
        .collect();

    if tracks.is_empty() {
        if let Some(cat) = catalog_rel(res) {
            tracks = rel_rows(cat, "tracks")
                .iter()
                .filter_map(|row| parse_track(row, None))
                .map(|mut t| {
                    if t.item.album.is_none() {
                        t.item.album = Some(album_stub.clone());
                    }
                    t.item
                })
                .collect();
        }
    }

    let total_tracks = if parsed.total_tracks > 0 {
        parsed.total_tracks
    } else {
        tracks.len() as i64
    };

    Some(AlbumDetail {
        id: parsed.item.id,
        name: parsed.item.name,
        album_type: parsed.item.album_type,
        image_url,
        release_date: parsed.item.release_date,
        total_tracks,
        popularity: None,
        artists: parsed.item.artists,
        tracks,
    })
}

/// Parses a catalog `artists` or library `library-artists` resource.
pub fn parse_artist(res: &Value) -> Option<ParsedArtist> {
    let raw_id = str_field(res, "id")?;
    let res_type = str_field(res, "type");
    let own_attrs = attrs_of(res);
    let cat = catalog_rel(res);
    let is_lib = res_type == Some("library-artists") || is_library_id(raw_id);

    let primary = cat.filter(|c| attrs_of(c).is_some()).unwrap_or(res);
    let primary_attrs = attrs_of(primary).or(own_attrs)?;
    let id = cat
        .and_then(|c| str_field(c, "id"))
        .unwrap_or(raw_id)
        .to_owned();
    let name = str_field(primary_attrs, "name")
        .or_else(|| own_attrs.and_then(|a| str_field(a, "name")))?
        .to_owned();
    let image_url = artwork_url(primary_attrs, ART_SIZE)
        .or_else(|| own_attrs.and_then(|a| artwork_url(a, ART_SIZE)));
    let genres = parse_genres(primary_attrs);
    let date_added = own_attrs
        .and_then(|a| str_field(a, "dateAdded"))
        .map(str::to_owned);

    Some(ParsedArtist {
        item: ArtistItem {
            id,
            name,
            image_url,
            popularity: None,
        },
        genres,
        date_added,
        library_id: is_lib.then(|| raw_id.to_owned()),
    })
}

pub fn parse_artist_detail(res: &Value) -> Option<ArtistDetail> {
    let parsed = parse_artist(res)?;
    let attrs = attrs_of(res);
    let image_url = attrs
        .and_then(|a| artwork_url(a, HERO_SIZE))
        .or_else(|| parsed.item.image_url.clone());

    let mut albums = Vec::new();
    let mut singles = Vec::new();
    let mut seen_albums = HashSet::new();

    for (view_name, default_type) in [
        ("full-albums", "album"),
        ("live-albums", "album"),
        ("compilation-albums", "compilation"),
        ("singles", "single"),
    ] {
        for row in view_rows(res, view_name) {
            let Some(mut alb) = parse_album(row).map(|p| p.item) else {
                continue;
            };
            if default_type == "compilation" && alb.album_type == "album" {
                alb.album_type = "compilation".to_owned();
            } else if default_type == "single" && alb.album_type == "album" {
                alb.album_type = "single".to_owned();
            }
            if !seen_albums.insert(alb.id.clone()) {
                continue;
            }
            if view_name == "singles" || alb.album_type == "single" {
                singles.push(alb);
            } else {
                albums.push(alb);
            }
        }
    }

    // Library artist fallback when no catalog views exist.
    if albums.is_empty() && singles.is_empty() {
        for row in rel_rows(res, "albums") {
            if let Some(alb) = parse_album(row).map(|p| p.item) {
                if seen_albums.insert(alb.id.clone()) {
                    if alb.album_type == "single" {
                        singles.push(alb);
                    } else {
                        albums.push(alb);
                    }
                }
            }
        }
    }

    sort_albums_newest_first(&mut albums);
    sort_albums_newest_first(&mut singles);

    let mut top_tracks: Vec<TrackItem> = view_rows(res, "top-songs")
        .iter()
        .filter_map(|row| parse_track(row, None).map(|t| t.item))
        .collect();

    if top_tracks.is_empty() {
        for alb in rel_rows(res, "albums") {
            for row in rel_rows(alb, "tracks") {
                if let Some(t) = parse_track(row, None).map(|t| t.item) {
                    top_tracks.push(t);
                }
            }
        }
    }

    Some(ArtistDetail {
        id: parsed.item.id,
        name: parsed.item.name,
        image_url,
        genres: parsed.genres,
        popularity: None,
        albums,
        singles,
        top_tracks,
    })
}

pub fn parse_artist_extras(res: &Value) -> ArtistExtras {
    let Some(attrs) = attrs_of(res) else {
        return ArtistExtras::default();
    };
    let notes = attrs.get("editorialNotes");
    let raw_bio = str_field(attrs, "artistBio")
        .or_else(|| notes.and_then(|n| str_field(n, "standard")))
        .or_else(|| notes.and_then(|n| str_field(n, "short")))
        .or_else(|| notes.and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()));
    let biography = raw_bio.map(strip_html).filter(|s| !s.is_empty());

    let gallery = gallery_urls(attrs);
    let related = view_rows(res, "similar-artists")
        .iter()
        .filter_map(|row| parse_artist(row).map(|a| a.item))
        .collect();
    let appears_on = view_rows(res, "appears-on-albums")
        .iter()
        .filter_map(|row| parse_album(row).map(|a| a.item))
        .collect();
    let compilations = view_rows(res, "compilation-albums")
        .iter()
        .filter_map(|row| {
            let mut a = parse_album(row)?.item;
            if a.album_type == "album" {
                a.album_type = "compilation".to_owned();
            }
            Some(a)
        })
        .collect();

    ArtistExtras {
        biography,
        gallery,
        active_years: None,
        related,
        appears_on,
        compilations,
    }
}

pub fn parse_artist_overview(res: &Value) -> ArtistOverview {
    let Some(attrs) = attrs_of(res) else {
        return ArtistOverview::default();
    };
    ArtistOverview {
        header_image: banner_url(attrs),
        gallery: gallery_urls(attrs),
        ..ArtistOverview::default()
    }
}

pub fn parse_playlist(res: &Value) -> Option<ParsedPlaylist> {
    let raw_id = str_field(res, "id")?;
    let res_type = str_field(res, "type");
    let own_attrs = attrs_of(res);
    let cat = catalog_rel(res);
    let is_lib = res_type == Some("library-playlists") || is_library_id(raw_id);
    let can_edit = own_attrs
        .and_then(|a| bool_field(a, "canEdit"))
        .unwrap_or(false);

    // Editable library playlists must keep their `p.` id so PATCH/POST/DELETE mutations target them.
    let use_catalog = is_lib && !can_edit && cat.and_then(|c| str_field(c, "id")).is_some();
    let primary = if use_catalog { cat? } else { res };
    let primary_attrs = attrs_of(primary).or(own_attrs)?;

    let id = if use_catalog {
        str_field(primary, "id")?.to_owned()
    } else {
        raw_id.to_owned()
    };
    let name = str_field(primary_attrs, "name")
        .or_else(|| own_attrs.and_then(|a| str_field(a, "name")))?
        .to_owned();

    let description = extract_description(primary_attrs)
        .or_else(|| own_attrs.and_then(extract_description));
    let image_url = artwork_url(primary_attrs, ART_SIZE)
        .or_else(|| own_attrs.and_then(|a| artwork_url(a, ART_SIZE)));

    let is_catalog_playlist = !is_lib || use_catalog;
    let (owner_name, owner_id, public, collaborative) = if is_catalog_playlist {
        let curator = str_field(primary_attrs, "curatorName")
            .unwrap_or("Apple Music")
            .to_owned();
        (Some(curator.clone()), Some(curator), Some(true), Some(false))
    } else {
        let is_public = own_attrs.and_then(|a| bool_field(a, "isPublic"));
        let is_collab = own_attrs.and_then(|a| bool_field(a, "hasCollaboration"));
        (
            Some("You".to_owned()),
            Some("library".to_owned()),
            is_public,
            is_collab,
        )
    };

    let snapshot_id = own_attrs
        .and_then(|a| str_field(a, "lastModifiedDate").or_else(|| str_field(a, "dateAdded")))
        .or_else(|| str_field(primary_attrs, "lastModifiedDate"))
        .map(str::to_owned);

    let tracks: Vec<ParsedTrack> = rel_rows(res, "tracks")
        .iter()
        .filter_map(|row| parse_track(row, None))
        .collect();
    let total_tracks = i64_field(primary_attrs, "trackCount")
        .or_else(|| own_attrs.and_then(|a| i64_field(a, "trackCount")))
        .unwrap_or(tracks.len() as i64);

    Some(ParsedPlaylist {
        id,
        library_id: is_lib.then(|| raw_id.to_owned()),
        name,
        description,
        image_url,
        owner_name,
        owner_id,
        public,
        collaborative,
        can_edit,
        total_tracks,
        snapshot_id,
        tracks,
    })
}

fn extract_description(attrs: &Value) -> Option<String> {
    let desc = attrs.get("description")?;
    let raw = str_field(desc, "standard")
        .or_else(|| str_field(desc, "short"))
        .or_else(|| desc.as_str().map(str::trim).filter(|s| !s.is_empty()))?;
    let clean = strip_html(raw);
    (!clean.is_empty()).then_some(clean)
}

pub fn parse_search_results(body: &str) -> SearchResults {
    let page = parse_page(body);
    let Some(results) = page.results.as_ref() else {
        return empty_search_results();
    };
    let result_rows = |key: &str| -> &[Value] {
        results
            .get(key)
            .and_then(|r| r.get("data"))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    };

    SearchResults {
        tracks: result_rows("songs")
            .iter()
            .filter_map(|r| parse_track(r, None).map(|t| t.item))
            .collect(),
        artists: result_rows("artists")
            .iter()
            .filter_map(|r| parse_artist(r).map(|a| a.item))
            .collect(),
        albums: result_rows("albums")
            .iter()
            .filter_map(|r| parse_album(r).map(|a| a.item))
            .collect(),
        playlists: result_rows("playlists")
            .iter()
            .filter_map(|r| parse_playlist(r).map(ParsedPlaylist::into_card))
            .collect(),
        shows: Vec::new(),
        episodes: Vec::new(),
    }
}

fn empty_search_results() -> SearchResults {
    SearchResults {
        tracks: Vec::new(),
        artists: Vec::new(),
        albums: Vec::new(),
        playlists: Vec::new(),
        shows: Vec::new(),
        episodes: Vec::new(),
    }
}

pub fn parse_home_item(res: &Value) -> Option<HomeItem> {
    let res_type = str_field(res, "type")?;
    match res_type {
        "playlists" | "library-playlists" => {
            let pl = parse_playlist(res)?;
            Some(HomeItem {
                kind: "playlist".to_owned(),
                id: pl.id,
                name: pl.name,
                subtitle: pl.description.or(pl.owner_name),
                image_url: pl.image_url,
                show_id: None,
            })
        }
        "albums" | "library-albums" => {
            let alb = parse_album(res)?.item;
            let subtitle = (!alb.artists.is_empty()).then(|| {
                alb.artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            });
            Some(HomeItem {
                kind: "album".to_owned(),
                id: alb.id,
                name: alb.name,
                subtitle,
                image_url: alb.image_url,
                show_id: None,
            })
        }
        "artists" | "library-artists" => {
            let art = parse_artist(res)?.item;
            Some(HomeItem {
                kind: "artist".to_owned(),
                id: art.id,
                name: art.name,
                subtitle: Some("Artist".to_owned()),
                image_url: art.image_url,
                show_id: None,
            })
        }
        "songs" | "library-songs" => {
            let tr = parse_track(res, None)?.item;
            let subtitle = (!tr.artists.is_empty()).then(|| {
                tr.artists
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            });
            let image_url = tr.album.and_then(|a| a.image_url);
            Some(HomeItem {
                kind: "track".to_owned(),
                id: tr.id,
                name: tr.name,
                subtitle,
                image_url,
                show_id: None,
            })
        }
        _ => None,
    }
}

/// Parses `/me/recommendations` into `(sections, fallback_recent_items)`.
pub fn parse_recommendations(body: &str) -> (Vec<HomeSection>, Vec<HomeItem>) {
    let page = parse_page(body);
    let mut sections = Vec::new();
    let mut recents = Vec::new();

    for group in &page.data {
        let attrs = attrs_of(group);
        let is_recent = attrs
            .and_then(|a| str_field(a, "kind"))
            .is_some_and(|k| k == "recently-played");
        let title = attrs
            .and_then(|a| a.get("title"))
            .and_then(|t| str_field(t, "stringForDisplay"))
            .unwrap_or("Made for You")
            .to_owned();
        let id = str_field(group, "id")
            .unwrap_or(title.as_str())
            .to_owned();
        let items: Vec<HomeItem> = rel_rows(group, "contents")
            .iter()
            .filter_map(parse_home_item)
            .collect();
        if items.is_empty() {
            continue;
        }
        if is_recent && recents.is_empty() {
            recents = items;
        } else {
            sections.push(HomeSection { id, title, items });
        }
    }

    (sections, recents)
}

pub fn parse_charts_sections(body: &str) -> Vec<HomeSection> {
    let page = parse_page(body);
    let Some(results) = page.results.as_ref() else {
        return Vec::new();
    };

    let mut sections = Vec::new();
    for (key, default_title) in [
        ("playlists", "Top Playlists"),
        ("albums", "Top Albums"),
        ("songs", "Top Songs"),
    ] {
        let Some(charts) = results.get(key).and_then(Value::as_array) else {
            continue;
        };
        let mut items = Vec::new();
        let mut title = default_title.to_owned();
        for chart in charts {
            if let Some(name) = str_field(chart, "name") {
                if charts.len() == 1 {
                    title = name.to_owned();
                }
            }
            if let Some(rows) = chart.get("data").and_then(Value::as_array) {
                items.extend(rows.iter().filter_map(parse_home_item));
            }
        }
        if !items.is_empty() {
            sections.push(HomeSection {
                id: format!("apple-charts-{key}"),
                title,
                items,
            });
        }
    }
    sections
}

pub fn parse_charts_albums(body: &str) -> Vec<AlbumItem> {
    let page = parse_page(body);
    let Some(results) = page.results.as_ref() else {
        return Vec::new();
    };
    let Some(charts) = results.get("albums").and_then(Value::as_array) else {
        return Vec::new();
    };
    charts
        .iter()
        .filter_map(|c| c.get("data").and_then(Value::as_array))
        .flatten()
        .filter_map(|row| parse_album(row).map(|a| a.item))
        .collect()
}

pub fn parse_station_seed(body: &str) -> Option<(String, Option<String>, Vec<TrackItem>)> {
    let page = parse_page(body);
    let results = page.results.as_ref()?;
    let station = results.get("station")?;
    let station_id = str_field(station, "id")?.to_owned();
    let title = attrs_of(station)
        .and_then(|a| str_field(a, "name"))
        .map(str::to_owned);
    let tracks = results
        .get("tracks")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|row| parse_track(row, None).map(|t| t.item))
                .collect()
        })
        .unwrap_or_default();
    Some((station_id, title, tracks))
}

fn parse_genres(attrs: &Value) -> Vec<String> {
    let raw: Vec<String> = attrs
        .get("genreNames")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|g| !g.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let filtered: Vec<String> = raw
        .iter()
        .filter(|g| *g != "Music")
        .cloned()
        .collect();
    if filtered.is_empty() {
        raw
    } else {
        filtered
    }
}

fn sort_albums_newest_first(albums: &mut [AlbumItem]) {
    albums.sort_by(|a, b| b.release_date.cmp(&a.release_date));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("apple")
            .join("testdata")
            .join(name);
        std::fs::read_to_string(path).expect("fixture must exist")
    }

    #[test]
    fn a_song_without_an_album_relationship_keeps_its_album() {
        let song = serde_json::json!({
            "id": "697195787", "type": "songs",
            "attributes": {
                "name": "One More Time", "albumName": "Discovery", "durationInMillis": 320357,
                "artistName": "Daft Punk",
                "url": "https://music.apple.com/us/album/one-more-time/697194953?i=697195787",
                "artwork": { "url": "https://is1-ssl.mzstatic.com/image/thumb/x/{w}x{h}bb.jpg" },
                "playParams": { "id": "697195787", "kind": "song" }
            }
        });
        let track = parse_track(&song, None).expect("song");
        let album = track.item.album.expect("album from the song's own fields");
        assert_eq!(album.id, "697194953");
        assert_eq!(album.name, "Discovery");
        assert!(album.image_url.is_some_and(|u| !u.contains("{w}")));
        assert_eq!(album_id_from_url("https://music.apple.com/us/album/x"), None);
    }

    #[test]
    fn distinguishes_library_and_catalog_ids() {
        assert!(is_library_id("i.AWPV19dTN2g507K"));
        assert!(is_library_id("l.xBnV5i4"));
        assert!(is_library_id("r.CrpxQdf"));
        assert!(is_library_id("p.XMrmpaOfO8bWAm4"));
        assert!(!is_library_id("1440857781"));
        assert!(!is_library_id("pl.u-gxblkm0TbL9xYjk"));
        assert!(!is_library_id("pl.pm-123"));
        assert!(!is_library_id(""));
        assert!(!is_library_id("i"));
    }

    #[test]
    fn parses_catalog_song_fixture_with_isrc_and_artwork() {
        let page = parse_page(&fixture("catalog_song.json"));
        assert_eq!(page.data.len(), 1);
        let track = parse_track(&page.data[0], None).unwrap();
        assert_eq!(track.item.id, "1440857781");
        assert_eq!(track.item.name, "Blue Monday");
        assert_eq!(track.item.duration_ms, 447_000);
        assert!(track.item.explicit);
        assert_eq!(track.isrc.as_deref(), Some("GBAAP8300001"));
        assert_eq!(
            track.preview_url.as_deref(),
            Some("https://audio-ssl.itunes.apple.com/preview/blue-monday.m4a")
        );
        assert_eq!(track.item.artists[0].id, "555");
        assert_eq!(
            track.item.artists[0].image_url.as_deref(),
            Some("https://is1-ssl.mzstatic.com/image/thumb/Artist/640x640bb.jpg")
        );
        let alb = track.item.album.unwrap();
        assert_eq!(alb.id, "617154241");
        assert_eq!(alb.album_type, "compilation");
        assert_eq!(alb.release_date.as_deref(), Some("1987-08-17"));
        assert_eq!(
            alb.image_url.as_deref(),
            Some("https://is1-ssl.mzstatic.com/image/thumb/Album/640x640bb.jpg")
        );
    }

    #[test]
    fn parses_library_songs_and_skips_corrupt_rows() {
        let page = parse_page(&fixture("library_songs.json"));
        assert_eq!(page.total, Some(4));
        let mut seen = HashSet::new();
        let tracks: Vec<ParsedTrack> = page
            .data
            .iter()
            .filter_map(|row| parse_track(row, Some(&mut seen)))
            .collect();
        assert_eq!(tracks.len(), 3, "corrupt 4th row must be skipped");
        assert_eq!(tracks[0].item.id, "1440857781");
        assert_eq!(tracks[0].library_id.as_deref(), Some("i.matched1"));
        assert_eq!(tracks[0].isrc.as_deref(), Some("GBAAP8300001"));
        assert_eq!(tracks[1].item.id, "i.upload1");
        assert_eq!(tracks[1].item.artists[0].name, "Local Band");
        assert_eq!(tracks[2].item.id, "900002", "reportingId replaces pulled catalogId");
    }

    #[test]
    fn parses_catalog_and_library_albums() {
        let cat_page = parse_page(&fixture("catalog_album.json"));
        let detail = parse_album_detail(&cat_page.data[0]).unwrap();
        assert_eq!(detail.id, "1691419979");
        assert_eq!(detail.album_type, "single", "name suffix - Single detected");
        assert_eq!(detail.release_date.as_deref(), Some("2024-05-17"));
        assert_eq!(detail.total_tracks, 2);
        assert_eq!(detail.tracks.len(), 2);
        assert_eq!(
            detail.tracks[1].album.as_ref().map(|a| a.id.as_str()),
            Some("1691419979"),
            "parent album stamped onto tracks lacking own album rel"
        );

        let lib_page = parse_page(&fixture("library_albums.json"));
        let albums: Vec<ParsedAlbum> = lib_page.data.iter().filter_map(parse_album).collect();
        assert_eq!(albums.len(), 2, "ghost 0-track album must be skipped");
        assert_eq!(albums[0].item.id, "617154241");
        assert_eq!(albums[0].item.album_type, "compilation");
        assert_eq!(albums[0].date_added.as_deref(), Some("2025-10-14"));
        assert_eq!(albums[1].item.id, "l.only1");
        assert_eq!(albums[1].item.album_type, "album");
        assert_eq!(albums[1].item.release_date.as_deref(), Some("2019"));
    }

    #[test]
    fn parses_artist_detail_extras_and_overview() {
        let page = parse_page(&fixture("catalog_artist.json"));
        let row = &page.data[0];

        let detail = parse_artist_detail(row).unwrap();
        assert_eq!(detail.id, "5468295");
        assert_eq!(detail.name, "Daft Punk");
        assert_eq!(detail.genres, vec!["Electronic"]);
        assert_eq!(detail.top_tracks.len(), 1);
        assert_eq!(detail.singles.len(), 1);
        assert_eq!(detail.albums.len(), 3);
        assert_eq!(detail.albums[0].id, "2004", "newest album first (2007 > 2006 > 2001)");

        let extras = parse_artist_extras(row);
        assert_eq!(
            extras.biography.as_deref(),
            Some("French electronic duo formed in Paris.")
        );
        assert_eq!(extras.related.len(), 1);
        assert_eq!(extras.related[0].id, "5468296");
        assert_eq!(extras.appears_on.len(), 1);
        assert_eq!(extras.compilations.len(), 1);
        assert!(!extras.gallery.is_empty());

        let overview = parse_artist_overview(row);
        assert_eq!(
            overview.header_image.as_deref(),
            Some("https://is1-ssl.mzstatic.com/banner/2400x600.jpg")
        );
    }

    #[test]
    fn parses_catalog_and_library_playlists() {
        let cat = parse_page(&fixture("catalog_playlist.json"));
        let pl = parse_playlist(&cat.data[0]).unwrap();
        assert_eq!(pl.id, "pl.f4d106fed2bd41149aaacabb233eb5eb");
        assert_eq!(pl.owner_name.as_deref(), Some("Apple Music Hits"));
        assert_eq!(pl.public, Some(true));

        let lib = parse_page(&fixture("library_playlists.json"));
        let pls: Vec<ParsedPlaylist> = lib.data.iter().filter_map(parse_playlist).collect();
        assert_eq!(pls.len(), 2);
        assert_eq!(pls[0].id, "p.own1");
        assert_eq!(pls[0].owner_name.as_deref(), Some("You"));
        assert_eq!(pls[0].owner_id.as_deref(), Some("library"));
        assert_eq!(pls[0].public, Some(false));
        assert_eq!(pls[1].id, "pl.f4d106fed2bd41149aaacabb233eb5eb");
        assert_eq!(pls[1].library_id.as_deref(), Some("p.followed1"));
    }

    #[test]
    fn parses_search_recommendations_charts_and_station() {
        let search = parse_search_results(&fixture("search.json"));
        assert_eq!(search.tracks.len(), 1);
        assert_eq!(search.albums.len(), 1);
        assert_eq!(search.artists.len(), 1);
        assert_eq!(search.playlists.len(), 1);
        assert!(search.shows.is_empty());
        assert!(search.episodes.is_empty());

        let (sections, recents) = parse_recommendations(&fixture("recommendations.json"));
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Made for You");
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].kind, "album");

        let chart_sections = parse_charts_sections(&fixture("charts.json"));
        assert_eq!(chart_sections.len(), 3);
        let chart_albums = parse_charts_albums(&fixture("charts.json"));
        assert_eq!(chart_albums.len(), 1);
        assert_eq!(chart_albums[0].id, "9991");

        let (station_id, title, tracks) = parse_station_seed(&fixture("station.json")).unwrap();
        assert_eq!(station_id, "ra.1440857781");
        assert_eq!(title.as_deref(), Some("Blue Monday Station"));
        assert_eq!(tracks.len(), 2);
    }

    #[test]
    fn handles_garbage_and_wrong_shapes_without_panicking() {
        let page = parse_page(r#"{"data": ["bad", 123, null, {"id": "ok"}], "meta": "bad", "next": 99}"#);
        assert_eq!(page.data.len(), 1);
        assert_eq!(page.total, None);
        assert_eq!(page.next, None);
        assert!(parse_track(&page.data[0], None).is_none());
        assert_eq!(parse_page("not json at all").data.len(), 0);
    }
}
