//! what a podcast episode has beyond its audio: the read-along transcript
//! (spotify's synced subtitles) and, for video podcasts, a video preview.
//!
//! full video episodes stream as DRM'd segments through spotify's own video
//! player and can't be synced to librespot's audio, so what we can show is
//! the short unencrypted preview spotify serves alongside - looped, muted -
//! the same role a canvas plays for a song.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    internal::{cache, pathfinder, spclient},
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptLine {
    pub start_ms: i64,
    pub text:     String,
    /// set on the first line of each speaker turn, when spotify tags speakers
    pub speaker:  Option<String>,
    /// a chapter heading rather than speech
    pub heading:  bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub language: Option<String>,
    /// false for a static transcript with no timings
    pub synced:   bool,
    pub lines:    Vec<TranscriptLine>,
}

fn s(v: &Value, path: &[&str]) -> Option<String> {
    pathfinder::get_str(v, path).map(str::to_string)
}

pub(crate) fn transcript_from(v: &Value) -> Transcript {
    let mut lines = Vec::new();
    for sec in v.get("section").and_then(|x| x.as_array()).into_iter().flatten() {
        let start_ms = sec.get("startMs").and_then(|x| x.as_i64())
            .or_else(|| sec.get("startMs").and_then(|x| x.as_str()).and_then(|x| x.parse().ok()))
            .unwrap_or(0);
        if let Some(title) = s(sec, &["title", "title"]) {
            lines.push(TranscriptLine { start_ms, text: title, speaker: None, heading: true });
            continue;
        }
        let speaker = sec
            .get("speaker")
            .and_then(|sp| sp.get("speakers"))
            .and_then(|x| x.as_array())
            .and_then(|a| a.first())
            .and_then(|t| t.get("tag"))
            .and_then(|t| t.as_str())
            .map(str::to_string)
            .filter(|t| !t.is_empty());
        let text = s(sec, &["text", "sentence", "text"]).or_else(|| s(sec, &["fallback", "sentence", "text"]));
        let Some(text) = text else { continue };
        // the sentence's own start is finer than the section's when present
        let start_ms = sec
            .get("text")
            .and_then(|t| t.get("sentence"))
            .and_then(|t| t.get("startMs"))
            .and_then(|x| x.as_i64())
            .filter(|ms| *ms > 0)
            .unwrap_or(start_ms);
        lines.push(TranscriptLine { start_ms, text: text.trim().to_string(), speaker, heading: false });
    }
    let synced = lines.iter().any(|l| l.start_ms > 0);
    Transcript { language: s(v, &["language"]), synced, lines }
}

/// the synced transcript of an episode, or None when it has none
#[tauri::command]
pub async fn get_episode_transcript(app: AppHandle, id: String) -> Result<Option<Transcript>, AppError> {
    let id = spclient::uri_id(&id).to_string();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AppError::InvalidInput("bad episode id".into()));
    }
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:episode:{id}");
    cache::cached_json(&pool, &uri, "transcript", 7 * cache::DAY, || async {
        let endpoint = format!("/transcript-read-along/v2/episode/{id}?format=json&maxSentenceLength=120&excludeCC=true");
        match spclient::get_json_value(&app, &endpoint).await {
            Ok(v) => {
                let t = transcript_from(&v);
                Ok(if t.lines.is_empty() { None } else { Some(t) })
            }
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    })
    .await
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeMedia {
    /// the episode is a video podcast
    pub is_video:          bool,
    /// a short unencrypted preview clip to loop, when spotify has one
    pub video_preview_url: Option<String>,
    /// a still from the video (or the episode art)
    pub thumbnail_url:     Option<String>,
}

/// every string in a json tree, depth first
fn strings<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::String(s) => out.push(s),
        Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| strings(x, out)),
        _ => {}
    }
}

pub(crate) fn media_from(data: &Value) -> EpisodeMedia {
    let ep = pathfinder::get(data, &["episodeUnionV2"]).or_else(|| pathfinder::get(data, &["episode"])).unwrap_or(data);
    let media_types: Vec<String> = ep
        .get("mediaTypes")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|m| m.as_str().map(str::to_uppercase)).collect())
        .unwrap_or_default();
    let mut all = Vec::new();
    strings(ep, &mut all);
    // the preview is served as a plain mp4 from spotify's cdn
    let video_preview_url = all
        .iter()
        .find(|u| u.starts_with("https://") && (u.contains(".mp4") || u.contains("video/mp4")))
        .map(|u| u.to_string());
    let thumbnail_url = ["videoThumbnail", "videoPreviewThumbnail"]
        .iter()
        .find_map(|k| {
            let mut found = Vec::new();
            if let Some(v) = ep.get(*k) {
                strings(v, &mut found);
            }
            found.into_iter().find(|u| u.starts_with("https://")).map(str::to_string)
        });
    EpisodeMedia {
        is_video: media_types.iter().any(|m| m == "VIDEO") || video_preview_url.is_some(),
        video_preview_url,
        thumbnail_url,
    }
}

/// video info for an episode (pathfinder getEpisodeOrChapter)
#[tauri::command]
pub async fn get_episode_media(app: AppHandle, id: String) -> Result<EpisodeMedia, AppError> {
    let id = spclient::uri_id(&id).to_string();
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:episode:{id}");
    cache::cached_json(&pool, &uri, "episode-media", cache::DAY, || async {
        let data = pathfinder::query(&app, "getEpisodeOrChapter", serde_json::json!({ "uri": uri })).await?;
        Ok(media_from(&data))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_read_along() {
        let v: Value = serde_json::from_str(
            r#"{"language": "en", "section": [
                {"startMs": 0, "title": {"title": "Intro"}},
                {"startMs": 1200, "speaker": {"speakers": [{"tag": "Speaker 1"}]}, "text": {"sentence": {"startMs": 1250, "text": " Welcome back to the show. "}}},
                {"startMs": 4800, "text": {"sentence": {"startMs": 4800, "text": "Today we talk phones."}}},
                {"startMs": 9000, "musicTrack": {"uri": "spotify:track:x"}}
            ]}"#,
        )
        .unwrap();
        let t = transcript_from(&v);
        assert!(t.synced);
        assert_eq!(t.lines.len(), 3);
        assert!(t.lines[0].heading);
        assert_eq!(t.lines[1].start_ms, 1250);
        assert_eq!(t.lines[1].text, "Welcome back to the show.");
        assert_eq!(t.lines[1].speaker.as_deref(), Some("Speaker 1"));
        assert_eq!(t.lines[2].speaker, None);
    }

    #[test]
    fn finds_video_preview() {
        let v = serde_json::json!({"episodeUnionV2": {
            "mediaTypes": ["AUDIO", "VIDEO"],
            "previewPlayback": {"url": "https://video-fa.scdn.co/segments/abc/preview.mp4"},
            "videoThumbnail": {"image": {"sources": [{"url": "https://i.scdn.co/image/thumb"}]}}
        }});
        let m = media_from(&v);
        assert!(m.is_video);
        assert_eq!(m.video_preview_url.as_deref(), Some("https://video-fa.scdn.co/segments/abc/preview.mp4"));
        assert_eq!(m.thumbnail_url.as_deref(), Some("https://i.scdn.co/image/thumb"));
        assert!(!media_from(&serde_json::json!({"episodeUnionV2": {"mediaTypes": ["AUDIO"]}})).is_video);
    }
}
