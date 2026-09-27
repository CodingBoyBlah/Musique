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
