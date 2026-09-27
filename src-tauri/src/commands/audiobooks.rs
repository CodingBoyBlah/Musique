//! audiobooks + their chapters. chapter uris are `spotify:episode:<id>`, so
//! they're shaped as EpisodeItem and ride the podcast playback path; whether a
//! chapter actually plays depends on the account (premium listening hours or a
//! purchase), which spotify reports per chapter as `is_playable`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    commands::{
        podcasts::EpisodeItem,
        spotify::{tok, BASE},
    },
    errors::AppError,
    internal::cache,
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiobookItem {
    pub id:             String,
    pub name:           String,
    pub authors:        Vec<String>,
    pub narrators:      Vec<String>,
    pub publisher:      Option<String>,
    pub image_url:      Option<String>,
    pub description:    Option<String>,
    pub total_chapters: Option<i64>,
    pub explicit:       bool,
    pub edition:        Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiobookDetail {
    #[serde(flatten)]
    pub book:        AudiobookItem,
    pub languages:   Vec<String>,
    pub copyrights:  Vec<String>,
    pub chapters:    Vec<EpisodeItem>,
    pub next_offset: Option<i64>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string).filter(|x| !x.is_empty())
}

fn names(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|p| s(p, "name")).collect())
        .unwrap_or_default()
}

fn first_image(v: &Value) -> Option<String> {
    v.get("images")?.as_array()?.first()?.get("url")?.as_str().map(str::to_string)
}

pub(crate) fn audiobook_from(v: &Value) -> Option<AudiobookItem> {
    Some(AudiobookItem {
        id: s(v, "id")?,
        name: s(v, "name").unwrap_or_default(),
        authors: names(v, "authors"),
        narrators: names(v, "narrators"),
        publisher: s(v, "publisher"),
        image_url: first_image(v),
        description: s(v, "description"),
        total_chapters: v.get("total_chapters").and_then(|x| x.as_i64()),
        explicit: v.get("explicit").and_then(|x| x.as_bool()).unwrap_or(false),
        edition: s(v, "edition"),
    })
}

fn chapter_from(v: &Value, book: &AudiobookItem) -> Option<EpisodeItem> {
    let resume = v.get("resume_point");
    Some(EpisodeItem {
        id: s(v, "id")?,
        name: s(v, "name").unwrap_or_default(),
        description: s(v, "description"),
        image_url: first_image(v).or_else(|| book.image_url.clone()),
        release_date: s(v, "release_date"),
        duration_ms: v.get("duration_ms").and_then(|x| x.as_i64()).unwrap_or(0),
        explicit: v.get("explicit").and_then(|x| x.as_bool()).unwrap_or(false),
        is_playable: v.get("is_playable").and_then(|x| x.as_bool()).unwrap_or(true),
        resume_position_ms: resume.and_then(|r| r.get("resume_position_ms")).and_then(|x| x.as_i64()),
        fully_played: resume.and_then(|r| r.get("fully_played")).and_then(|x| x.as_bool()).unwrap_or(false),
        show_id: Some(book.id.clone()),
        show_name: Some(book.name.clone()),
        publisher: Some(book.authors.join(", ")).filter(|a| !a.is_empty()).or_else(|| book.publisher.clone()),
    })
}

fn chapters_page(page: &Value, book: &AudiobookItem) -> (Vec<EpisodeItem>, Option<i64>) {
    let items = page
        .get("items")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter(|c| !c.is_null()).filter_map(|c| chapter_from(c, book)).collect())
        .unwrap_or_default();
    let next = if page.get("next").map(|n| !n.is_null()).unwrap_or(false) {
        Some(page.get("offset").and_then(|x| x.as_i64()).unwrap_or(0) + page.get("limit").and_then(|x| x.as_i64()).unwrap_or(50))
    } else {
        None
    };
    (items, next)
}

#[tauri::command]
pub async fn get_audiobook(app: AppHandle, id: String) -> Result<AudiobookDetail, AppError> {
    let token = tok(&app).await?;
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:audiobook:{id}");
    cache::cached_json(&pool, &uri, "audiobook", 2 * 60_000, || async {
        let v: Value = crate::spotify::spotify_get(&token, &format!("{BASE}/audiobooks/{id}?market=from_token")).await?;
        let book = audiobook_from(&v).ok_or_else(|| AppError::NotFound(format!("audiobook {id}")))?;
        let (chapters, next_offset) = v.get("chapters").map(|p| chapters_page(p, &book)).unwrap_or_default();
        Ok(AudiobookDetail {
            languages: v
                .get("languages")
                .and_then(|x| x.as_array())
                .map(|a| a.iter().filter_map(|l| l.as_str().map(str::to_string)).collect())
                .unwrap_or_default(),
            copyrights: v
                .get("copyrights")
                .and_then(|x| x.as_array())
                .map(|a| a.iter().filter_map(|c| s(c, "text")).collect())
                .unwrap_or_default(),
            book,
            chapters,
            next_offset,
        })
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
pub struct ChapterPage {
    pub chapters:    Vec<EpisodeItem>,
    pub next_offset: Option<i64>,
}

#[tauri::command]
pub async fn get_audiobook_chapters(app: AppHandle, id: String, offset: i64) -> Result<ChapterPage, AppError> {
    let detail = get_audiobook(app.clone(), id.clone()).await?;
    let token = tok(&app).await?;
    let page: Value = crate::spotify::spotify_get(
        &token,
        &format!("{BASE}/audiobooks/{id}/chapters?market=from_token&limit=50&offset={}", offset.max(0)),
    )
    .await?;
    let (chapters, next_offset) = chapters_page(&page, &detail.book);
    Ok(ChapterPage { chapters, next_offset })
}

#[tauri::command]
pub async fn get_saved_audiobooks(app: AppHandle) -> Result<Vec<AudiobookItem>, AppError> {
    let token = tok(&app).await?;
    let pool = app.state::<AppState>().db.clone();
    cache::cached_json(&pool, "spotify:me", "saved-audiobooks", 10 * 60_000, || async {
        let mut out = Vec::new();
        let mut next = Some(format!("{BASE}/me/audiobooks?limit=50"));
        while let Some(url) = next.take() {
            let page: Value = crate::spotify::spotify_get(&token, &url).await?;
            if let Some(items) = page.get("items").and_then(|x| x.as_array()) {
                // /me/audiobooks items are the audiobook itself (no wrapper)
                out.extend(items.iter().filter_map(|it| audiobook_from(it.get("audiobook").unwrap_or(it))));
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

async fn invalidate_saved(app: &AppHandle) {
    let pool = app.state::<AppState>().db.clone();
    let _ = sqlx::query("DELETE FROM extension_cache WHERE entity_uri = 'spotify:me' AND kind = 'saved-audiobooks'")
        .execute(&pool)
        .await;
}

#[tauri::command]
pub async fn save_audiobook(app: AppHandle, id: String) -> Result<(), AppError> {
    let token = tok(&app).await?;
    crate::spotify::spotify_write(&token, reqwest::Method::PUT, &format!("{BASE}/me/audiobooks?ids={id}")).await?;
    invalidate_saved(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn unsave_audiobook(app: AppHandle, id: String) -> Result<(), AppError> {
    let token = tok(&app).await?;
    crate::spotify::spotify_write(&token, reqwest::Method::DELETE, &format!("{BASE}/me/audiobooks?ids={id}")).await?;
    invalidate_saved(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn is_audiobook_saved(app: AppHandle, id: String) -> Result<bool, AppError> {
    let token = tok(&app).await?;
    let v: Vec<bool> = crate::spotify::spotify_get(&token, &format!("{BASE}/me/audiobooks/contains?ids={id}")).await?;
    Ok(v.first().copied().unwrap_or(false))
}

/// audiobooks only exist in some markets, and asking for them where they don't
/// can fail the whole search - so they're a separate, best-effort request
pub(crate) async fn search_audiobooks(token: &str, query: &str) -> Vec<AudiobookItem> {
    let mut url = url::Url::parse(&format!("{BASE}/search")).unwrap();
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("type", "audiobook")
        .append_pair("market", "from_token")
        .append_pair("limit", "20");
    match crate::spotify::spotify_get::<Value>(token, url.as_str()).await {
        Ok(v) => v
            .get("audiobooks")
            .and_then(|p| p.get("items"))
            .and_then(|i| i.as_array())
            .map(|a| a.iter().filter(|b| !b.is_null()).filter_map(audiobook_from).collect())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapters_carry_the_book() {
        let book: Value = serde_json::from_str(
            r#"{"id": "b1", "name": "Dune", "authors": [{"name": "Frank Herbert"}],
                "narrators": [{"name": "Scott Brick"}], "images": [{"url": "https://img/b"}]}"#,
        )
        .unwrap();
        let book = audiobook_from(&book).unwrap();
        assert_eq!(book.authors, vec!["Frank Herbert"]);
        let ch: Value = serde_json::from_str(r#"{"id": "c1", "name": "Chapter 1", "duration_ms": 600000}"#).unwrap();
        let c = chapter_from(&ch, &book).unwrap();
        assert_eq!(c.show_id.as_deref(), Some("b1"));
        assert_eq!(c.publisher.as_deref(), Some("Frank Herbert"));
        assert_eq!(c.image_url.as_deref(), Some("https://img/b"));
    }
}
