//! betterlyrics apple music ttml provider.
//!
//! fetches apple-music-style TTML documents from the boidu reverse-engineered
//! api. this serves as the fallback for tracks that the isrc-keyed apple music
//! provider misses, matching on title, artist, album, and duration.
//!
//! endpoint: https://lyrics-api.boidu.dev/getLyrics
//!   params: s=<title> a=<artist> d=<duration> al=<album>
//!
//! note: `d` must be integer seconds (`track.duration_sec()`). passing milliseconds
//! or uncached duration keys causes the service to return HTTP 401 Unauthorized.

use serde::Deserialize;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::ttml::parse_ttml;
use crate::lyrics::types::{Candidate, TrackRef};

const API_BASE: &str = "https://lyrics-api.boidu.dev/getLyrics";

#[derive(Debug, Deserialize, Default)]
pub(crate) struct Response {
    /// escaped apple-music-style TTML document
    #[serde(default)]
    pub ttml:        Option<String>,
    #[serde(default)]
    pub duration:    Option<f64>,
    #[serde(default, alias = "track_name", alias = "trackName")]
    pub title:       Option<String>,
    #[allow(dead_code)]
    #[serde(default, alias = "artist_name", alias = "artistName")]
    pub artist:      Option<String>,
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    if track.name.trim().is_empty() || track.artist.trim().is_empty() {
        return None;
    }

    // try first with album tag for tighter matching; if that fails (e.g. 401 on
    // an uncached album variant or name discrepancy), retry without album
    let resp = if !track.album.trim().is_empty() {
        match query(track, true).await {
            Some(r) => Some(r),
            None => query(track, false).await,
        }
    } else {
        query(track, false).await
    }?;

    if !validate(&resp, track) {
        return None;
    }

    let ttml = resp.ttml.as_deref()?;
    let lines = parse_ttml(ttml);
    if lines.is_empty() {
        return None;
    }

    Some(Candidate::new("betterlyrics", lines))
}

async fn query(track: &TrackRef, with_album: bool) -> Option<Response> {
    let mut url = url::Url::parse(API_BASE).ok()?;
    let clean = track.clean_name();
    let title = if clean.is_empty() { &track.name } else { &clean };

    {
        let mut q = url.query_pairs_mut();
        q.append_pair("s", title)
            .append_pair("a", &track.artist)
            .append_pair("d", &track.duration_sec().to_string());
        if with_album && !track.album.trim().is_empty() {
            q.append_pair("al", &track.album);
        }
    }

    let resp = crate::http::cookie_client()
        .get(url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body = resp.text().await.ok()?;
    let parsed: Response = serde_json::from_str(&body).ok()?;
    if parsed.ttml.is_none() {
        return None;
    }
    Some(parsed)
}

/// gate metadata if returned by the api. when boidu returns bare `{"ttml": "..."}`
/// without track/duration fields, accept it: the downstream alignment stage
/// validates candidate timestamps against the sync reference.
pub(crate) fn validate(resp: &Response, track: &TrackRef) -> bool {
    if let Some(dur) = resp.duration {
        let dur_ms = if dur > 10_000.0 {
            dur.round() as i64
        } else {
            (dur * 1000.0).round() as i64
        };
        if !duration_ok(dur_ms, track.duration_ms) {
            return false;
        }
    }

    if let Some(ref title) = resp.title {
        let clean = track.clean_name();
        if !text_matches(title, &track.name) && !text_matches(title, &clean) {
            return false;
        }
    }

    if let Some(ref artist) = resp.artist {
        if !text_matches(artist, &track.artist) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track() -> TrackRef {
        TrackRef {
            id:          "test_id".into(),
            name:        "Blinding Lights".into(),
            artist:      "The Weeknd".into(),
            album:       "After Hours".into(),
            isrc:        None,
            duration_ms: 200_040,
        }
    }

    #[test]
    fn test_deserialize_live_shape() {
        let json = r#"{"ttml":"<tt xmlns=\"http://www.w3.org/ns/ttml\"><head></head><body></body></tt>"}"#;
        let resp: Response = serde_json::from_str(json).expect("valid json");
        assert!(resp.ttml.is_some());
        assert!(resp.duration.is_none());
        assert!(resp.title.is_none());
    }

    #[test]
    fn test_deserialize_tolerates_extra_fields() {
        let json = r#"{"ttml":"<tt></tt>","duration":200.0,"title":"Blinding Lights","extra":123,"nested":{"a":true}}"#;
        let resp: Response = serde_json::from_str(json).expect("tolerates extra fields");
        assert_eq!(resp.duration, Some(200.0));
        assert_eq!(resp.title.as_deref(), Some("Blinding Lights"));
    }

    #[test]
    fn test_validate_duration_gate() {
        let track = sample_track();

        let mut resp = Response::default();
        resp.duration = Some(190.0); // 10s gap > 1.5s tolerance
        assert!(!validate(&resp, &track));

        resp.duration = Some(200.0); // within tolerance
        assert!(validate(&resp, &track));

        resp.duration = Some(200_040.0); // in ms
        assert!(validate(&resp, &track));
    }

    #[test]
    fn test_validate_title_gate() {
        let track = sample_track();

        let mut resp = Response::default();
        resp.title = Some("Totally Different Song".into());
        assert!(!validate(&resp, &track));

        resp.title = Some("Blinding Lights".into());
        assert!(validate(&resp, &track));
    }

    #[test]
    fn test_validate_bare_response_accepted() {
        let track = sample_track();
        let resp = Response {
            ttml:     Some("<tt></tt>".into()),
            duration: None,
            title:    None,
            artist:   None,
        };
        assert!(validate(&resp, &track));
    }
}
