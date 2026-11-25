//! Apple Music's own lyrics for the catalog song being played, read from amp-api with the
//! signed-in account (`/songs/{id}/syllable-lyrics`, then `/songs/{id}/lyrics`).
//!
//! Only while Apple Music is the provider: the document belongs to the exact song id the
//! player streams, so its timings are on that stream's own clock. The binimum lookup
//! (`applemusic`) keys by ISRC instead and can hand back a sibling release's document.

use serde_json::Value;

use crate::apple::client::{self, AppleClient};
use crate::lyrics::ttml::{parse_plain_ttml, parse_ttml};
use crate::lyrics::types::{Candidate, TrackRef};

/// The lyrics of `track` when it is an Apple catalog song and Apple Music is signed in.
pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    if !crate::apple::is_active() || crate::apple::wire::is_library_id(&track.id) {
        return None;
    }
    let c = client::current().await.ok()?;
    fetch_song(&c, &track.id).await
}

/// Syllable (word-timed) lyrics first, then line-timed ones.
pub async fn fetch_song(c: &AppleClient, id: &str) -> Option<Candidate> {
    for kind in ["syllable-lyrics", "lyrics"] {
        let path = c.catalog_path(&format!("/songs/{}/{kind}", client::encode_segment(id)));
        let Ok(body) = c.get(&path, &[]).await else { continue };
        if let Some(cand) = from_body(&body) {
            return Some(cand);
        }
    }
    None
}

fn from_body(body: &str) -> Option<Candidate> {
    let answer: Value = serde_json::from_str(body).ok()?;
    let ttml = answer.pointer("/data/0/attributes/ttml").and_then(Value::as_str)?;
    let lines = parse_ttml(ttml);
    let plain = if lines.is_empty() { parse_plain_ttml(ttml) } else { None };
    if lines.is_empty() && plain.is_none() {
        return None;
    }
    // same source name as the binimum document: it is the same Apple TTML, and every
    // ranking, upgrade and cache rule already treats "applemusic" as the canonical text
    let mut cand = Candidate::new("applemusic", lines);
    cand.plain = plain;
    cand.exact = true;
    Some(cand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_ttml_out_of_an_amp_api_answer() {
        let ttml = r#"<tt xmlns="http://www.w3.org/ns/ttml" xmlns:itunes="http://music.apple.com/lyric-ttml-internal" itunes:timing="Line"><body><div><p begin="1.500" end="3.000">Hello there</p><p begin="3.250" end="5.000">Second line</p></div></body></tt>"#;
        let body = serde_json::json!({ "data": [{ "id": "1", "type": "lyrics", "attributes": { "ttml": ttml } }] }).to_string();
        let cand = from_body(&body).expect("a candidate");
        assert_eq!(cand.source, "applemusic");
        assert!(cand.exact);
        assert_eq!(cand.lines.len(), 2);
        assert_eq!(cand.lines[0].time_ms, 1500);
        assert_eq!(cand.lines[1].time_ms, 3250);
    }

    #[test]
    fn an_answer_without_ttml_is_no_candidate() {
        assert!(from_body(r#"{"data":[]}"#).is_none());
        assert!(from_body("not json").is_none());
    }
}
