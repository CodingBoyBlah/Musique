//! netease cloud music (music.163.com).
//!
//! the only free source that hands back *word-level* timings (`yrc`) together
//! with a translation and a romanisation for the same track, in one response.
//! that last part matters beyond karaoke: a provider-supplied pinyin / romaji
//! line means the frontend never has to run pinyin-pro or wanakana at render
//! time, which is the difference between an instant paint and a stutter.
//!
//! two fixes over the old `fetch_netease_yrc` in `lyrics/mod.rs`:
//!   - it accepted any hit inside a 6s duration window, which routinely landed
//!     on a different master (remix, live cut, extended edit). here the match
//!     has to pass `duration_ok` (1.5s) *and* a title check, so a duration
//!     coincidence inside a 10-hit search page can't pick another song.
//!   - it read only `yrc` and threw the rest of the same response away. a track
//!     with line-level lyrics plus a translation looked like "nothing found"
//!     even though the bytes were already in hand.

use serde::Deserialize;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::parse::{merge_parallel, parse_lrc, parse_yrc, strip_noise};
use crate::lyrics::types::{Candidate, LyricLine, TrackRef};

const SEARCH_URL: &str = "https://music.163.com/api/search/get";
/// interface3 is the host that reliably serves the word-level `yrc` field;
/// the plain music.163.com lyric endpoint silently omits it
const LYRIC_URL: &str = "https://interface3.music.163.com/api/song/lyric/v1";
const REFERER: &str = "https://music.163.com";
/// pretending to be the pc client is what unlocks the full lyric set
const NE_COOKIE: &str = "os=pc; appver=8.9.70";

#[derive(Deserialize)]
struct SearchResp {
    result: Option<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    #[serde(default)]
    songs: Vec<Song>,
}

#[derive(Deserialize)]
struct Song {
    id:       i64,
    #[serde(default)]
    duration: i64, // ms
    #[serde(default)]
    name:     String,
    #[serde(default)]
    artists:  Vec<Artist>,
}

#[derive(Deserialize)]
struct Artist {
    #[serde(default)]
    name: String,
}

/// every lyric flavour the v1 endpoint can return. all optional - most tracks
/// only carry `lrc`, and the `y*` fields only exist where a yrc body does
#[derive(Deserialize, Default)]
struct LyricResp {
    #[serde(default)]
    yrc:      Option<Body>, // word level, the good one
    #[serde(default)]
    klyric:   Option<Body>, // older karaoke field, word-ish on some entries
    #[serde(default)]
    lrc:      Option<Body>, // line level
    #[serde(default)]
    tlyric:   Option<Body>, // translation, lrc-shaped
    #[serde(default)]
    romalrc:  Option<Body>, // romanisation, lrc-shaped
    #[serde(default)]
    ytlrc:    Option<Body>, // translation carrying the yrc onsets
    #[serde(default)]
    yromalrc: Option<Body>, // romanisation carrying the yrc onsets
}

#[derive(Deserialize)]
struct Body {
    #[serde(default)]
    lyric: Option<String>,
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    let client = crate::http::cookie_client();

    // 1. search. the bare title searches far better than the spotify one here
    //    ("- 2011 Remaster" finds nothing); the match is verified below anyway
    let clean = track.clean_name();
    let mut url = url::Url::parse(SEARCH_URL).ok()?;
    url.query_pairs_mut()
        .append_pair("s", &format!("{clean} {}", track.artist))
        .append_pair("type", "1")
        .append_pair("limit", "10");

    let body = get(client, url).send().await.ok()?.text().await.ok()?;
    let songs = serde_json::from_str::<SearchResp>(&body).ok()?.result?.songs;

    let id = best_hit(&songs, track, &clean)?;

    // 2. one request for every flavour at once. `rv=1` is the romanisation
    //    track - the old code omitted it, so every pinyin / romaji line netease
    //    already had was dropped for free inside the same round trip
    let mut url = url::Url::parse(LYRIC_URL).ok()?;
    url.query_pairs_mut()
        .append_pair("id", &id.to_string())
        .append_pair("cp", "false")
        .append_pair("lv", "1")
        .append_pair("kv", "1")
        .append_pair("tv", "1")
        .append_pair("yv", "1")
        .append_pair("ytv", "1")
        .append_pair("yrc", "1")
        .append_pair("rv", "1");

    let raw = get(client, url).send().await.ok()?.text().await.ok()?;
    let resp: LyricResp = serde_json::from_str(&raw).ok()?;

    let lines = assemble(&resp);
    if lines.is_empty() {
        return None;
    }
    Some(Candidate::new("netease", lines))
}

/// shared request shape. netease 403s a default agent and hides the word-level
/// fields without the pc-client cookie
fn get(client: &reqwest::Client, url: url::Url) -> reqwest::RequestBuilder {
    client
        .get(url)
        .header("User-Agent", UA)
        .header("Referer", REFERER)
        .header("Cookie", NE_COOKIE)
}

/// closest-duration hit that also agrees on the title. both gates, not just
/// duration: near-ties are common inside one search page (a remix is often cut
/// to the same length as the original), and serving another song's lyrics is
/// worse than serving none
fn best_hit(songs: &[Song], track: &TrackRef, clean: &str) -> Option<i64> {
    let mut best: Option<((bool, i64), i64)> = None; // ((artist mismatch, gap), id)

    for s in songs {
        if !duration_ok(s.duration, track.duration_ms) {
            continue;
        }
        if !text_matches(&s.name, clean) {
            continue;
        }

        // an artist agreement is a tiebreak, never a veto: netease lists cjk
        // names the spotify catalogue romanises, so requiring it would reject
        // most of the tracks this provider exists for. false sorts first
        let missed = !s.artists.iter().any(|a| text_matches(&a.name, &track.artist));
        let rank = (missed, (s.duration - track.duration_ms).abs());

        if best.map(|(b, _)| rank < b).unwrap_or(true) {
            best = Some((rank, s.id));
        }
    }

    best.map(|(_, id)| id)
}

/// turn one lyric response into final lines: the best available timing source,
/// plus whatever translation and romanisation shipped alongside it. pure, so
/// the preference order is testable without the network
fn assemble(resp: &LyricResp) -> Vec<LyricLine> {
    // word level first - yrc is the real sung timing
    let mut lines = parse_yrc(text(&resp.yrc));
    let mut on_yrc = !lines.is_empty();

    // klyric is the older karaoke field. it's yrc-shaped on some entries and
    // plain lrc on others, so try both before giving up on word timings
    if lines.is_empty() {
        let k = text(&resp.klyric);
        if !k.trim().is_empty() {
            lines = parse_yrc(k);
            on_yrc = !lines.is_empty();
            if lines.is_empty() {
                lines = parse_lrc(k);
            }
        }
    }

    if lines.is_empty() {
        lines = parse_lrc(text(&resp.lrc));
    }
    if lines.is_empty() {
        return Vec::new();
    }

    // the y-variants carry the yrc onsets, which drift from the lrc ones by
    // more than merge_parallel's 350ms window on some tracks - so take the one
    // written against the body we actually chose
    let tl = pick(&resp.ytlrc, &resp.tlyric, on_yrc);
    let rl = pick(&resp.yromalrc, &resp.romalrc, on_yrc);
    if !tl.trim().is_empty() {
        merge_parallel(&mut lines, tl, false);
    }
    if !rl.trim().is_empty() {
        merge_parallel(&mut lines, rl, true);
    }

    // netease bodies almost always open with timestamped credit lines
    // (composer / lyricist / arranger), which would otherwise render as lyrics
    strip_noise(lines)
}

fn text(b: &Option<Body>) -> &str {
    b.as_ref().and_then(|b| b.lyric.as_deref()).unwrap_or("")
}

/// the yrc-aligned body when we're on yrc and it exists, else the plain one
fn pick<'a>(y: &'a Option<Body>, plain: &'a Option<Body>, on_yrc: bool) -> &'a str {
    if on_yrc {
        let t = text(y);
        if !t.trim().is_empty() {
            return t;
        }
    }
    text(plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(s: &str) -> Option<Body> {
        Some(Body { lyric: Some(s.to_string()) })
    }

    fn song(id: i64, duration: i64, name: &str) -> Song {
        Song { id, duration, name: name.to_string(), artists: Vec::new() }
    }

    #[test]
    fn prefers_word_level_and_merges_both_parallels() {
        let resp = LyricResp {
            yrc:     body("[0,500](0,250,0)hello (250,250,0)world"),
            lrc:     body("[00:00.00]hello world"),
            tlyric:  body("[00:00.00]\u{4f60}\u{597d}"),
            romalrc: body("[00:00.00]ni hao"),
            ..Default::default()
        };
        let lines = assemble(&resp);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].words.len(), 2); // came from yrc, not from lrc
        assert_eq!(lines[0].translation.as_deref(), Some("\u{4f60}\u{597d}"));
        assert_eq!(lines[0].roman.as_deref(), Some("ni hao"));
    }

    #[test]
    fn falls_back_to_lrc_and_still_takes_the_translation() {
        let resp = LyricResp {
            lrc:    body("[00:10.00]hello"),
            tlyric: body("[00:10.00]bonjour"),
            ..Default::default()
        };
        let lines = assemble(&resp);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].words.is_empty());
        assert_eq!(lines[0].translation.as_deref(), Some("bonjour"));
    }

    #[test]
    fn klyric_covers_a_missing_yrc() {
        let resp = LyricResp {
            klyric: body("[0,400](0,200,0)la (200,200,0)la"),
            ..Default::default()
        };
        assert_eq!(assemble(&resp)[0].words.len(), 2);
    }

    #[test]
    fn drops_the_credit_header() {
        let resp = LyricResp {
            lrc: body("[00:00.00]\u{4f5c}\u{8bcd} : someone\n[00:05.00]real line"),
            ..Default::default()
        };
        let lines = assemble(&resp);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "real line");
    }

    #[test]
    fn duration_and_title_both_gate_the_hit() {
        let track = TrackRef {
            name:        "Shape of You".into(),
            duration_ms: 233_712,
            ..Default::default()
        };
        let songs = vec![
            song(1, 233_712, "Perfect"),      // right length, wrong song
            song(2, 240_000, "Shape of You"), // right song, wrong master
            song(3, 233_000, "Shape of You"), // the one
        ];
        assert_eq!(best_hit(&songs, &track, "Shape of You"), Some(3));
    }

    #[test]
    fn no_hit_when_nothing_lines_up() {
        let track = TrackRef { name: "Perfect".into(), duration_ms: 263_000, ..Default::default() };
        assert_eq!(best_hit(&[song(1, 233_712, "Shape of You")], &track, "Perfect"), None);
    }
}
