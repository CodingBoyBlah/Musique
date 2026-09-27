//! spotify's own color-lyrics endpoint - the sync reference.
//!
//! this one is special: it is keyed to the EXACT track id we are playing, so
//! there is zero version ambiguity (no remaster/live/edit mixup possible) and
//! the timings are the ones the official client renders. every other provider
//! gets aligned against this when it's present.
//!
//! it goes over the live librespot session rather than the web api - the
//! endpoint is spclient-only and needs the first-party client identity that the
//! session already carries. no session (playback not initialised yet) means no
//! answer, never a panic.

use librespot_core::{error::ErrorKind, Session, SpotifyId};
use serde::Deserialize;
use tauri::Manager;

use crate::lyrics::{
    parse::scrub_spaces,
    types::{Candidate, LyricLine, LyricWord, TrackRef},
};

/// the interlude markers spotify emits for instrumental stretches. we keep the
/// lines (the ui needs the gaps) but recognise them for the instrumental check
const INTERLUDE: [&str; 3] = ["\u{266a}", "\u{266b}", "\u{266c}"]; // ♪ ♫ ♬

// wire types

#[derive(Deserialize)]
struct Root {
    #[serde(default)]
    lyrics: Option<Body>,
}

#[derive(Deserialize)]
struct Body {
    #[serde(rename = "syncType", default)]
    sync_type: String,
    #[serde(default)]
    lines: Vec<Line>,
}

#[derive(Deserialize)]
struct Line {
    // every timestamp arrives as a *string* of millis, so these stay Value and
    // go through `ms()` which also survives the numeric shape
    #[serde(rename = "startTimeMs", default)]
    start_time_ms: serde_json::Value,
    #[serde(rename = "endTimeMs", default)]
    end_time_ms: serde_json::Value,
    #[serde(default)]
    words: String,
    /// usually empty; when spotify does fill it we get word-level for free
    #[serde(default)]
    syllables: Vec<Syllable>,
}

#[derive(Deserialize)]
struct Syllable {
    #[serde(rename = "startTimeMs", default)]
    start_time_ms: serde_json::Value,
    #[serde(rename = "endTimeMs", default)]
    end_time_ms: serde_json::Value,
    /// how many chars of the parent line's `words` this syllable covers
    #[serde(rename = "numChars", default)]
    num_chars: usize,
}

// entry point

pub async fn fetch(app: &tauri::AppHandle, track: &TrackRef) -> Option<Candidate> {
    let session = live_session(app).await?;
    let Ok(id) = SpotifyId::from_base62(&track.id) else {
        return None;
    };

    let body = match session.spclient().get_lyrics(&id).await {
        Ok(b) => b,
        Err(e) => {
            // 404 -> spotify simply has no lyrics for this track, which is the
            // common case. this runs on every track change, so stay quiet about
            // it and only shout when something actually went wrong
            if e.kind != ErrorKind::NotFound {
                eprintln!("[lyrics] spotify color-lyrics failed for {}: {e}", track.id);
            }
            return None;
        }
    };

    let root: Root = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[lyrics] spotify color-lyrics bad json for {}: {e}", track.id);
            return None;
        }
    };

    parse_body(root.lyrics?)
}

/// clone the session out of app state, checking it is actually usable first.
/// the guard is released before we return so no network i/o ever happens while
/// the playback mutex is held - that would stall every transport control
async fn live_session(app: &tauri::AppHandle) -> Option<Session> {
    // try_state, not state: state() panics when nothing is managed, and lyrics
    // must never take the app down
    let playback = app.try_state::<crate::state::AppState>()?.playback.clone();

    let guard = playback.lock().await;
    let inner = guard.as_ref()?;
    // a dropped/expired session eats spclient calls; no lyrics is better than a
    // hang waiting on a dead connection
    if inner.session_invalid() {
        return None;
    }
    Some(inner.session())
}

// parsing

fn parse_body(body: Body) -> Option<Candidate> {
    if body.lines.is_empty() {
        return None;
    }

    // LINE_SYNCED is what we get in practice; SYLLABLE_SYNCED shows up on the
    // handful of word-timed tracks. anything else (UNSYNCED) has no usable
    // timestamps at all and can only ever be plain text
    let synced = matches!(body.sync_type.as_str(), "LINE_SYNCED" | "SYLLABLE_SYNCED");
    let instrumental = body.lines.iter().all(|l| is_interlude(&l.words));

    if !synced {
        let plain = body
            .lines
            .iter()
            .map(|l| scrub_spaces(l.words.trim()))
            .collect::<Vec<_>>()
            .join("\n");
        let plain = plain.trim().to_string();
        if plain.is_empty() && !instrumental {
            return None;
        }

        let mut cand = Candidate::new("spotify", Vec::new());
        cand.exact = true; // fetched BY track id - there is no other recording it could be
        cand.plain = (!plain.is_empty()).then_some(plain);
        cand.instrumental = instrumental;
        return Some(cand);
    }

    let mut lines: Vec<LyricLine> = Vec::new();
    for l in &body.lines {
        let Some(start) = ms(&l.start_time_ms) else {
            continue;
        };
        let text = scrub_spaces(l.words.trim());
        let end = ms(&l.end_time_ms).filter(|e| *e > start);

        match syllable_words(l, start, end) {
            Some(words) => lines.push(LyricLine::worded(start, Some(text), words)),
            None => lines.push(LyricLine::line(start, text)),
        }
    }

    if lines.is_empty() {
        return None;
    }
    lines.sort_by_key(|l| l.time_ms);

    let mut cand = Candidate::new("spotify", lines);
    cand.exact = true; // fetched BY track id - there is no other recording it could be
    cand.instrumental = instrumental;
    Some(cand)
}

/// turn a line's `syllables` into real word timings. `numChars` walks the
/// line's `words` string, so the only way this is trustworthy is if the counts
/// actually fit the text - otherwise we'd slice gibberish, and a line-level
/// entry is strictly better than wrong word timings
fn syllable_words(line: &Line, line_start: i64, line_end: Option<i64>) -> Option<Vec<LyricWord>> {
    if line.syllables.is_empty() {
        return None;
    }

    let chars: Vec<char> = line.words.chars().collect();
    let total: usize = line.syllables.iter().map(|s| s.num_chars).sum();
    if total == 0 || total > chars.len() {
        return None;
    }

    let starts: Vec<i64> = line
        .syllables
        .iter()
        .map(|s| ms(&s.start_time_ms))
        .collect::<Option<Vec<_>>>()?;

    // some payloads time syllables from the start of the line instead of the
    // start of the track; if every one of them lands before the line does,
    // they're relative and need rebasing
    let rebase = starts.iter().all(|s| *s < line_start);

    let mut words: Vec<LyricWord> = Vec::new();
    let mut cursor = 0usize;
    for (i, syl) in line.syllables.iter().enumerate() {
        let text: String = chars[cursor..cursor + syl.num_chars].iter().collect();
        cursor += syl.num_chars;

        let start = if rebase { line_start + starts[i] } else { starts[i] };
        // endTimeMs is frequently "0"; fall back to the next syllable, then to
        // the line's own end, then to a zero-length word
        let end = ms(&syl.end_time_ms)
            .map(|e| if rebase { line_start + e } else { e })
            .filter(|e| *e > start)
            .or_else(|| {
                starts
                    .get(i + 1)
                    .map(|n| if rebase { line_start + *n } else { *n })
                    .filter(|n| *n > start)
            })
            .or(line_end)
            .unwrap_or(start);

        let text = scrub_spaces(&text);
        if text.trim().is_empty() {
            continue; // pure whitespace slice carries no timing worth keeping
        }
        words.push(LyricWord { time_ms: start, end_ms: end.max(start), text });
    }

    // any chars the counts didn't cover belong to the last word, so the joined
    // word text still reads as the full line
    if cursor < chars.len() {
        let tail = scrub_spaces(&chars[cursor..].iter().collect::<String>());
        if !tail.trim().is_empty() {
            match words.last_mut() {
                Some(w) => w.text.push_str(&tail),
                None => return None,
            }
        }
    }

    (!words.is_empty()).then_some(words)
}

/// millis out of the wire value. spotify sends strings, but accept the numeric
/// shape too rather than dropping a whole line over it
fn ms(v: &serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::String(s) => s.trim().parse().ok(),
        serde_json::Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

/// a blank or ♪ line marks an instrumental stretch rather than sung text
fn is_interlude(text: &str) -> bool {
    let t = text.trim();
    t.is_empty() || INTERLUDE.contains(&t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(raw: &str) -> Body {
        serde_json::from_str::<Root>(raw).unwrap().lyrics.unwrap()
    }

    #[test]
    fn parses_line_synced() {
        let cand = parse_body(body(
            r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[
                {"startTimeMs":"12340","words":"hello there","syllables":[],"endTimeMs":"0"},
                {"startTimeMs":"15000","words":"♪","syllables":[],"endTimeMs":"0"}]}}"#,
        ))
        .unwrap();
        assert_eq!(cand.source, "spotify");
        assert_eq!(cand.lines.len(), 2);
        assert_eq!(cand.lines[0].time_ms, 12_340);
        assert_eq!(cand.lines[0].text, "hello there");
        assert_eq!(cand.lines[1].text, "\u{266a}"); // interlude markers survive
        assert!(!cand.instrumental);
        assert!(!cand.word_level());
    }

    #[test]
    fn all_markers_is_instrumental() {
        let cand = parse_body(body(
            r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[
                {"startTimeMs":"0","words":"♪","syllables":[],"endTimeMs":"0"},
                {"startTimeMs":"9000","words":"","syllables":[],"endTimeMs":"0"}]}}"#,
        ))
        .unwrap();
        assert!(cand.instrumental);
    }

    #[test]
    fn unsynced_is_plain_only() {
        let cand = parse_body(body(
            r#"{"lyrics":{"syncType":"UNSYNCED","lines":[
                {"startTimeMs":"0","words":"first","syllables":[],"endTimeMs":"0"},
                {"startTimeMs":"0","words":"second","syllables":[],"endTimeMs":"0"}]}}"#,
        ))
        .unwrap();
        assert!(cand.lines.is_empty());
        assert_eq!(cand.plain.as_deref(), Some("first\nsecond"));
    }

    #[test]
    fn syllables_become_words() {
        let cand = parse_body(body(
            r#"{"lyrics":{"syncType":"SYLLABLE_SYNCED","lines":[
                {"startTimeMs":"1000","words":"hey you","endTimeMs":"2000","syllables":[
                    {"startTimeMs":"1000","endTimeMs":"1400","numChars":4},
                    {"startTimeMs":"1400","endTimeMs":"2000","numChars":3}]}]}}"#,
        ))
        .unwrap();
        let l = &cand.lines[0];
        assert_eq!(l.text, "hey you");
        assert_eq!(l.words.len(), 2);
        assert_eq!(l.words[0].text, "hey ");
        assert_eq!(l.words[0].end_ms, 1400);
        assert_eq!(l.words[1].text, "you");
        assert_eq!(l.words[1].end_ms, 2000);
    }

    #[test]
    fn nonsense_syllable_counts_fall_back_to_line_level() {
        let cand = parse_body(body(
            r#"{"lyrics":{"syncType":"LINE_SYNCED","lines":[
                {"startTimeMs":"1000","words":"hi","endTimeMs":"0","syllables":[
                    {"startTimeMs":"1000","endTimeMs":"1400","numChars":40}]}]}}"#,
        ))
        .unwrap();
        assert!(!cand.word_level());
        assert_eq!(cand.lines[0].text, "hi");
    }
}
