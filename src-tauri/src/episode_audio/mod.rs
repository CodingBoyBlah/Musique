//! podcast audio when playback comes from youtube music instead of spotify.
//!
//! spotify won't hand episode audio to anything but its own streaming session,
//! which the youtube backend by definition isn't using. but almost every show
//! on spotify is an ordinary rss podcast underneath, so the same episode is
//! sitting on the publisher's own cdn:
//!
//!  1. **the show's rss feed.** apple's public podcast directory maps the show
//!     name to its feed; the feed's `<enclosure>` for this episode is the audio.
//!     same file, same ads, no account needed. covers nearly everything.
//!  2. **youtube music.** shows that also post to youtube (a lot of the video
//!     podcasts) are in its episodes search.
//!
//! what's left - spotify originals/exclusives that exist nowhere else - really
//! can't play here, and the error says so.
//!
//! a found source is remembered per episode, so the next play (and every
//! resume) skips the lookup.

pub mod feed;
pub mod mp4;
pub mod remote;
pub mod source;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::{errors::AppError, http, internal::cache};

use feed::FeedItem;

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EpisodeInfo {
    name:         String,
    show:         String,
    publisher:    String,
    release_date: Option<String>,
    duration_ms:  u64,
}

/// where an episode's audio was found
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "via", rename_all = "lowercase")]
enum Pick {
    Rss { url: String, mime: Option<String> },
    Youtube { video_id: String },
}

/// a playable url for an episode
pub struct Resolved {
    pub url:         String,
    pub user_agent:  Option<&'static str>,
    pub mime:        Option<String>,
    /// spotify's length for it - only used to aim seeks
    pub duration_ms: u64,
    pub via:         &'static str,
}

async fn info(pool: &SqlitePool, token: &str, id: &str) -> Result<EpisodeInfo, AppError> {
    let uri = format!("spotify:episode:{id}");
    cache::cached_json(pool, &uri, "episode-audio-info", 7 * DAY_MS, || async {
        let v: Value = crate::spotify::spotify_get(
            token,
            &format!("{}/episodes/{id}?market=from_token", crate::commands::spotify::BASE),
        )
        .await?;
        let s = |v: &Value, k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or_default().to_string();
        let show = v.get("show").cloned().unwrap_or(Value::Null);
        Ok(EpisodeInfo {
            name: s(&v, "name"),
            show: s(&show, "name"),
            publisher: s(&show, "publisher"),
            release_date: v.get("release_date").and_then(|x| x.as_str()).map(str::to_string),
            duration_ms: v.get("duration_ms").and_then(|x| x.as_u64()).unwrap_or(0),
        })
    })
    .await
}

/// feed urls for a show, best guess first, from apple's podcast directory
async fn feeds_for(pool: &SqlitePool, show: &str, publisher: &str) -> Vec<String> {
    let key = format!("podcast-feed:{}", feed::normalize(show));
    let found: Result<Vec<String>, AppError> = cache::cached_json(pool, &key, "podcast-feed", 3 * DAY_MS, || async {
        let v: Value = http::client()
            .get("https://itunes.apple.com/search")
            .query(&[("media", "podcast"), ("entity", "podcast"), ("limit", "10"), ("term", show)])
            .send()
            .await
            .map_err(|e| AppError::Network(format!("podcast directory: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Network(format!("podcast directory: {e}")))?;
        let mut ranked: Vec<(u8, String)> = v
            .get("results")
            .and_then(|r| r.as_array())
            .into_iter()
            .flatten()
            .filter_map(|r| {
                let url = r.get("feedUrl")?.as_str()?.to_string();
                let name = r.get("collectionName").and_then(|x| x.as_str()).unwrap_or_default();
                let artist = r.get("artistName").and_then(|x| x.as_str()).unwrap_or_default();
                let same_name = feed::normalize(name) == feed::normalize(show);
                let same_publisher = !publisher.is_empty() && feed::title_similarity(artist, publisher) >= 0.6;
                let rank = match (same_name, same_publisher) {
                    (true, true) => 0,
                    (true, false) => 1,
                    (false, true) if feed::title_similarity(name, show) >= 0.6 => 2,
                    _ => return None,
                };
                Some((rank, url))
            })
            .collect();
        ranked.sort_by_key(|(rank, _)| *rank);
        Ok(ranked.into_iter().map(|(_, url)| url).take(3).collect())
    })
    .await;
    found.unwrap_or_default()
}

/// parsed feeds, kept for a while so playing through a show's episodes
/// doesn't re-download a multi-megabyte feed each time
fn feed_cache() -> &'static Mutex<HashMap<String, (Instant, Arc<Vec<FeedItem>>)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, Arc<Vec<FeedItem>>)>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

async fn feed_items(url: &str) -> Result<Arc<Vec<FeedItem>>, AppError> {
    const FRESH: Duration = Duration::from_secs(30 * 60);
    if let Some((at, items)) = feed_cache().lock().await.get(url) {
        if at.elapsed() < FRESH {
            return Ok(Arc::clone(items));
        }
    }
    let xml = http::client()
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("podcast feed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Network(format!("podcast feed: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Network(format!("podcast feed: {e}")))?;
    let items = Arc::new(
        tokio::task::spawn_blocking(move || feed::parse(&xml))
            .await
            .map_err(|e| AppError::Playback(format!("podcast feed: {e}")))?,
    );
    let mut cache = feed_cache().lock().await;
    if cache.len() >= 8 {
        cache.retain(|_, (at, _)| at.elapsed() < FRESH);
    }
    cache.insert(url.to_string(), (Instant::now(), Arc::clone(&items)));
    Ok(items)
}

async fn rss_pick(pool: &SqlitePool, ep: &EpisodeInfo) -> Option<Pick> {
    if ep.show.is_empty() {
        return None;
    }
    let date = ep.release_date.as_deref().and_then(feed::parse_iso_day);
    for url in feeds_for(pool, &ep.show, &ep.publisher).await {
        let items = match feed_items(&url).await {
            Ok(items) => items,
            Err(e) => {
                eprintln!("[episode] {url}: {e}");
                continue;
            }
        };
        if let Some(item) = feed::best_match(&items, &ep.name, date, ep.duration_ms) {
            eprintln!("[episode] \"{}\" -> rss {}", ep.name, item.url);
            return Some(Pick::Rss { url: item.url.clone(), mime: item.mime.clone() });
        }
    }
    None
}

async fn youtube_pick(ep: &EpisodeInfo) -> Option<(Pick, crate::youtube::ExtractedStream)> {
    let query = format!("{} {}", ep.show, ep.name);
    let rows = match crate::youtube::search::episodes(query.trim()).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("[episode] youtube search: {e}");
            return None;
        }
    };
    let close = |d: u64| ep.duration_ms == 0 || d.abs_diff(ep.duration_ms) <= 90_000.max(ep.duration_ms / 20);
    for row in rows.iter().filter(|r| feed::title_similarity(&ep.name, &r.title) >= 0.6).take(3) {
        if row.duration_ms.is_some_and(|d| !close(d)) {
            continue;
        }
        match crate::youtube::player::extract(&row.video_id).await {
            // the search row doesn't always say how long it is; the video does
            Ok(stream) if stream.duration_ms.is_none_or(close) => {
                eprintln!("[episode] \"{}\" -> youtube {}", ep.name, row.video_id);
                return Some((Pick::Youtube { video_id: row.video_id.clone() }, stream));
            }
            Ok(_) => {}
            Err(e) => eprintln!("[episode] youtube {}: {e}", row.video_id),
        }
    }
    None
}

fn from_stream(stream: crate::youtube::ExtractedStream, ep: &EpisodeInfo) -> Resolved {
    Resolved {
        url: stream.format.url,
        user_agent: Some(stream.user_agent),
        mime: Some(stream.format.mime_type),
        duration_ms: ep.duration_ms,
        via: "youtube",
    }
}

/// find somewhere to play episode `id` (bare base62 id) from
pub async fn resolve(pool: &SqlitePool, token: &str, id: &str) -> Result<Resolved, AppError> {
    let ep = info(pool, token, id).await?;
    let key = format!("spotify:episode:{id}");

    if let Some(pick) = cache::get_json::<Pick>(pool, &key, "episode-audio", 30 * DAY_MS).await {
        match pick {
            Pick::Rss { url, mime } => {
                return Ok(Resolved { url, user_agent: None, mime, duration_ms: ep.duration_ms, via: "rss" });
            }
            // youtube urls expire after a few hours, so only the video is kept
            Pick::Youtube { video_id } => match crate::youtube::player::extract(&video_id).await {
                Ok(stream) => return Ok(from_stream(stream, &ep)),
                Err(e) => eprintln!("[episode] cached youtube {video_id}: {e}"),
            },
        }
    }

    if let Some(pick) = rss_pick(pool, &ep).await {
        cache::put_json(pool, &key, "episode-audio", &pick).await;
        if let Pick::Rss { url, mime } = pick {
            return Ok(Resolved { url, user_agent: None, mime, duration_ms: ep.duration_ms, via: "rss" });
        }
    }
    if let Some((pick, stream)) = youtube_pick(&ep).await {
        cache::put_json(pool, &key, "episode-audio", &pick).await;
        return Ok(from_stream(stream, &ep));
    }

    let what = if ep.name.is_empty() { "This episode".to_string() } else { format!("\"{}\"", ep.name) };
    Err(AppError::NotFound(format!("{what} is only on Spotify, so it can't play from YouTube Music")))
}

/// forget where an episode was found (the file moved, the video went away)
pub async fn forget(pool: &SqlitePool, id: &str) {
    cache::put(pool, &format!("spotify:episode:{id}"), "episode-audio", b"null").await;
}

#[cfg(test)]
mod live_tests;
