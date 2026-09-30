//! spotify's personalised home feed (pathfinder `home`): the made-for-you
//! shelves - daily mixes, daylist, discover weekly, release radar, "jump back
//! in", mood mixes. none of it exists on the public api.
//!
//! the response is a deep graphql tree whose __typenames drift between web
//! player releases, so parsing is deliberately loose: find the name, the art
//! and the uri wherever they are, and drop anything that doesn't have them.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    internal::{
        cache,
        pathfinder::{self, get, get_str},
        spclient,
    },
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeItem {
    /// playlist | album | artist | show | episode | track
    pub kind:      String,
    pub id:        String,
    pub name:      String,
    pub subtitle:  Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeSection {
    pub id:    String,
    pub title: String,
    pub items: Vec<HomeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeFeed {
    pub greeting: Option<String>,
    pub sections: Vec<HomeSection>,
}

/// first artwork url anywhere in the usual spots
fn image(d: &Value) -> Option<String> {
    let candidates = [
        get(d, &["images", "items", "0", "sources"]),
        get(d, &["coverArt", "sources"]),
        get(d, &["visuals", "avatarImage", "sources"]),
        get(d, &["podcastV2", "data", "coverArt", "sources"]),
        get(d, &["albumOfTrack", "coverArt", "sources"]),
    ];
    for sources in candidates.into_iter().flatten() {
        let Some(arr) = sources.as_array() else { continue };
        // prefer a mid-size image (~300px) over the 640 original for tiles
        let pick = arr
            .iter()
            .filter_map(|s| Some((s.get("url")?.as_str()?, s.get("width").and_then(|w| w.as_i64()).unwrap_or(0))))
            .min_by_key(|(_, w)| (w - 300).abs());
        if let Some((url, _)) = pick {
            return Some(url.to_string());
        }
    }
    None
}

fn plain(text: &str) -> String {
    crate::commands::artist_extras::strip_html(text).replace('\n', " ")
}

fn artists_line(d: &Value) -> Option<String> {
    let names: Vec<&str> = get(d, &["artists", "items"])?
        .as_array()?
        .iter()
        .filter_map(|a| get_str(a, &["profile", "name"]))
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

pub(crate) fn item_from(v: &Value) -> Option<HomeItem> {
    // items wrap their entity as content.data (older builds: itemV2.data,
    // artist overview lists: a bare data)
    let d = get(v, &["content", "data"])
        .or_else(|| get(v, &["itemV2", "data"]))
        .or_else(|| get(v, &["data"]).filter(|d| d.get("uri").is_some()))
        .unwrap_or(v);
    let uri = get_str(d, &["uri"]).or_else(|| get_str(v, &["uri"]))?;
    let mut parts = uri.split(':');
    let (_, kind, id) = (parts.next()?, parts.next()?, parts.next()?);
    let kind = match kind {
        "playlist" | "album" | "artist" | "show" | "episode" | "track" => kind,
        _ => return None,
    };
    let name = get_str(d, &["name"]).or_else(|| get_str(d, &["profile", "name"]))?.to_string();
    let subtitle = match kind {
        "playlist" => get_str(d, &["description"])
            .map(plain)
            .filter(|s| !s.is_empty())
            .or_else(|| get_str(d, &["ownerV2", "data", "name"]).map(|o| format!("By {o}"))),
        "album" => artists_line(d),
        "track" => artists_line(d),
        "artist" => Some("Artist".into()),
        "show" => get_str(d, &["publisher", "name"]).map(str::to_string),
        "episode" => get_str(d, &["podcastV2", "data", "name"]).map(str::to_string),
        _ => None,
    };
    Some(HomeItem { kind: kind.into(), id: id.to_string(), name, subtitle, image_url: image(d) })
}

pub(crate) fn feed_from(data: &Value) -> HomeFeed {
    let greeting = get_str(data, &["home", "greeting", "transformedLabel"])
        .or_else(|| get_str(data, &["home", "greeting", "text"]))
        .map(str::to_string);
    let sections = get(data, &["home", "sectionContainer", "sections", "items"])
        .and_then(|x| x.as_array())
        .map(|secs| {
            secs.iter()
                .filter_map(|s| {
                    let title = get_str(s, &["data", "title", "transformedLabel"])
                        .or_else(|| get_str(s, &["data", "title", "text"]))
                        .or_else(|| get_str(s, &["data", "title", "originalLabel"]))?
                        .to_string();
                    let items: Vec<HomeItem> = get(s, &["sectionItems", "items"])?
                        .as_array()?
                        .iter()
                        .filter_map(item_from)
                        .collect();
                    if items.is_empty() {
                        return None;
                    }
                    let id = get_str(s, &["uri"]).map(str::to_string).unwrap_or_else(|| title.clone());
                    Some(HomeSection { id, title, items })
                })
                .collect()
        })
        .unwrap_or_default();
    HomeFeed { greeting, sections }
}

/// the home feed, cached half an hour and served stale when pathfinder is down
#[tauri::command]
pub async fn get_home_feed(app: AppHandle, time_zone: Option<String>) -> Result<HomeFeed, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let user = spclient::username(&app).await.unwrap_or_else(|_| "me".into());
    let key = format!("spotify:user:{user}");
    cache::cached_json(&pool, &key, "home-feed", 30 * 60_000, || async {
        let data = pathfinder::query(
            &app,
            "home",
            json!({
                "homeEndUserIntegration": "INTEGRATION_WEB_PLAYER",
                "timeZone": time_zone.unwrap_or_else(|| "UTC".into()),
                "sp_t": "",
                "facet": "",
                "sectionItemsLimit": 12,
                "includeEpisodeContentRatingsV2": false,
            }),
        )
        .await?;
        let feed = feed_from(&data);
        if feed.sections.is_empty() {
            return Err(AppError::NotFound("home feed came back empty".into()));
        }
        Ok(feed)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_home_feed() {
        let data = json!({
          "home": {
            "greeting": {"transformedLabel": "Good evening"},
            "sectionContainer": {"sections": {"items": [
              {
                "uri": "spotify:section:0JQ5DAqbMKFHOzuVTgTizF",
                "data": {"__typename": "HomeGenericSectionData", "title": {"transformedLabel": "Made For You"}},
                "sectionItems": {"items": [
                  {"uri": "spotify:playlist:37i9dQZF1E35", "content": {"__typename": "PlaylistResponseWrapper", "data": {
                    "__typename": "Playlist", "uri": "spotify:playlist:37i9dQZF1E35", "name": "Daily Mix 1",
                    "description": "Band A, Band B and more",
                    "images": {"items": [{"sources": [{"url": "https://img/640", "width": 640}, {"url": "https://img/300", "width": 300}]}]}
                  }}},
                  {"uri": "spotify:album:alb", "content": {"data": {
                    "uri": "spotify:album:alb", "name": "Record",
                    "artists": {"items": [{"profile": {"name": "Band A"}}]},
                    "coverArt": {"sources": [{"url": "https://img/a"}]}
                  }}},
                  {"uri": "spotify:unknown:x", "content": {"data": {"uri": "spotify:unknown:x", "name": "?"}}}
                ]}
              },
              {"uri": "spotify:section:empty", "data": {"title": {"text": "Nothing"}}, "sectionItems": {"items": []}}
            ]}}
          }
        });
        let feed = feed_from(&data);
        assert_eq!(feed.greeting.as_deref(), Some("Good evening"));
        assert_eq!(feed.sections.len(), 1);
        let s = &feed.sections[0];
        assert_eq!(s.title, "Made For You");
        assert_eq!(s.items.len(), 2);
        assert_eq!(s.items[0].name, "Daily Mix 1");
        assert_eq!(s.items[0].image_url.as_deref(), Some("https://img/300"));
        assert_eq!(s.items[1].subtitle.as_deref(), Some("Band A"));
    }
}
