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
    if crate::apple::is_active() {
        return crate::apple::api::tracks(&ids).await;
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
    // Spotify-only: an Apple Music account has none
    if crate::apple::is_active() {
        return Ok(None);
    }
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
    if crate::apple::is_active() {
        return Err(AppError::NotFound("credits come from Spotify's catalogue".into()));
    }
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
                    attrs.and_then(crate::internal::spclient::list_picture),
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
    // Apple Music playlists have no Spotify rootlist: the flat list is used
    if crate::apple::is_active() {
        return Err(AppError::NotFound("no playlist folders for Apple Music".into()));
    }
    use tauri::Manager;
    let pool = app.state::<crate::state::AppState>().db.clone();
    // "rootlist2": older cached trees predate picture_size covers, so they'd
    // keep showing spotify-made playlists blank until the ttl ran out
    crate::internal::cache::cached_json(&pool, "spotify:me", "rootlist2", 10 * 60_000, || async {
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

/// the playlist spotify's own client opens as "<seed> Radio": the inspired-by
/// mix for a track, the artist radio (37i9dQZF1E4...) for an artist. any seed
/// uri works - checked live for tracks and artists. (playlist id, title, ids)
async fn radio_playlist(app: &AppHandle, uri: &str) -> Option<(String, Option<String>, Vec<String>)> {
    let s = crate::internal::spclient::session(app).await.ok()?;
    let spotify_uri = librespot_core::SpotifyUri::from_uri(uri).ok()?;
    let bytes = s.spclient().get_radio_for_track(&spotify_uri).await.ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let pl_uri = v
        .get("mediaItems")?
        .as_array()?
        .first()?
        .get("uri")?
        .as_str()?
        .to_string();
    let (title, ids) = resolve_context_ids(app, &pl_uri).await.ok()?;
    if ids.is_empty() {
        return None;
    }
    Some((crate::internal::spclient::uri_id(&pl_uri).to_string(), title, ids))
}

/// spotify's own radio for a track: the "inspired by" mix first (a real,
/// openable playlist), apollo's station as the fallback
#[tauri::command]
pub async fn get_track_radio(app: AppHandle, track_id: String) -> Result<RadioResult, AppError> {
    use tauri::Manager;
    if crate::apple::is_active() {
        return apple_station(&track_id).await;
    }
    let uri = seed_uri(&track_id);
    let pool = app.state::<crate::state::AppState>().db.clone();
    crate::internal::cache::cached_json(&pool, &uri, "radio", 6 * crate::internal::cache::HOUR, || async {
        let (playlist_id, title, ids) = match radio_playlist(&app, &uri).await {
            Some((id, title, ids)) => (Some(id), title, ids),
            None => (None, None, apollo_ids(&app, "stations", &uri, 50, &[], false).await?),
        };
        let tracks = crate::internal::metadata::tracks(&app, &ids).await?;
        Ok(RadioResult { title, playlist_id, tracks })
    })
    .await
}

/// a station seeded from any artist / album / playlist / track uri: spotify's
/// own radio playlist for it when there is one (an artist always has one),
/// otherwise apollo's station
#[tauri::command]
pub async fn get_station(app: AppHandle, seed: String) -> Result<RadioResult, AppError> {
    use tauri::Manager;
    if crate::apple::is_active() {
        return apple_station(&seed).await;
    }
    let uri = seed_uri(&seed);
    let pool = app.state::<crate::state::AppState>().db.clone();
    // "-v2": entries cached before the radio playlist was looked up have no
    // playlist id, and would keep the button on the anonymous station for 6h
    crate::internal::cache::cached_json(&pool, &uri, "station-v2", 6 * crate::internal::cache::HOUR, || async {
        let (playlist_id, title, ids) = match radio_playlist(&app, &uri).await {
            Some((id, title, ids)) => (Some(id), title, ids),
            None => (None, None, apollo_ids(&app, "stations", &uri, 50, &[], false).await?),
        };
        let tracks = crate::internal::metadata::tracks(&app, &ids).await?;
        Ok(RadioResult { title, playlist_id, tracks })
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
    if crate::apple::is_active() {
        // carry on from the last song heard, or from the context when nothing was
        let seed = recent_track_ids.last().cloned().unwrap_or_else(|| context_uri.clone());
        let recent: std::collections::HashSet<&String> = recent_track_ids.iter().collect();
        let station = apple_station(&seed).await?;
        return Ok(station.tracks.into_iter().filter(|t| !recent.contains(&t.id)).collect());
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

// ── spotify-made playlists ───────────────────────────────────────────────────

/// a playlist read straight off spclient's `playlist/v2` - the path for the
/// algorithmic and editorial ones (Daily Mix, Discover Weekly, "This Is",
/// radio mixes) the public web api 404s for most apps. tracks are hydrated
/// through extended-metadata.
pub(crate) async fn internal_playlist(
    app: &AppHandle,
    id: &str,
) -> Result<crate::spotify::types::PlaylistDetail, AppError> {
    use librespot_protocol::playlist4_external::SelectedListContent;
    use protobuf::Message;

    let sid = librespot_core::SpotifyId::from_base62(id)
        .map_err(|_| AppError::InvalidInput(format!("bad playlist id: {id}")))?;
    let s = crate::internal::spclient::session(app).await?;

    let (name, description, image_url, owner, ids) = match s.spclient().get_playlist(&sid).await {
        Ok(bytes) => {
            let list = SelectedListContent::parse_from_bytes(&bytes)
                .map_err(|e| AppError::Network(format!("playlist v2: {e}")))?;
            let attrs = list.attributes.as_ref();
            let ids: Vec<String> = list
                .contents
                .as_ref()
                .map(|c| {
                    c.items
                        .iter()
                        .filter(|i| i.uri().starts_with("spotify:track:"))
                        .map(|i| crate::internal::spclient::uri_id(i.uri()).to_string())
                        .collect()
                })
                .unwrap_or_default();
            (
                attrs.map(|a| a.name().to_string()).filter(|n| !n.is_empty()),
                attrs.map(|a| a.description().to_string()).filter(|d| !d.is_empty()),
                attrs.and_then(crate::internal::spclient::list_picture),
                Some(list.owner_username().to_string()).filter(|o| !o.is_empty()),
                ids,
            )
        }
        // some personalised lists only resolve as a context
        Err(_) => {
            let (title, ids) = resolve_context_ids(app, &format!("spotify:playlist:{id}")).await?;
            (title, None, None, Some("spotify".to_string()), ids)
        }
    };

    let tracks = crate::internal::metadata::tracks(app, &ids).await?;
    Ok(crate::spotify::types::PlaylistDetail {
        id: id.to_string(),
        name: name.unwrap_or_else(|| "Playlist".into()),
        description,
        image_url,
        owner_name: owner.as_deref().map(|o| if o == "spotify" { "Spotify".to_string() } else { o.to_string() }),
        total_tracks: tracks.len() as i64,
        tracks,
        owner_id: owner,
        public: None,
        collaborative: None,
        followers: None,
    })
}

// ── extracted colours (pathfinder) ───────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedColor {
    /// "#rrggbb" as spotify picked them for this artwork
    pub raw:   Option<String>,
    pub dark:  Option<String>,
    pub light: Option<String>,
}

pub(crate) fn colors_from(data: &serde_json::Value) -> Vec<Option<ExtractedColor>> {
    use crate::internal::pathfinder::get_str;
    data.get("extractedColors")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| {
                    let raw = get_str(c, &["colorRaw", "hex"]).map(str::to_string);
                    let dark = get_str(c, &["colorDark", "hex"]).map(str::to_string);
                    let light = get_str(c, &["colorLight", "hex"]).map(str::to_string);
                    (raw.is_some() || dark.is_some()).then_some(ExtractedColor { raw, dark, light })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// spotify's own palette for artwork urls, same order as asked. used where
/// the in-app extractor can't read the image
#[tauri::command]
pub async fn get_extracted_colors(app: AppHandle, image_urls: Vec<String>) -> Result<Vec<Option<ExtractedColor>>, AppError> {
    use tauri::Manager;
    if image_urls.is_empty() {
        return Ok(Vec::new());
    }
    let pool = app.state::<crate::state::AppState>().db.clone();
    let mut out: Vec<Option<ExtractedColor>> = Vec::with_capacity(image_urls.len());
    let mut missing: Vec<String> = Vec::new();
    for url in &image_urls {
        let hit = crate::internal::cache::get_json::<Option<ExtractedColor>>(&pool, url, "colors", 30 * crate::internal::cache::DAY).await;
        if hit.is_none() {
            missing.push(url.clone());
        }
        out.push(hit.flatten());
    }
    if !missing.is_empty() {
        let data = crate::internal::pathfinder::query(&app, "fetchExtractedColors", serde_json::json!({ "uris": missing })).await?;
        let fetched = colors_from(&data);
        for (url, color) in missing.iter().zip(fetched.iter()) {
            crate::internal::cache::put_json(&pool, url, "colors", color).await;
            if let Some(i) = image_urls.iter().position(|u| u == url) {
                out[i] = color.clone();
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn reads_colors() {
        let v: serde_json::Value = serde_json::from_str(
            r##"{"extractedColors": [
                {"colorRaw": {"hex": "#A0522D"}, "colorDark": {"hex": "#503010"}, "colorLight": {"hex": "#E0C0A0"}},
                {"colorRaw": {"hex": ""}}
            ]}"##,
        )
        .unwrap();
        let c = colors_from(&v);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].as_ref().unwrap().raw.as_deref(), Some("#A0522D"));
        assert!(c[1].is_none());
    }
}

// ── blends ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlendMember {
    pub username: String,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlendInfo {
    pub is_blend: bool,
    pub format: Option<String>,
    pub members: Vec<BlendMember>,
    pub taste_match: Option<u8>,
    pub contributors: std::collections::HashMap<String, Vec<String>>,
}

/// members off the list's format attributes, in the order spotify lists them:
/// `blend.userinfo-<user>.{username,name,profileimage}`
fn blend_members(fattrs: &[(String, String)]) -> Vec<BlendMember> {
    let mut out: Vec<BlendMember> = Vec::new();
    for (k, v) in fattrs {
        let Some(rest) = k.strip_prefix("blend.userinfo-") else { continue };
        let Some((user, field)) = rest.rsplit_once('.') else { continue };
        if user.is_empty() {
            continue;
        }
        let i = match out.iter().position(|m| m.username == user) {
            Some(i) => i,
            None => {
                out.push(BlendMember { username: user.to_string(), name: user.to_string(), image_url: None });
                out.len() - 1
            }
        };
        let v = v.trim();
        match field {
            "name" if !v.is_empty() => out[i].name = v.to_string(),
            "profileimage" if !v.is_empty() => out[i].image_url = Some(v.to_string()),
            _ => {}
        }
    }
    out
}

/// who a blend track was picked for: `multiUserAttributionMetadata` JSON, else
/// the comma list in `multiUserAttributionUsernamesV2`. never `added_by` (it is
/// always the viewer)
fn blend_item_users(fattrs: &[(String, String)]) -> Vec<String> {
    if let Some((_, json)) = fattrs.iter().find(|(k, _)| k == "multiUserAttributionMetadata") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
            let users: Vec<String> = v["attributed_users"]
                .as_array()
                .map(|a| a.iter().filter_map(|u| u["username"].as_str()).filter(|u| !u.is_empty()).map(str::to_string).collect())
                .unwrap_or_default();
            if !users.is_empty() {
                return users;
            }
        }
    }
    fattrs
        .iter()
        .find(|(k, _)| k == "multiUserAttributionUsernamesV2")
        .map(|(_, v)| v.split(',').map(str::trim).filter(|u| !u.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

async fn fetch_blend_info(app: &AppHandle, id: &str) -> Result<BlendInfo, AppError> {
    use librespot_protocol::playlist4_external::SelectedListContent;
    use protobuf::Message;

    let sid = librespot_core::SpotifyId::from_base62(id)
        .map_err(|_| AppError::InvalidInput(format!("bad playlist id: {id}")))?;
    let s = crate::internal::spclient::session(app).await?;
    let bytes = s
        .spclient()
        .get_playlist(&sid)
        .await
        .map_err(|e| AppError::Network(format!("playlist v2: {e}")))?;
    let list = SelectedListContent::parse_from_bytes(&bytes)
        .map_err(|e| AppError::Network(format!("playlist v2: {e}")))?;

    let attrs = list.attributes.as_ref();
    let format = attrs.map(|a| a.format().to_string()).filter(|f| !f.is_empty());
    if !format.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("blend")) {
        return Ok(BlendInfo { format, ..Default::default() });
    }
    let fattrs: Vec<(String, String)> = attrs
        .map(|a| a.format_attributes.iter().map(|f| (f.key().to_string(), f.value().to_string())).collect())
        .unwrap_or_default();
    let members = blend_members(&fattrs);

    let mut contributors: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    if let Some(c) = list.contents.as_ref() {
        for item in &c.items {
            if !item.uri().starts_with("spotify:track:") {
                continue;
            }
            let Some(ia) = item.attributes.as_ref() else { continue };
            let f: Vec<(String, String)> = ia.format_attributes.iter().map(|f| (f.key().to_string(), f.value().to_string())).collect();
            let users = blend_item_users(&f);
            if !users.is_empty() {
                contributors.insert(crate::internal::spclient::uri_id(item.uri()).to_string(), users);
            }
        }
    }

    // spotify doesn't put the taste match on the list itself
    Ok(BlendInfo { is_blend: true, format, members, taste_match: None, contributors })
}

#[cfg(test)]
mod blend_tests {
    use super::*;

    fn kv(k: &str, v: &str) -> (String, String) {
        (k.to_string(), v.to_string())
    }

    #[test]
    fn reads_members_and_attribution() {
        let list = vec![
            kv("madeFor.username", "me"),
            kv("blend.userinfo-me.username", "me"),
            kv("blend.userinfo-me.name", "anshuman"),
            kv("blend.userinfo-me.profileimage", "https://i.scdn.co/image/a"),
            kv("blend.userinfo-31x.username", "31x"),
            kv("blend.userinfo-31x.name", "tharun"),
            kv("track_attributed_desc", "Listened to this song"),
        ];
        let m = blend_members(&list);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].name, "anshuman");
        assert_eq!(m[0].image_url.as_deref(), Some("https://i.scdn.co/image/a"));
        assert_eq!(m[1].username, "31x");
        assert_eq!(m[1].image_url, None);

        let item = vec![kv(
            "multiUserAttributionMetadata",
            r#"{"attributed_users":[{"username":"31x","display_name":"tharun"},{"username":"me"}]}"#,
        )];
        assert_eq!(blend_item_users(&item), vec!["31x", "me"]);
        let item = vec![kv("multiUserAttributionUsernamesV2", "me, 31x")];
        assert_eq!(blend_item_users(&item), vec!["me", "31x"]);
        assert!(blend_item_users(&[]).is_empty());
    }
}

/// blend members, taste match and who each track was picked for, read off the
/// playlist4 list. non-blends come back with `isBlend: false`.
#[tauri::command]
pub async fn get_blend_info(app: AppHandle, id: String) -> Result<Option<BlendInfo>, AppError> {
    // Blends are Spotify playlists
    if crate::apple::is_active() {
        return Ok(None);
    }
    use tauri::Manager;
    let pool = app.state::<crate::state::AppState>().db.clone();
    let uri = format!("spotify:playlist:{id}");
    let info = crate::internal::cache::cached_json(&pool, &uri, "blend2", 10 * 60_000, || async {
        fetch_blend_info(&app, &id).await
    })
    .await?;
    Ok(Some(info))
}

/// An Apple Music station from any seed the interface hands over: a bare track id, or a
/// `spotify:<kind>:<id>` context uri, which the app uses as its own context naming whatever
/// the provider. Apple only seeds stations from a song, so an album, artist or playlist is
/// seeded by its first one.
async fn apple_station(seed: &str) -> Result<RadioResult, AppError> {
    let mut parts = seed.rsplitn(3, ':');
    let id = parts.next().unwrap_or(seed).to_string();
    let kind = parts.next().unwrap_or("track");
    let track_id = match kind {
        "album" => crate::apple::api::album(&id).await?.tracks.first().map(|t| t.id.clone()),
        "artist" => crate::apple::api::artist(&id).await?.top_tracks.first().map(|t| t.id.clone()),
        "playlist" => crate::apple::api::playlist(&id).await?.tracks.first().map(|t| t.id.clone()),
        _ => Some(id),
    };
    let Some(track_id) = track_id else {
        return Err(AppError::NotFound("nothing to start a station from".into()));
    };
    crate::apple::api::track_radio(&track_id).await
}
