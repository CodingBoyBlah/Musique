//! ipc surface for the internal (spclient / pathfinder) data layer. the heavy
//! lifting lives in `crate::internal`; these just validate and forward.

use tauri::AppHandle;

use crate::{errors::AppError, spotify::types::TrackItem};

/// hydrate bare track ids into full rows via extended-metadata. order is kept,
/// unknown ids are dropped.
#[tauri::command]
pub async fn get_tracks_metadata(app: AppHandle, ids: Vec<String>) -> Result<Vec<TrackItem>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    crate::internal::metadata::tracks(&app, &ids).await
}

// ── canvas ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Canvas {
    /// mp4 (or jpg for image canvases) on spotify's cdn
    pub url:           String,
    /// "video" | "image" | "gif"
    pub kind:          String,
    pub artist_name:   Option<String>,
    pub artist_avatar: Option<String>,
}

fn canvas_from(msg: &librespot_protocol::canvaz::entity_canvaz_response::Canvaz) -> Option<Canvas> {
    use librespot_protocol::canvaz::Type;
    if msg.url.is_empty() {
        return None;
    }
    let kind = match msg.type_.enum_value_or(Type::IMAGE) {
        Type::IMAGE => "image",
        Type::GIF => "gif",
        _ => "video",
    };
    let artist = msg.artist.as_ref();
    Some(Canvas {
        url: msg.url.clone(),
        kind: kind.into(),
        artist_name: artist.map(|a| a.name.clone()).filter(|n| !n.is_empty()),
        artist_avatar: artist.map(|a| a.avatar.clone()).filter(|n| !n.is_empty()),
    })
}

async fn fetch_canvas(app: &AppHandle, uri: &str) -> Result<Option<Canvas>, AppError> {
    use crate::internal::wire;
    use protobuf::Message;

    // EntityCanvazRequest { repeated Entity entities = 1 { string entity_uri = 1 } }
    let mut entity = Vec::new();
    wire::put_bytes(&mut entity, 1, uri.as_bytes());
    let mut req = Vec::new();
    wire::put_bytes(&mut req, 1, &entity);

    let body = crate::internal::spclient::post_raw_protobuf(app, "/canvaz-cache/v0/canvases", &req).await?;

    // EntityCanvazResponse { repeated Canvaz canvases = 1; int64 ttl_in_seconds = 2 }
    let fields = wire::fields(&body).ok_or_else(|| AppError::Network("canvas: bad protobuf".into()))?;
    for (num, val) in fields {
        if let (1, wire::Value::Bytes(b)) = (num, val) {
            if let Ok(c) = librespot_protocol::canvaz::entity_canvaz_response::Canvaz::parse_from_bytes(b) {
                if c.entity_uri.is_empty() || c.entity_uri == uri {
                    return Ok(canvas_from(&c));
                }
            }
        }
    }
    Ok(None)
}

/// the looping video (spotify "canvas") for a track, if its artist set one.
/// `None` is the common answer and is cached too, so a track without a canvas
/// isn't asked about again for a day.
#[tauri::command]
pub async fn get_canvas(app: AppHandle, track_id: String) -> Result<Option<Canvas>, AppError> {
    use tauri::Manager;
    let id = crate::internal::spclient::uri_id(&track_id).to_string();
    if id.is_empty() || track_id.starts_with("spotify:episode:") {
        return Ok(None);
    }
    let uri = format!("spotify:track:{id}");
    let pool = app.state::<crate::state::AppState>().db.clone();
    crate::internal::cache::cached_json(&pool, &uri, "canvas", crate::internal::cache::DAY, || async {
        fetch_canvas(&app, &uri).await
    })
    .await
}

// ── credits ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreditPerson {
    pub name:      String,
    /// set when the person is also a spotify artist, so the row can link
    pub artist_id: Option<String>,
    pub image_url: Option<String>,
    /// "Vocals", "Composer", "Producer", ...
    pub roles:     Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreditSection {
    /// "Performers" / "Writers" / "Producers" / ...
    pub title:  String,
    pub people: Vec<CreditPerson>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackCredits {
    pub track_name: Option<String>,
    pub sections:   Vec<CreditSection>,
    /// "Source: <label>" lines
    pub sources:    Vec<String>,
}

pub(crate) fn credits_from(v: &serde_json::Value) -> TrackCredits {
    use crate::internal::spclient::{image_uri_to_url, uri_id};
    let str_of = |v: &serde_json::Value, k: &str| {
        v.get(k).and_then(|x| x.as_str()).map(str::to_string).filter(|x| !x.is_empty())
    };
    let sections = v
        .get("roleCredits")
        .and_then(|x| x.as_array())
        .map(|roles| {
            roles
                .iter()
                .filter_map(|r| {
                    let people: Vec<CreditPerson> = r
                        .get("artists")
                        .and_then(|x| x.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|p| {
                                    let uri = str_of(p, "uri");
                                    Some(CreditPerson {
                                        name: str_of(p, "name")?,
                                        artist_id: uri
                                            .as_deref()
                                            .filter(|u| u.starts_with("spotify:artist:"))
                                            .map(|u| uri_id(u).to_string()),
                                        image_url: str_of(p, "imageUri").and_then(|i| image_uri_to_url(&i)),
                                        roles: p
                                            .get("subroles")
                                            .and_then(|x| x.as_array())
                                            .map(|s| s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                                            .unwrap_or_default(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    if people.is_empty() {
                        return None;
                    }
                    Some(CreditSection { title: str_of(r, "roleTitle").unwrap_or_else(|| "Credits".into()), people })
                })
                .collect()
        })
        .unwrap_or_default();
    TrackCredits {
        track_name: str_of(v, "trackTitle"),
        sections,
        sources: v
            .get("sourceNames")
            .and_then(|x| x.as_array())
            .map(|s| s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default(),
    }
}

/// who performed, wrote and produced a track (spclient track-credits-view)
#[tauri::command]
pub async fn get_track_credits(app: AppHandle, track_id: String) -> Result<TrackCredits, AppError> {
    use tauri::Manager;
    let id = crate::internal::spclient::uri_id(&track_id).to_string();
    if id.is_empty() || track_id.starts_with("spotify:episode:") {
        return Err(AppError::InvalidInput("credits are only available for tracks".into()));
    }
    let pool = app.state::<crate::state::AppState>().db.clone();
    let uri = format!("spotify:track:{id}");
    crate::internal::cache::cached_json(&pool, &uri, "credits", 7 * crate::internal::cache::DAY, || async {
        let v = crate::internal::spclient::get_json_value(&app, &format!("/track-credits-view/v0/experimental/{id}/credits")).await?;
        Ok(credits_from(&v))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_credits() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{
              "trackTitle": "Song",
              "roleCredits": [
                {"roleTitle": "Performers", "artists": [
                  {"uri": "spotify:artist:a1", "name": "Singer", "imageUri": "spotify:image:ab", "subroles": ["Vocals"]}
                ]},
                {"roleTitle": "Writers", "artists": [{"uri": "", "name": "Writer", "subroles": ["Composer", "Lyricist"]}]},
                {"roleTitle": "Producers", "artists": []}
              ],
              "sourceNames": ["Label"]
            }"#,
        )
        .unwrap();
        let c = credits_from(&v);
        assert_eq!(c.sections.len(), 2, "empty sections are dropped");
        assert_eq!(c.sections[0].people[0].artist_id.as_deref(), Some("a1"));
        assert_eq!(c.sections[0].people[0].image_url.as_deref(), Some("https://i.scdn.co/image/ab"));
        assert_eq!(c.sections[1].people[0].artist_id, None);
        assert_eq!(c.sections[1].people[0].roles, vec!["Composer", "Lyricist"]);
        assert_eq!(c.sources, vec!["Label"]);
    }
}

// ── playlist folders (rootlist) ──────────────────────────────────────────────

/// one entry in your library's playlist tree, in the order spotify shows it
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RootItem {
    Playlist {
        id:        String,
        name:      Option<String>,
        image_url: Option<String>,
        length:    Option<i64>,
    },
    Folder {
        id:       String,
        name:     String,
        children: Vec<RootItem>,
    },
}

/// `spotify:start-group:<id>:<url-encoded name>` -> (id, name)
fn parse_group(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix("spotify:start-group:")?;
    let (id, name) = rest.split_once(':').unwrap_or((rest, ""));
    let name = url::form_urlencoded::parse(format!("n={name}").as_bytes())
        .next()
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default();
    Some((id.to_string(), if name.is_empty() { "Folder".into() } else { name }))
}

/// flat rootlist rows (uri + optional meta) -> nested tree. start-group opens
/// a folder, end-group closes the innermost one; an unbalanced list (seen when
/// a page boundary cuts a folder) just closes whatever is still open.
pub(crate) fn build_tree(rows: Vec<(String, Option<(Option<String>, Option<String>, Option<i64>)>)>) -> Vec<RootItem> {
    let mut stack: Vec<(String, String, Vec<RootItem>)> = Vec::new();
    let mut root: Vec<RootItem> = Vec::new();
    for (uri, meta) in rows {
        if let Some((id, name)) = parse_group(&uri) {
            stack.push((id, name, Vec::new()));
            continue;
        }
        if uri.starts_with("spotify:end-group:") {
            if let Some((id, name, children)) = stack.pop() {
                let folder = RootItem::Folder { id, name, children };
                match stack.last_mut() {
                    Some(parent) => parent.2.push(folder),
                    None => root.push(folder),
                }
            }
            continue;
        }
        let Some(id) = uri.strip_prefix("spotify:playlist:") else { continue };
        let (name, image_url, length) = meta.unwrap_or((None, None, None));
        let item = RootItem::Playlist { id: id.to_string(), name, image_url, length };
        match stack.last_mut() {
            Some(parent) => parent.2.push(item),
            None => root.push(item),
        }
    }
    while let Some((id, name, children)) = stack.pop() {
        let folder = RootItem::Folder { id, name, children };
        match stack.last_mut() {
            Some(parent) => parent.2.push(folder),
            None => root.push(folder),
        }
    }
    root
}

async fn fetch_rootlist(app: &AppHandle) -> Result<Vec<RootItem>, AppError> {
    use librespot_protocol::playlist4_external::SelectedListContent;
    use protobuf::Message;

    let session = crate::internal::spclient::session(app).await?;
    let mut rows = Vec::new();
    let mut from = 0usize;
    loop {
        let bytes = session
            .spclient()
            .get_rootlist(from, Some(500))
            .await
            .map_err(crate::internal::spclient::map_err)?;
        let list = SelectedListContent::parse_from_bytes(&bytes)
            .map_err(|e| AppError::Network(format!("rootlist: {e}")))?;
        let Some(contents) = list.contents.as_ref() else { break };
        for (i, item) in contents.items.iter().enumerate() {
            let meta = contents.meta_items.get(i).map(|m| {
                let attrs = m.attributes.as_ref();
                (
                    attrs.map(|a| a.name().to_string()).filter(|n| !n.is_empty()),
                    attrs
                        .filter(|a| a.has_picture())
                        .and_then(|a| crate::internal::spclient::image_url(a.picture())),
                    m.has_length().then(|| m.length() as i64),
                )
            });
            rows.push((item.uri().to_string(), meta));
        }
        let n = contents.items.len();
        if !contents.truncated() || n == 0 || from > 20_000 {
            break;
        }
        from += n;
    }
    Ok(build_tree(rows))
}

/// your playlists as spotify's desktop client arranges them: folders, nesting
/// and order included. the web api only has a flat list.
#[tauri::command]
pub async fn get_playlist_folders(app: AppHandle) -> Result<Vec<RootItem>, AppError> {
    use tauri::Manager;
    let pool = app.state::<crate::state::AppState>().db.clone();
    crate::internal::cache::cached_json(&pool, "spotify:me", "rootlist", 10 * 60_000, || async {
        fetch_rootlist(&app).await
    })
    .await
}

#[cfg(test)]
mod rootlist_tests {
    use super::*;

    fn row(uri: &str) -> (String, Option<(Option<String>, Option<String>, Option<i64>)>) {
        (uri.to_string(), None)
    }

    #[test]
    fn nests_folders() {
        let tree = build_tree(vec![
            row("spotify:playlist:a"),
            row("spotify:start-group:f1:Road+Trip"),
            row("spotify:playlist:b"),
            row("spotify:start-group:f2:Night%20Drives"),
            row("spotify:playlist:c"),
            row("spotify:end-group:f2"),
            row("spotify:end-group:f1"),
            row("spotify:playlist:d"),
        ]);
        assert_eq!(tree.len(), 3);
        let RootItem::Folder { name, children, .. } = &tree[1] else { panic!() };
        assert_eq!(name, "Road Trip");
        assert_eq!(children.len(), 2);
        let RootItem::Folder { name, .. } = &children[1] else { panic!() };
        assert_eq!(name, "Night Drives");
    }

    #[test]
    fn closes_unbalanced_groups() {
        let tree = build_tree(vec![row("spotify:start-group:f1:X"), row("spotify:playlist:a")]);
        assert_eq!(tree.len(), 1);
        assert!(matches!(&tree[0], RootItem::Folder { children, .. } if children.len() == 1));
    }
}

// ── radio / stations / autoplay ──────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RadioResult {
    /// what spotify calls it ("<track> Radio"), when it said
    pub title:       Option<String>,
    /// the backing playlist for track radio (spotify:playlist:37i9...), so the
    /// ui can open it as a page
    pub playlist_id: Option<String>,
    pub tracks:      Vec<TrackItem>,
}

/// track ids out of a resolved context, in order, skipping non-tracks
pub(crate) fn context_track_ids(ctx: &librespot_protocol::context::Context) -> Vec<String> {
    let mut ids = Vec::new();
    for page in &ctx.pages {
        for t in &page.tracks {
            let id = if t.uri().starts_with("spotify:track:") {
                Some(crate::internal::spclient::uri_id(t.uri()).to_string())
            } else if t.uri().is_empty() && t.has_gid() {
                crate::internal::metadata::gid_to_id(t.gid())
            } else {
                None
            };
            if let Some(id) = id.filter(|i| !i.is_empty()) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    ids
}

/// apollo answers `{"tracks": [{"uri": ...}], ...}`; some builds nest it
/// under "mediaItems"
pub(crate) fn apollo_track_ids(v: &serde_json::Value) -> Vec<String> {
    let list = v
        .get("tracks")
        .or_else(|| v.get("mediaItems"))
        .and_then(|x| x.as_array());
    let mut ids = Vec::new();
    for t in list.into_iter().flatten() {
        if let Some(uri) = t.get("uri").and_then(|u| u.as_str()) {
            if uri.starts_with("spotify:track:") {
                let id = crate::internal::spclient::uri_id(uri).to_string();
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    ids
}

pub(crate) async fn resolve_context_ids(app: &AppHandle, uri: &str) -> Result<(Option<String>, Vec<String>), AppError> {
    let s = crate::internal::spclient::session(app).await?;
    let ctx = s.spclient().get_context(uri).await.map_err(crate::internal::spclient::map_err)?;
    let title = ctx
        .metadata
        .get("context_description")
        .or_else(|| ctx.metadata.get("title"))
        .cloned()
        .filter(|t| !t.is_empty());
    Ok((title, context_track_ids(&ctx)))
}

async fn apollo_ids(
    app: &AppHandle,
    scope: &str,
    seed_uri: &str,
    count: usize,
    prev: &[String],
    autoplay: bool,
) -> Result<Vec<String>, AppError> {
    let s = crate::internal::spclient::session(app).await?;
    let prev: Vec<librespot_core::SpotifyId> = prev
        .iter()
        .filter_map(|p| librespot_core::SpotifyId::from_base62(crate::internal::spclient::uri_id(p)).ok())
        .take(50)
        .collect();
    let bytes = s
        .spclient()
        .get_apollo_station(scope, seed_uri, Some(count), prev, autoplay)
        .await
        .map_err(crate::internal::spclient::map_err)?;
    let v: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| AppError::Network(format!("apollo: {e}")))?;
    Ok(apollo_track_ids(&v))
}

fn seed_uri(seed: &str) -> String {
    if seed.starts_with("spotify:") {
        seed.to_string()
    } else {
        format!("spotify:track:{seed}")
    }
}

/// spotify's own radio for a track: the "inspired by" mix first (a real,
/// openable playlist), apollo's station as the fallback
#[tauri::command]
pub async fn get_track_radio(app: AppHandle, track_id: String) -> Result<RadioResult, AppError> {
    use tauri::Manager;
    let uri = seed_uri(&track_id);
    let pool = app.state::<crate::state::AppState>().db.clone();
    crate::internal::cache::cached_json(&pool, &uri, "radio", 6 * crate::internal::cache::HOUR, || async {
        let s = crate::internal::spclient::session(&app).await?;
        let spotify_uri = librespot_core::SpotifyUri::from_uri(&uri)
            .map_err(|e| AppError::InvalidInput(e.to_string()))?;

        let mut playlist_id = None;
        let mut title = None;
        let mut ids = Vec::new();
        if let Ok(bytes) = s.spclient().get_radio_for_track(&spotify_uri).await {
            let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
            let pl_uri = v
                .get("mediaItems")
                .and_then(|m| m.as_array())
                .and_then(|a| a.first())
                .and_then(|i| i.get("uri"))
                .and_then(|u| u.as_str())
                .map(str::to_string);
            if let Some(pl_uri) = pl_uri {
                if let Ok((t, found)) = resolve_context_ids(&app, &pl_uri).await {
                    playlist_id = Some(crate::internal::spclient::uri_id(&pl_uri).to_string());
                    title = t;
                    ids = found;
                }
            }
        }
        if ids.is_empty() {
            ids = apollo_ids(&app, "stations", &uri, 50, &[], false).await?;
        }
        let tracks = crate::internal::metadata::tracks(&app, &ids).await?;
        Ok(RadioResult { title, playlist_id, tracks })
    })
    .await
}

/// a station seeded from any artist / album / playlist / track uri
#[tauri::command]
pub async fn get_station(app: AppHandle, seed: String) -> Result<RadioResult, AppError> {
    use tauri::Manager;
    let uri = seed_uri(&seed);
    let pool = app.state::<crate::state::AppState>().db.clone();
    crate::internal::cache::cached_json(&pool, &uri, "station", 6 * crate::internal::cache::HOUR, || async {
        let ids = apollo_ids(&app, "stations", &uri, 50, &[], false).await?;
        let tracks = crate::internal::metadata::tracks(&app, &ids).await?;
        Ok(RadioResult { title: None, playlist_id: None, tracks })
    })
    .await
}

/// what spotify would autoplay after `context_uri`, given what just played.
/// this is the auto-DJ's first choice; the local taste engine is its fallback
#[tauri::command]
pub async fn get_autoplay_tracks(
    app: AppHandle,
    context_uri: String,
    recent_track_ids: Vec<String>,
) -> Result<Vec<TrackItem>, AppError> {
    use librespot_protocol::autoplay_context_request::AutoplayContextRequest;
    if context_uri.contains(":show:") || context_uri.contains(":episode:") {
        return Ok(Vec::new());
    }
    let recent: Vec<String> = recent_track_ids.iter().rev().take(30).map(|i| seed_uri(i)).collect();
    let s = crate::internal::spclient::session(&app).await?;
    let req = AutoplayContextRequest {
        context_uri: Some(seed_uri(&context_uri)),
        recent_track_uri: recent,
        ..Default::default()
    };
    let ids = match s.spclient().get_autoplay_context(&req).await {
        Ok(ctx) => context_track_ids(&ctx),
        Err(_) => Vec::new(),
    };
    let ids = if ids.is_empty() {
        apollo_ids(&app, "tracks", &seed_uri(&context_uri), 30, &recent_track_ids, true).await?
    } else {
        ids
    };
    let recent: std::collections::HashSet<&str> =
        recent_track_ids.iter().map(|i| crate::internal::spclient::uri_id(i)).collect();
    let ids: Vec<String> = ids.into_iter().filter(|i| !recent.contains(i.as_str())).collect();
    crate::internal::metadata::tracks(&app, &ids).await
}

#[cfg(test)]
mod radio_tests {
    use super::*;

    #[test]
    fn reads_apollo_tracks() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"tracks": [{"uri": "spotify:track:a"}, {"uri": "spotify:episode:x"}, {"uri": "spotify:track:a"}, {"uri": "spotify:track:b"}]}"#,
        )
        .unwrap();
        assert_eq!(apollo_track_ids(&v), vec!["a", "b"]);
    }

    #[test]
    fn reads_context_tracks() {
        use librespot_protocol::{context::Context, context_page::ContextPage, context_track::ContextTrack};
        let mut page = ContextPage::new();
        for uri in ["spotify:track:one", "spotify:episode:nope", "spotify:track:two"] {
            let mut t = ContextTrack::new();
            t.set_uri(uri.into());
            page.tracks.push(t);
        }
        let mut ctx = Context::new();
        ctx.pages.push(page);
        assert_eq!(context_track_ids(&ctx), vec!["one", "two"]);
    }
}
