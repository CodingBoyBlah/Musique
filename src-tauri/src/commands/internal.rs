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
