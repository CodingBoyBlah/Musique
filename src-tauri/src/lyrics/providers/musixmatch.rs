//! musixmatch - the biggest karaoke catalogue and our only large source of
//! real per-word timing.
//!
//! one `macro.subtitles.get` call answers three questions at once: word-level
//! (`track.richsync.get`), line-level (`track.subtitles.get`) and plain text
//! (`track.lyrics.get`). the old code read only the richsync block and threw
//! the other two away, which meant a track without richsync counted as a miss
//! even though musixmatch had perfectly good synced lines sitting in the same
//! response. we now take the richest body that survives the match gate.
//!
//! the client MUST have a cookie store: `token.get` sets the `x-mxm-*` cookies
//! that the macro call needs, otherwise it 401s "renew".

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::Mutex;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::parse::{parse_lrc, parse_richsync, strip_noise};
use crate::lyrics::types::{Candidate, TrackRef};

const APP_ID: &str = "web-desktop-app-v1.0";
const TOKEN_URL: &str = "https://apic-desktop.musixmatch.com/ws/1.1/token.get";
const MACRO_URL: &str = "https://apic-desktop.musixmatch.com/ws/1.1/macro.subtitles.get";

/// musixmatch tokens stay valid far longer than this; 10 minutes just bounds
/// how long we keep using one that got revoked
const TOKEN_TTL: Duration = Duration::from_secs(600);

static TOKEN: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    let token = token().await?;
    let isrc = track.isrc.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let mut url = url::Url::parse(MACRO_URL).ok()?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("format", "json")
            .append_pair("namespace", "lyrics_richsynced")
            .append_pair("subtitle_format", "mxm")
            .append_pair("app_id", APP_ID)
            .append_pair("usertoken", &token)
            .append_pair("q_track", &track.clean_name())
            .append_pair("q_artist", &track.artist)
            .append_pair("q_duration", &track.duration_sec().to_string());
        // the isrc pins the exact recording, so the matcher can't hand us a
        // re-record / remaster / live cut whose words drift against the audio.
        // this is the single strongest fix for version mismatch
        if let Some(isrc) = isrc {
            q.append_pair("track_isrc", isrc);
        }
    }

    let body = crate::http::cookie_client()
        .get(url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let v: Value = serde_json::from_str(&body).ok()?;
    let calls = v.get("message")?.get("body")?.get("macro_calls")?;

    let matcher = call_body(calls, "matcher.track.get").and_then(|b| b.get("track"));
    let lyrics = call_body(calls, "track.lyrics.get").and_then(|b| b.get("lyrics"));
    let subtitle =
        call_body(calls, "track.subtitles.get").and_then(|b| b.pointer("/subtitle_list/0/subtitle"));

    // richsync: real sung per-word timing, the whole reason to come here
    let rich = call_body(calls, "track.richsync.get")
        .and_then(|b| b.pointer("/richsync/richsync_body"))
        .and_then(Value::as_str)
        .map(parse_richsync)
        .unwrap_or_default();

    // line-level lrc from the same response, free. most of the catalogue has
    // this even when it has no richsync
    let subs = subtitle
        .and_then(|s| s.get("subtitle_body"))
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(parse_lrc)
        .unwrap_or_default();

    // matcher.track.get sometimes omits track_length; the subtitle knows the
    // length of the recording it was timed against, which is just as good a
    // verification key
    let alt_len_sec = subtitle
        .and_then(|s| s.get("subtitle_length"))
        .and_then(loose_i64)
        .unwrap_or(0);

    let verified = matcher.map(|m| verify(m, track, isrc, alt_len_sec));
    if verified == Some(false) {
        return None; // different master - drifting lyrics are worse than none
    }

    // instrumental is authoritative: there is nothing to show and the ui says
    // so instead of spinning through the rest of the chain
    if flag(lyrics, "instrumental") || flag(matcher, "instrumental") {
        let mut cand = Candidate::new("musixmatch", Vec::new());
        cand.instrumental = true;
        return Some(cand);
    }

    // `restricted` means the body we got back is a placeholder, not the words
    let plain = lyrics
        .filter(|l| !flag(Some(*l), "restricted"))
        .and_then(|l| l.get("lyrics_body"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let lines = strip_noise(if rich.is_empty() { subs } else { rich });

    if lines.is_empty() && plain.is_none() {
        return None;
    }
    // with no matcher block there is nothing to verify against, so we only
    // trust a timed body - a bare plain text could belong to any recording
    if verified.is_none() && lines.is_empty() {
        return None;
    }

    let mut cand = Candidate::new("musixmatch", lines);
    cand.plain = plain;
    Some(cand)
}

/// every entry in `macro_calls` wraps its payload in the same
/// `message.body` envelope as a standalone api call
fn call_body<'a>(calls: &'a Value, name: &str) -> Option<&'a Value> {
    calls.get(name)?.get("message")?.get("body")
}

/// cached user token. a track change fans out to every provider at once and
/// each one used to pay its own `token.get`; the lock is held across the fetch
/// on purpose so a burst issues one request, not four
async fn token() -> Option<String> {
    let mut slot = TOKEN.get_or_init(|| Mutex::new(None)).lock().await;

    if let Some((tok, at)) = slot.as_ref() {
        if at.elapsed() < TOKEN_TTL {
            return Some(tok.clone());
        }
    }

    let body = crate::http::cookie_client()
        .get(TOKEN_URL)
        .query(&[("app_id", APP_ID), ("format", "json")])
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let v: Value = serde_json::from_str(&body).ok()?;
    let tok = v.pointer("/message/body/user_token").and_then(Value::as_str)?;
    if !usable_token(tok) {
        *slot = None;
        return None;
    }

    *slot = Some((tok.to_string(), Instant::now()));
    Some(tok.to_string())
}

/// `""` and the `Upgrade...` sentinel mean no token was issued. an all-zero
/// token is musixmatch's blocked state and is the nastiest of the three: it
/// answers 200 with a canned unrelated track and obfuscated gibberish lyrics,
/// so it has to be caught here or we'd cache nonsense as a real hit
fn usable_token(tok: &str) -> bool {
    !tok.is_empty() && !tok.starts_with("Upgrade") && tok.bytes().any(|b| b != b'0')
}

/// is this really the recording we asked about? matching a different master is
/// the single biggest cause of lyrics that look right but drift
fn verify(m: &Value, track: &TrackRef, sent_isrc: Option<&str>, alt_len_sec: i64) -> bool {
    // isrc first when we sent one: if the answer names isrcs and ours isn't
    // among them it's a different recording, however well the title reads
    if let Some(want) = sent_isrc {
        let got = isrcs(m);
        if !got.is_empty() {
            return got.iter().any(|g| g.eq_ignore_ascii_case(want));
        }
    }

    let title_ok = m
        .get("track_name")
        .and_then(Value::as_str)
        .map(|n| text_matches(n, &track.clean_name()))
        .unwrap_or(false);
    if !title_ok {
        return false;
    }

    let len_sec = m.get("track_length").and_then(loose_i64).filter(|s| *s > 0).unwrap_or(alt_len_sec);
    if len_sec > 0 {
        return duration_ok(len_sec * 1000, track.duration_ms);
    }

    // no length anywhere: fall back to the artist agreeing too. q_duration was
    // part of the query, so the matcher already weighed the length itself
    m.get("artist_name")
        .and_then(Value::as_str)
        .map(|a| text_matches(a, &track.artist))
        .unwrap_or(false)
}

/// isrcs off a matcher track, which spells them as a bare `track_isrc`, a list
/// of strings, or a list of `{ "track_isrc": ... }` objects depending on the
/// endpoint that filled the block
fn isrcs(m: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    if let Some(s) = m.get("track_isrc").and_then(Value::as_str) {
        if !s.trim().is_empty() {
            out.push(s.trim().to_string());
        }
    }
    if let Some(list) = m.get("track_isrc_list").and_then(Value::as_array) {
        for item in list {
            let s = item
                .as_str()
                .or_else(|| item.get("track_isrc").and_then(Value::as_str))
                .unwrap_or("");
            if !s.trim().is_empty() {
                out.push(s.trim().to_string());
            }
        }
    }
    out
}

/// musixmatch is inconsistent about numbers vs numeric strings
fn loose_i64(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)).or_else(|| v.as_str()?.trim().parse().ok())
}

/// its booleans are `1`/`0` ints, but some blocks use real booleans
fn flag(obj: Option<&Value>, key: &str) -> bool {
    obj.and_then(|o| o.get(key))
        .map(|v| v.as_bool().unwrap_or(false) || loose_i64(v).unwrap_or(0) != 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> TrackRef {
        TrackRef {
            id:          "x".into(),
            name:        "Blinding Lights - Remastered".into(),
            artist:      "The Weeknd".into(),
            album:       "After Hours".into(),
            isrc:        None,
            duration_ms: 200_040,
        }
    }

    #[test]
    fn rejects_blocked_tokens() {
        assert!(usable_token("abc123"));
        assert!(!usable_token(""));
        assert!(!usable_token("UpgradeOnlyUponRequest"));
        assert!(!usable_token("0000000000000000000000"));
    }

    #[test]
    fn verifies_by_duration_and_title() {
        let ok = serde_json::json!({ "track_name": "Blinding Lights", "artist_name": "The Weeknd", "track_length": 200 });
        assert!(verify(&ok, &track(), None, 0));

        // right title, different master
        let long = serde_json::json!({ "track_name": "Blinding Lights", "track_length": 248 });
        assert!(!verify(&long, &track(), None, 0));

        // musixmatch's blocked-state canned answer
        let wrong = serde_json::json!({ "track_name": "NOKIA", "artist_name": "Drake" });
        assert!(!verify(&wrong, &track(), None, 0));
    }

    #[test]
    fn isrc_overrides_everything() {
        let m = serde_json::json!({
            "track_name": "Something Else",
            "track_length": 999,
            "track_isrc_list": [{ "track_isrc": "USUG11904206" }],
        });
        assert!(verify(&m, &track(), Some("usug11904206"), 0));
        assert!(!verify(&m, &track(), Some("GBAYE0601498"), 0));
    }

    #[test]
    fn falls_back_to_subtitle_length() {
        let m = serde_json::json!({ "track_name": "Blinding Lights", "artist_name": "The Weeknd" });
        assert!(verify(&m, &track(), None, 200));
        assert!(!verify(&m, &track(), None, 240));
    }

    #[test]
    fn reads_string_numbers_and_int_flags() {
        assert_eq!(loose_i64(&serde_json::json!("200")), Some(200));
        assert_eq!(loose_i64(&serde_json::json!(200.4)), Some(200));
        assert!(flag(Some(&serde_json::json!({ "instrumental": 1 })), "instrumental"));
        assert!(!flag(Some(&serde_json::json!({ "instrumental": 0 })), "instrumental"));
    }
}
