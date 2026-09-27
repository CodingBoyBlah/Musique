//! lrclib (https://lrclib.net) - the community line-level db, and our last
//! resort before giving up on a track.
//!
//! two endpoints: `/api/get` matches on exact tags + duration and is the one
//! we trust, `/api/search` is fuzzy and is where wrong masters come from. the
//! old code took the closest search hit whatever its duration, which is how a
//! 4:08 live cut ended up timing a 3:20 studio track. a lyric that drifts is
//! worse than no lyric at all, because everything upstream of us will happily
//! render it, so search hits now have to land inside `DUR_TOL_MS` or we return
//! nothing.

use serde::Deserialize;

use super::{text_matches, DUR_TOL_MS};
use crate::lyrics::parse::{parse_lrc, strip_noise};
use crate::lyrics::types::{Candidate, TrackRef};

/// lrclib asks third-party clients to identify themselves. it's a volunteer-run
/// community api, so it gets an honest name instead of the browser ua the
/// scrape-ish providers need
const LRCLIB_UA: &str = "Musique (Tauri desktop lyrics client)";

#[derive(Debug, Deserialize)]
struct Hit {
    #[serde(default)]
    instrumental:  bool,
    #[serde(default)]
    duration:      Option<f64>, // seconds, float
    #[serde(default, rename = "trackName")]
    track_name:    Option<String>,
    #[serde(default, rename = "plainLyrics")]
    plain_lyrics:  Option<String>,
    #[serde(default, rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

impl Hit {
    fn has_body(&self) -> bool {
        self.instrumental || self.synced_lyrics.is_some() || self.plain_lyrics.is_some()
    }
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    let hit = match exact(track).await {
        Some(hit) => hit,
        None => search(track).await?,
    };
    candidate(hit)
}

/// exact endpoint: lrclib keys on the tags a player would send, so it gets the
/// untouched title rather than the search-friendly cleaned one
async fn exact(track: &TrackRef) -> Option<Hit> {
    let mut url = url::Url::parse("https://lrclib.net/api/get").ok()?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("track_name", &track.name)
            .append_pair("artist_name", &track.artist)
            .append_pair("duration", &track.duration_sec().to_string());
        if !track.album.trim().is_empty() {
            q.append_pair("album_name", &track.album);
        }
    }

    let resp = crate::http::cookie_client()
        .get(url)
        .header("User-Agent", LRCLIB_UA)
        .send()
        .await
        .ok()?;
    // 404 (no exact match) and 5xx (busy, it says so in json) both just mean
    // "try the fuzzy endpoint"
    if !resp.status().is_success() {
        return None;
    }

    let hit: Hit = serde_json::from_str(&resp.text().await.ok()?).ok()?;
    hit.has_body().then_some(hit)
}

async fn search(track: &TrackRef) -> Option<Hit> {
    let mut url = url::Url::parse("https://lrclib.net/api/search").ok()?;
    url.query_pairs_mut()
        .append_pair("track_name", &track.clean_name())
        .append_pair("artist_name", &track.artist);

    let resp = crate::http::cookie_client()
        .get(url)
        .header("User-Agent", LRCLIB_UA)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let hits: Vec<Hit> = serde_json::from_str(&resp.text().await.ok()?).ok()?;
    best(hits, track)
}

/// pick the one hit that is provably the same recording. duration is a hard
/// gate, not a score - lrclib routinely lists the same title at 3:20, 3:22 and
/// 4:08, and only one of those is timed against what is actually playing
fn best(hits: Vec<Hit>, track: &TrackRef) -> Option<Hit> {
    let want = track.clean_name();
    let mut pick: Option<(bool, i64, Hit)> = None;

    for hit in hits {
        if !hit.has_body() {
            continue;
        }
        // no duration means nothing to verify against, so it can't be trusted
        let Some(dur) = hit.duration else { continue };
        let gap = ((dur * 1000.0).round() as i64 - track.duration_ms).abs();
        if gap > DUR_TOL_MS {
            continue;
        }
        // fuzzy search happily returns covers and samples of the query
        if let Some(name) = hit.track_name.as_deref() {
            if !text_matches(name, &want) {
                continue;
            }
        }

        let synced = hit.synced_lyrics.is_some();
        let better = match pick.as_ref() {
            // synced beats plain outright; past that, the closest duration wins
            Some((b_synced, b_gap, _)) => (synced, -gap) > (*b_synced, -*b_gap),
            None => true,
        };
        if better {
            pick = Some((synced, gap, hit));
        }
    }

    pick.map(|(_, _, hit)| hit)
}

fn candidate(hit: Hit) -> Option<Candidate> {
    if hit.instrumental {
        let mut cand = Candidate::new("lrclib", Vec::new());
        cand.instrumental = true;
        return Some(cand);
    }

    let lines = hit
        .synced_lyrics
        .as_deref()
        .map(|lrc| strip_noise(parse_lrc(lrc)))
        .unwrap_or_default();
    let plain = hit.plain_lyrics.filter(|p| !p.trim().is_empty());

    if lines.is_empty() && plain.is_none() {
        return None;
    }

    let mut cand = Candidate::new("lrclib", lines);
    cand.plain = plain;
    Some(cand)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> TrackRef {
        TrackRef {
            id:          "x".into(),
            name:        "Blinding Lights".into(),
            artist:      "The Weeknd".into(),
            album:       "After Hours".into(),
            isrc:        None,
            duration_ms: 200_040,
        }
    }

    fn hits(raw: &str) -> Vec<Hit> {
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn rejects_every_out_of_tolerance_hit() {
        let out = hits(
            r#"[{"trackName":"Blinding Lights","duration":248.0,"syncedLyrics":"[00:01.00]a"},
                {"trackName":"Blinding Lights","duration":202.0,"syncedLyrics":"[00:01.00]b"}]"#,
        );
        assert!(best(out, &track()).is_none());
    }

    #[test]
    fn prefers_synced_inside_tolerance() {
        let out = hits(
            r#"[{"trackName":"Blinding Lights","duration":200.0,"plainLyrics":"plain only"},
                {"trackName":"Blinding Lights","duration":200.9,"syncedLyrics":"[00:01.00]synced"}]"#,
        );
        let hit = best(out, &track()).expect("one hit is inside tolerance");
        assert_eq!(hit.synced_lyrics.as_deref(), Some("[00:01.00]synced"));
    }

    #[test]
    fn skips_a_different_song_at_the_same_length() {
        let out = hits(r#"[{"trackName":"Save Your Tears","duration":200.0,"syncedLyrics":"[00:01.00]a"}]"#);
        assert!(best(out, &track()).is_none());
    }

    #[test]
    fn instrumental_short_circuits() {
        let hit = hits(r#"[{"trackName":"Blinding Lights","duration":200.0,"instrumental":true}]"#)
            .pop()
            .unwrap();
        let cand = candidate(hit).unwrap();
        assert!(cand.instrumental);
        assert!(cand.lines.is_empty());
    }
}
