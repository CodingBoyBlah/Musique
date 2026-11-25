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
    /// episodes only: the show it belongs to (there's no episode page to open)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_id:   Option<String>,
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
    /// the untitled "shorts" grid at the top of spotify's home: the user's
    /// recent and heavy-rotation playlists, albums, artists and liked songs.
    /// liked songs comes through as kind "collection". `default` keeps feeds
    /// cached before this field existed readable.
    #[serde(default)]
    pub shortcuts: Vec<HomeItem>,
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
    let show_id = (kind == "episode")
        .then(|| get_str(d, &["podcastV2", "data", "uri"]))
        .flatten()
        .and_then(|u| u.strip_prefix("spotify:show:"))
        .map(str::to_string);
    Some(HomeItem { kind: kind.into(), id: id.to_string(), name, subtitle, image_url: image(d), show_id })
}

/// a shortcut tile: an ordinary item, or one of the user's collections, whose
/// uris (`spotify:user:<name>:collection`, `spotify:collection:tracks`,
/// `...:collection:your-episodes`) the plain parser rejects. liked songs is
/// kind "collection", your episodes is kind "your-episodes".
fn shortcut_from(v: &Value) -> Option<HomeItem> {
    if let Some(item) = item_from(v) {
        return Some(item);
    }
    let d = get(v, &["content", "data"]).or_else(|| get(v, &["itemV2", "data"])).unwrap_or(v);
    let uri = get_str(d, &["uri"]).or_else(|| get_str(v, &["uri"]))?;
    if !uri.contains(":collection") {
        return None;
    }
    let (kind, id, name) = if uri.contains("your-episodes") {
        ("your-episodes", "your-episodes", "Your Episodes")
    } else {
        ("collection", "liked-songs", "Liked Songs")
    };
    Some(HomeItem {
        kind:      kind.into(),
        id:        id.into(),
        name:      get_str(d, &["name"]).unwrap_or(name).to_string(),
        subtitle:  None,
        image_url: image(d),
        show_id:   None,
    })
}

fn is_shortcuts_section(s: &Value) -> bool {
    get_str(s, &["data", "__typename"]).is_some_and(|t| t.contains("Short"))
}

pub(crate) fn feed_from(data: &Value) -> HomeFeed {
    let greeting = get_str(data, &["home", "greeting", "transformedLabel"])
        .or_else(|| get_str(data, &["home", "greeting", "text"]))
        .map(str::to_string);
    let raw = get(data, &["home", "sectionContainer", "sections", "items"]).and_then(|x| x.as_array());
    let shortcuts: Vec<HomeItem> = raw
        .and_then(|secs| secs.iter().find(|s| is_shortcuts_section(s)))
        .and_then(|s| get(s, &["sectionItems", "items"])?.as_array())
        .map(|items| items.iter().filter_map(shortcut_from).collect())
        .unwrap_or_default();
    let sections = raw
        .map(|secs| {
            secs.iter()
                .filter(|s| !is_shortcuts_section(s))
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
    HomeFeed { greeting, sections, shortcuts }
}

/// the home feed. the stored copy answers at once; one older than half an hour
/// is refetched behind it and the page updated when it lands
#[tauri::command]
pub async fn get_home_feed(app: AppHandle, time_zone: Option<String>) -> Result<HomeFeed, AppError> {
    if crate::apple::is_active() {
        return crate::apple::api::home_feed().await;
    }
    let pool = app.state::<AppState>().db.clone();
    let user = spclient::account_username(&app).await.unwrap_or_else(|_| "me".into());
    let key = format!("spotify:user:{user}");
    let query_key = json!(["library", "home-feed"]);
    let handle = app.clone();
    cache::swr_json(&app, &pool, &key, "home-feed", 30 * 60_000, query_key, move || async move {
        let app = handle;
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
        if feed.sections.is_empty() && feed.shortcuts.is_empty() {
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
                "uri": "spotify:section:shorts",
                "data": {"__typename": "HomeShortsSectionData"},
                "sectionItems": {"items": [
                  {"uri": "spotify:user:me:collection", "content": {"data": {"__typename": "LikedSongs", "uri": "spotify:user:me:collection"}}},
                  {"uri": "spotify:user:me:collection:your-episodes", "content": {"data": {"uri": "spotify:user:me:collection:your-episodes"}}},
                  {"uri": "spotify:episode:ep1", "content": {"data": {"uri": "spotify:episode:ep1", "name": "Ep One",
                    "podcastV2": {"data": {"uri": "spotify:show:sh1", "name": "The Show"}}}}},
                  {"uri": "spotify:playlist:pl1", "content": {"data": {"uri": "spotify:playlist:pl1", "name": "Road Trip",
                    "images": {"items": [{"sources": [{"url": "https://img/p", "width": 300}]}]}}}}
                ]}
              },
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
        assert_eq!(feed.shortcuts.len(), 4);
        assert_eq!(feed.shortcuts[0].kind, "collection");
        assert_eq!(feed.shortcuts[1].kind, "your-episodes");
        assert_eq!(feed.shortcuts[2].show_id.as_deref(), Some("sh1"));
        assert_eq!(feed.shortcuts[3].name, "Road Trip");
    }
}
