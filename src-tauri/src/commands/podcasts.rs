//! podcasts: shows, episodes, the saved-shows library. all public web api -
//! the show/episode objects carry everything a page needs, including where you
//! stopped listening (`resume_point`, needs user-read-playback-position).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    commands::spotify::{tok, BASE},
    errors::AppError,
    internal::cache,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowItem {
    pub id:             String,
    pub name:           String,
    pub publisher:      String,
    pub image_url:      Option<String>,
    pub description:    Option<String>,
    pub total_episodes: Option<i64>,
    pub explicit:       bool,
    pub media_type:     Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeItem {
    pub id:                 String,
    pub name:               String,
    pub description:        Option<String>,
    pub image_url:          Option<String>,
    pub release_date:       Option<String>,
    pub duration_ms:        i64,
    pub explicit:           bool,
    pub is_playable:        bool,
    pub resume_position_ms: Option<i64>,
    pub fully_played:       bool,
    pub show_id:            Option<String>,
    pub show_name:          Option<String>,
    pub publisher:          Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowDetail {
    #[serde(flatten)]
    pub show:        ShowItem,
    pub html_description: Option<String>,
    pub languages:   Vec<String>,
    pub episodes:    Vec<EpisodeItem>,
    /// offset for the next page of episodes, None when that was the last
    pub next_offset: Option<i64>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string).filter(|x| !x.is_empty())
}

/// the largest artwork on offer. show/episode image lists aren't reliably
/// largest-first like album ones are, and taking [0] got 64px covers blown up
/// to tile size
pub(crate) fn first_image(v: &Value) -> Option<String> {
    v.get("images")?
        .as_array()?
        .iter()
        .enumerate()
        .filter_map(|(n, i)| Some((i.get("url")?.as_str()?, i.get("width").and_then(|w| w.as_i64()).unwrap_or(0), n)))
        // widest wins; with no widths given, keep spotify's own first pick
        .max_by_key(|(_, w, n)| (*w, std::cmp::Reverse(*n)))
        .map(|(u, _, _)| u.to_string())
}

pub(crate) fn show_from(v: &Value) -> Option<ShowItem> {
    Some(ShowItem {
        id: s(v, "id")?,
        name: s(v, "name").unwrap_or_default(),
        publisher: s(v, "publisher").unwrap_or_default(),
        image_url: first_image(v),
        description: s(v, "description"),
        total_episodes: v.get("total_episodes").and_then(|x| x.as_i64()),
        explicit: v.get("explicit").and_then(|x| x.as_bool()).unwrap_or(false),
        media_type: s(v, "media_type"),
    })
}

pub(crate) fn episode_from(v: &Value, show: Option<&ShowItem>) -> Option<EpisodeItem> {
    let embedded = v.get("show").and_then(show_from);
    let show = embedded.as_ref().or(show);
    let resume = v.get("resume_point");
    Some(EpisodeItem {
        id: s(v, "id")?,
        name: s(v, "name").unwrap_or_default(),
        description: s(v, "description"),
        image_url: first_image(v).or_else(|| show.and_then(|sh| sh.image_url.clone())),
        release_date: s(v, "release_date"),
        duration_ms: v.get("duration_ms").and_then(|x| x.as_i64()).unwrap_or(0),
        explicit: v.get("explicit").and_then(|x| x.as_bool()).unwrap_or(false),
        is_playable: v.get("is_playable").and_then(|x| x.as_bool()).unwrap_or(true),
        resume_position_ms: resume.and_then(|r| r.get("resume_position_ms")).and_then(|x| x.as_i64()),
        fully_played: resume
            .and_then(|r| r.get("fully_played"))
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        show_id: show.map(|sh| sh.id.clone()),
        show_name: show.map(|sh| sh.name.clone()),
        publisher: show.map(|sh| sh.publisher.clone()),
    })
}

fn episodes_page(page: &Value, show: Option<&ShowItem>) -> (Vec<EpisodeItem>, Option<i64>) {
    let items = page
        .get("items")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter(|e| !e.is_null()).filter_map(|e| episode_from(e, show)).collect())
        .unwrap_or_default();
    let next = if page.get("next").map(|n| !n.is_null()).unwrap_or(false) {
        let offset = page.get("offset").and_then(|x| x.as_i64()).unwrap_or(0);
        let limit = page.get("limit").and_then(|x| x.as_i64()).unwrap_or(50);
        Some(offset + limit)
    } else {
        None
    };
    (items, next)
}

#[tauri::command]
pub async fn get_show(app: AppHandle, id: String) -> Result<ShowDetail, AppError> {
    let token = tok(&app).await?;
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:show:{id}");
    // episode resume points change as you listen, so keep this short
    cache::cached_json(&pool, &uri, "show", 2 * 60_000, || async {
        let v: Value = crate::spotify::spotify_get(&token, &format!("{BASE}/shows/{id}?market=from_token")).await?;
        let show = show_from(&v).ok_or_else(|| AppError::NotFound(format!("show {id}")))?;
        let (episodes, next_offset) = v
            .get("episodes")
            .map(|p| episodes_page(p, Some(&show)))
            .unwrap_or_default();
        Ok(ShowDetail {
            html_description: s(&v, "html_description"),
            languages: v
                .get("languages")
                .and_then(|x| x.as_array())
                .map(|a| a.iter().filter_map(|l| l.as_str().map(str::to_string)).collect())
                .unwrap_or_default(),
            show,
            episodes,
            next_offset,
        })
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
pub struct EpisodePage {
    pub episodes:    Vec<EpisodeItem>,
    pub next_offset: Option<i64>,
}

/// further pages of a show's episodes (newest first)
#[tauri::command]
pub async fn get_show_episodes(app: AppHandle, id: String, offset: i64) -> Result<EpisodePage, AppError> {
    let token = tok(&app).await?;
    let page: Value = crate::spotify::spotify_get(
        &token,
        &format!("{BASE}/shows/{id}/episodes?market=from_token&limit=50&offset={}", offset.max(0)),
    )
    .await?;
    let (episodes, next_offset) = episodes_page(&page, None);
    Ok(EpisodePage { episodes, next_offset })
}

#[tauri::command]
pub async fn get_episode(app: AppHandle, id: String) -> Result<EpisodeItem, AppError> {
    let token = tok(&app).await?;
    let v: Value = crate::spotify::spotify_get(&token, &format!("{BASE}/episodes/{id}?market=from_token")).await?;
    episode_from(&v, None).ok_or_else(|| AppError::NotFound(format!("episode {id}")))
}

/// every show you follow, newest follow first
#[tauri::command]
pub async fn get_saved_shows(app: AppHandle) -> Result<Vec<ShowItem>, AppError> {
    let token = tok(&app).await?;
    let pool = app.state::<AppState>().db.clone();
    cache::cached_json(&pool, "spotify:me", "saved-shows", 10 * 60_000, || async {
        let mut out = Vec::new();
        let mut next = Some(format!("{BASE}/me/shows?limit=50"));
        while let Some(url) = next.take() {
            let page: Value = crate::spotify::spotify_get(&token, &url).await?;
            if let Some(items) = page.get("items").and_then(|x| x.as_array()) {
                out.extend(items.iter().filter_map(|it| it.get("show").and_then(show_from)));
            }
            next = s(&page, "next");
            if out.len() >= 500 {
                break;
            }
        }
        Ok(out)
    })
    .await
}

async fn invalidate_saved_shows(app: &AppHandle) {
    let pool = app.state::<AppState>().db.clone();
    let _ = sqlx::query("DELETE FROM extension_cache WHERE entity_uri = 'spotify:me' AND kind = 'saved-shows'")
        .execute(&pool)
        .await;
}

#[tauri::command]
pub async fn save_show(app: AppHandle, id: String) -> Result<(), AppError> {
    let token = tok(&app).await?;
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &format!("{BASE}/me/shows?ids={id}")).await?;
    invalidate_saved_shows(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn unsave_show(app: AppHandle, id: String) -> Result<(), AppError> {
    let token = tok(&app).await?;
    crate::spotify::spotify_write(&token, reqwest::Method::DELETE, &format!("{BASE}/me/shows?ids={id}")).await?;
    invalidate_saved_shows(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn is_show_saved(app: AppHandle, id: String) -> Result<bool, AppError> {
    let token = tok(&app).await?;
    let v: Vec<bool> = crate::spotify::spotify_get(&token, &format!("{BASE}/me/shows/contains?ids={id}")).await?;
    Ok(v.first().copied().unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_episode_with_resume_point() {
        let v: Value = serde_json::from_str(
            r#"{
              "id": "ep1", "name": "Pilot", "duration_ms": 3600000, "explicit": false,
              "release_date": "2024-02-01", "images": [{"url": "https://img/ep"}],
              "resume_point": {"fully_played": false, "resume_position_ms": 120000},
              "show": {"id": "sh1", "name": "The Show", "publisher": "Pub", "images": [{"url": "https://img/sh"}]}
            }"#,
        )
        .unwrap();
        let e = episode_from(&v, None).unwrap();
        assert_eq!(e.resume_position_ms, Some(120_000));
        assert_eq!(e.show_name.as_deref(), Some("The Show"));
        assert_eq!(e.image_url.as_deref(), Some("https://img/ep"));
    }

    #[test]
    fn picks_largest_art() {
        let v: Value = serde_json::from_str(
            r#"{"images": [{"url": "s", "width": 64}, {"url": "l", "width": 640}, {"url": "m", "width": 300}]}"#,
        )
        .unwrap();
        assert_eq!(first_image(&v).as_deref(), Some("l"));
        let v: Value = serde_json::from_str(r#"{"images": [{"url": "a"}, {"url": "b"}]}"#).unwrap();
        assert_eq!(first_image(&v).as_deref(), Some("a"));
    }

    #[test]
    fn paging_offsets() {
        let v: Value = serde_json::from_str(r#"{"items": [], "next": "x", "offset": 50, "limit": 50}"#).unwrap();
        assert_eq!(episodes_page(&v, None).1, Some(100));
        let v: Value = serde_json::from_str(r#"{"items": [], "next": null}"#).unwrap();
        assert_eq!(episodes_page(&v, None).1, None);
    }
}
