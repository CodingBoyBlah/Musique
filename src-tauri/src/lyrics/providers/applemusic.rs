//! apple music lyrics via the binimum reverse-engineered lyrics api.
//!
//! this is potentially the highest-quality provider in the system: when an ISRC
//! is available on the track reference, the lookup identifies the exact master
//! recording with zero version ambiguity and no fuzzy matching.
//!
//! endpoints:
//!   - isrc lookup: https://lyrics-api.binimum.org/?isrc=<ISRC>
//!   - metadata fallback: https://lyrics-api.binimum.org/?track=<title>&artist=<artist>&album=<album>&duration=<seconds>
//!
//! note: the binimum api expects `duration` in integer seconds (not milliseconds).
//! passing milliseconds causes the service to return 404 Not Found.

use serde::Deserialize;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::ttml::parse_ttml;
use crate::lyrics::types::{Candidate, TrackRef};

const API_BASE: &str = "https://lyrics-api.binimum.org/";

#[derive(Debug, Deserialize, Default)]
struct ApiResponse {
    #[allow(dead_code)]
    #[serde(default)]
    total:   usize,
    #[serde(default)]
    results: Vec<Hit>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Hit {
    #[serde(default)]
    pub id:          String,
    #[serde(default, alias = "trackName")]
    pub track_name:  String,
    #[allow(dead_code)]
    #[serde(default, alias = "artistName")]
    pub artist_name: String,
    #[allow(dead_code)]
    #[serde(default, alias = "albumName")]
    pub album_name:  String,
    /// duration in integer seconds from the api response
    #[serde(default)]
    pub duration:    f64,
    #[allow(dead_code)]
    #[serde(default)]
    pub isrc:        Option<String>,
    /// "word" or line-level timing type
    #[serde(default, alias = "timingType")]
    pub timing_type: String,
    #[serde(default, rename = "lyricsUrl", alias = "lyrics_url")]
    pub lyrics_url:  String,
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    // only the ISRC route identifies the exact recording; the title/artist
    // fallback is an ordinary fuzzy match and must stay subject to being shifted
    // onto the sync reference like any other
    let mut exact = false;
    let hit = match track.isrc.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(isrc) => match by_isrc(isrc).await {
            Some(hit) => {
                exact = true;
                Some(hit)
            }
            None => by_search(track).await,
        },
        None => by_search(track).await,
    }?;

    if hit.lyrics_url.is_empty() {
        return None;
    }

    let resp = crate::http::cookie_client()
        .get(&hit.lyrics_url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body = resp.text().await.ok()?;
    let lines = parse_ttml(&body);
    if lines.is_empty() {
        return None;
    }

    // note: do not apply the `<audio lyricOffset="..."/>` value found in
    // the TTML's iTunesMetadata. it is apple's spatial-audio metadata, not a
    // lyric sync correction, and applying it would shift correctly-timed lyrics.
    let mut cand = Candidate::new("applemusic", lines);
    cand.exact = exact;
    Some(cand)
}

/// isrc lookup pins the exact recording with zero fuzzy matching
async fn by_isrc(isrc: &str) -> Option<Hit> {
    let mut url = url::Url::parse(API_BASE).ok()?;
    url.query_pairs_mut().append_pair("isrc", isrc);

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
    let parsed: ApiResponse = serde_json::from_str(&body).ok()?;
    pick_isrc(parsed.results)
}

/// fallback title/artist/album/duration search when isrc is missing or unindexed
async fn by_search(track: &TrackRef) -> Option<Hit> {
    let clean = track.clean_name();
    let title = if clean.is_empty() { &track.name } else { &clean };
    if title.trim().is_empty() {
        return None;
    }

    // try first with album for tighter search results; fall back to bare
    // title/artist if album search returns no results
    if !track.album.trim().is_empty() {
        if let Some(hits) = search_request(title, &track.artist, &track.album, track.duration_sec()).await {
            if let Some(hit) = pick_search(hits, track) {
                return Some(hit);
            }
        }
    }

    let hits = search_request(title, &track.artist, "", track.duration_sec()).await?;
    pick_search(hits, track)
}

async fn search_request(title: &str, artist: &str, album: &str, dur_sec: i64) -> Option<Vec<Hit>> {
    let mut url = url::Url::parse(API_BASE).ok()?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("track", title);
        if !artist.trim().is_empty() {
            q.append_pair("artist", artist);
        }
        if !album.trim().is_empty() {
            q.append_pair("album", album);
        }
        if dur_sec > 0 {
            q.append_pair("duration", &dur_sec.to_string());
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
    let parsed: ApiResponse = serde_json::from_str(&body).ok()?;
    Some(parsed.results)
}

/// for an isrc search, the match is already exact: accept it, preferring word-timed
pub(crate) fn pick_isrc(results: Vec<Hit>) -> Option<Hit> {
    results
        .into_iter()
        .filter(|h| !h.lyrics_url.is_empty())
        .max_by_key(|h| (h.timing_type.eq_ignore_ascii_case("word"), !h.id.is_empty()))
}

/// gate fuzzy search hits strictly on duration and title agreement so this provider
/// never falls back to returning the wrong master
pub(crate) fn pick_search(results: Vec<Hit>, track: &TrackRef) -> Option<Hit> {
    let clean = track.clean_name();
    let mut best: Option<(bool, i64, Hit)> = None;

    for hit in results {
        if hit.lyrics_url.is_empty() {
            continue;
        }

        // binimum duration is in whole seconds; convert to ms for tolerance check
        let cand_ms = (hit.duration * 1000.0).round() as i64;
        if !duration_ok(cand_ms, track.duration_ms) {
            continue;
        }

        // title must match either raw or cleaned track name
        if !text_matches(&hit.track_name, &track.name) && !text_matches(&hit.track_name, &clean) {
            continue;
        }

        let is_word = hit.timing_type.eq_ignore_ascii_case("word");
        let gap = (cand_ms - track.duration_ms).abs();

        let better = match best.as_ref() {
            Some((b_word, b_gap, _)) => (is_word, -gap) > (*b_word, -*b_gap),
            None => true,
        };

        if better {
            best = Some((is_word, gap, hit));
        }
    }

    best.map(|(_, _, hit)| hit)
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
            isrc:        Some("USUG11904206".into()),
            duration_ms: 200_040,
        }
    }

    #[test]
    fn test_pick_isrc_prefers_word_timing() {
        let hits = vec![
            Hit {
                id:          "1".into(),
                track_name:  "Blinding Lights".into(),
                artist_name: "The Weeknd".into(),
                album_name:  "After Hours".into(),
                duration:    199.0,
                isrc:        Some("USUG11904206".into()),
                timing_type: "line".into(),
                lyrics_url:  "https://example.com/line.ttml".into(),
            },
            Hit {
                id:          "2".into(),
                track_name:  "Blinding Lights".into(),
                artist_name: "The Weeknd".into(),
                album_name:  "After Hours".into(),
                duration:    199.0,
                isrc:        Some("USUG11904206".into()),
                timing_type: "word".into(),
                lyrics_url:  "https://example.com/word.ttml".into(),
            },
        ];

        let chosen = pick_isrc(hits).expect("should pick a hit");
        assert_eq!(chosen.id, "2");
        assert_eq!(chosen.timing_type, "word");
    }

    #[test]
    fn test_pick_isrc_empty() {
        assert!(pick_isrc(Vec::new()).is_none());
    }

    #[test]
    fn test_pick_search_gating() {
        let track = sample_track();

        // wrong duration (> 1.5s difference)
        let bad_dur = vec![Hit {
            id:          "bad_dur".into(),
            track_name:  "Blinding Lights".into(),
            artist_name: "The Weeknd".into(),
            album_name:  "After Hours".into(),
            duration:    190.0,
            isrc:        None,
            timing_type: "word".into(),
            lyrics_url:  "https://example.com/bad_dur.ttml".into(),
        }];
        assert!(pick_search(bad_dur, &track).is_none());

        // wrong title
        let bad_title = vec![Hit {
            id:          "bad_title".into(),
            track_name:  "Save Your Tears".into(),
            artist_name: "The Weeknd".into(),
            album_name:  "After Hours".into(),
            duration:    200.0,
            isrc:        None,
            timing_type: "word".into(),
            lyrics_url:  "https://example.com/bad_title.ttml".into(),
        }];
        assert!(pick_search(bad_title, &track).is_none());

        // valid match (199s vs 200.04s is ~1s difference <= 1.5s tolerance)
        let good = vec![Hit {
            id:          "good".into(),
            track_name:  "Blinding Lights".into(),
            artist_name: "The Weeknd".into(),
            album_name:  "After Hours".into(),
            duration:    199.0,
            isrc:        None,
            timing_type: "word".into(),
            lyrics_url:  "https://example.com/good.ttml".into(),
        }];
        let chosen = pick_search(good, &track).expect("should match valid hit");
        assert_eq!(chosen.id, "good");
    }

    #[test]
    fn test_deserialize_live_shape() {
        let json = r#"{"total":1,"source":"HIT-LOCAL-LIBSQL","results":[{"id":"69afd2d1003379914387","track_name":"Blinding Lights","artist_name":"The Weeknd","album_name":"After Hours (Deluxe Video Album)","duration":199,"isrc":"USUG11904206","timing_type":"word","lyricsUrl":"https://lyrics-storage.binimum.org/USUG11904206.ttml"}]}"#;
        let resp: ApiResponse = serde_json::from_str(json).expect("valid json");
        assert_eq!(resp.total, 1);
        assert_eq!(resp.results.len(), 1);
        let hit = &resp.results[0];
        assert_eq!(hit.track_name, "Blinding Lights");
        assert_eq!(hit.artist_name, "The Weeknd");
        assert_eq!(hit.duration, 199.0);
        assert_eq!(hit.isrc.as_deref(), Some("USUG11904206"));
        assert_eq!(hit.timing_type, "word");
        assert_eq!(hit.lyrics_url, "https://lyrics-storage.binimum.org/USUG11904206.ttml");
    }
}
