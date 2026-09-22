//! kugou music (kugou.com).
//!
//! kugou's dedicated lyric search takes a track duration up front, which makes
//! it one of the most reliable providers for pinning the exact master cut.
//! lyrics ship in kugou's proprietary krc format, which carries real word-level
//! timings encrypted under a static repeating xor key.
//!
//! like netease, krc often bundles translations inside a base64 json header on
//! the body, giving us word-by-word karaoke and line translations in one fetch.

use std::io::Read;

use base64::Engine;
use flate2::read::ZlibDecoder;
use serde::Deserialize;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::parse::{scrub_spaces, strip_noise};
use crate::lyrics::types::{Candidate, LyricLine, LyricWord, TrackRef};

const SEARCH_URL: &str = "https://lyrics.kugou.com/search";
const DOWNLOAD_URL: &str = "https://lyrics.kugou.com/download";

/// repeating 16-byte xor key for krc payloads (post-header).
const KRC_KEY: [u8; 16] = [
    0x40, 0x47, 0x61, 0x77, 0x5e, 0x32, 0x74, 0x47,
    0x51, 0x36, 0x31, 0x2d, 0xce, 0xd2, 0x6e, 0x69,
];

#[derive(Deserialize, Default)]
struct SearchResp {
    #[serde(default)]
    candidates: Vec<SearchCandidate>,
}

#[derive(Deserialize, Default)]
struct SearchCandidate {
    id:        i64,
    #[serde(default)]
    accesskey: String,
    #[serde(default)]
    duration:  i64, // ms
    #[serde(default)]
    song:      String,
    #[serde(default)]
    singer:    String,
}

#[derive(Deserialize, Default)]
struct DownloadResp {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize, Default)]
struct LangHeader {
    #[serde(default)]
    content: Vec<LangContent>,
}

#[derive(Deserialize, Default)]
struct LangContent {
    #[serde(default, rename = "type")]
    content_type:  i32,
    #[serde(default, rename = "lyricContent")]
    lyric_content: Vec<Vec<String>>,
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    let client = crate::http::cookie_client();

    // 1. search. passing the expected duration directly filters out wrong masters
    //    at query time; clean title avoids clutter like "- Remastered"
    let clean = track.clean_name();
    let keyword = if track.artist.is_empty() {
        clean.clone()
    } else {
        format!("{} - {clean}", track.artist)
    };

    let mut url = url::Url::parse(SEARCH_URL).ok()?;
    url.query_pairs_mut()
        .append_pair("ver", "1")
        .append_pair("man", "yes")
        .append_pair("client", "pc")
        .append_pair("keyword", &keyword)
        .append_pair("duration", &track.duration_ms.to_string())
        .append_pair("hash", "");

    let raw = client
        .get(url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let search_resp: SearchResp = serde_json::from_str(&raw).ok()?;
    let candidate = best_hit(&search_resp.candidates, track, &clean)?;

    // 2. download encrypted krc
    let mut dl_url = url::Url::parse(DOWNLOAD_URL).ok()?;
    dl_url
        .query_pairs_mut()
        .append_pair("ver", "1")
        .append_pair("client", "pc")
        .append_pair("fmt", "krc")
        .append_pair("charset", "utf8")
        .append_pair("accesskey", &candidate.accesskey)
        .append_pair("id", &candidate.id.to_string());

    let dl_raw = client
        .get(dl_url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let dl_resp: DownloadResp = serde_json::from_str(&dl_raw).ok()?;
    let content_b64 = dl_resp.content?;

    // 3. decrypt: b64 decode -> drop "krc1" magic -> repeating xor -> zlib inflate
    let decrypted = decrypt_krc(&content_b64)?;

    // 4. parse word-level timestamps and any embedded translation
    let lines = strip_noise(parse_krc(&decrypted));
    if lines.is_empty() {
        return None;
    }

    Some(Candidate::new("kugou", lines))
}

/// pick the closest duration hit that agrees on title. artist mismatch is
/// tolerated when title matches, since romanized western vs cjk artist names
/// often disagree between spotify and chinese metadata
fn best_hit<'a>(
    candidates: &'a [SearchCandidate],
    track: &TrackRef,
    clean: &str,
) -> Option<&'a SearchCandidate> {
    let mut best: Option<((bool, i64), &'a SearchCandidate)> = None;

    for c in candidates {
        if !duration_ok(c.duration, track.duration_ms) {
            continue;
        }
        if !c.song.is_empty() && !text_matches(&c.song, clean) {
            continue;
        }

        let missed = !track.artist.is_empty() && !text_matches(&c.singer, &track.artist);
        let rank = (missed, (c.duration - track.duration_ms).abs());

        if best.as_ref().map(|(b, _)| &rank < b).unwrap_or(true) {
            best = Some((rank, c));
        }
    }

    best.map(|(_, c)| c)
}

/// decode base64 krc body, strip the 4-byte header, un-xor and inflate zlib
pub fn decrypt_krc(b64: &str) -> Option<String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64.trim().as_bytes())
        .ok()?;

    // magic must be "krc1" (4 bytes) followed by payload
    if raw.len() <= 4 || &raw[..4] != b"krc1" {
        return None;
    }

    let payload = &raw[4..];
    let unxored: Vec<u8> = payload
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ KRC_KEY[i % 16])
        .collect();

    let mut decoder = ZlibDecoder::new(&unxored[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    String::from_utf8(decompressed).ok()
}

/// extract line-by-line translations from the optional [language:<base64 json>] tag
fn extract_translations(raw: &str) -> Vec<String> {
    for line in raw.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("[language:") else {
            continue;
        };
        let Some(b64) = rest.strip_suffix(']') else {
            continue;
        };
        let Ok(json_bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) else {
            continue;
        };
        let Ok(header) = serde_json::from_slice::<LangHeader>(&json_bytes) else {
            continue;
        };

        for item in header.content {
            // type 1 indicates translation text
            if item.content_type == 1 {
                return item
                    .lyric_content
                    .into_iter()
                    .map(|row| row.join(" ").trim().to_string())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// parse krc text into word-timed lines with attached translations
pub fn parse_krc(raw: &str) -> Vec<LyricLine> {
    let translations = extract_translations(raw);
    let mut lines = Vec::new();
    let mut line_idx = 0;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }

        let Some(close_bracket) = line.find(']') else {
            continue;
        };
        let header = &line[1..close_bracket];
        let Some((start_s, dur_s)) = header.split_once(',') else {
            continue; // skips non-timed tags: [ti:..], [ar:..], [offset:..]
        };
        let Ok(line_start) = start_s.trim().parse::<i64>() else {
            continue;
        };
        let _line_dur = dur_s.trim().parse::<i64>().unwrap_or(0);

        let translation = translations.get(line_idx).filter(|t| !t.is_empty()).cloned();
        line_idx += 1;

        let rest = &line[close_bracket + 1..];
        let mut words = Vec::new();
        let mut cur = rest;

        // tokens are shaped as: <wOffsetMs,wDurMs,0>word<wOffsetMs,wDurMs,0>word...
        while let Some(open) = cur.find('<') {
            let Some(close) = cur[open..].find('>') else { break };
            let close = open + close;
            let meta = &cur[open + 1..close];
            let mut parts = meta.split(',');
            let offset = parts.next().and_then(|s| s.trim().parse::<i64>().ok());
            let dur = parts.next().and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0);

            let after = &cur[close + 1..];
            let next_tag = after.find('<').unwrap_or(after.len());
            let word_text = &after[..next_tag];

            if let Some(offset) = offset {
                let clean = scrub_spaces(word_text);
                if !clean.is_empty() {
                    // word offset is relative to line start
                    let start_ms = line_start + offset;
                    words.push(LyricWord {
                        time_ms: start_ms,
                        end_ms:  start_ms + dur.max(0),
                        text:    clean,
                    });
                }
            }
            cur = &after[next_tag..];
        }

        let text = if !words.is_empty() {
            words.iter().map(|w| w.text.as_str()).collect::<String>().trim().to_string()
        } else {
            scrub_spaces(rest).trim().to_string()
        };
        if text.is_empty() {
            continue;
        }

        let mut lyric_line = if !words.is_empty() {
            LyricLine::worded(line_start, Some(text), words)
        } else {
            LyricLine::line(line_start, text)
        };
        lyric_line.translation = translation;
        lines.push(lyric_line);
    }

    lines.sort_by_key(|l| l.time_ms);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_krc_words_with_relative_offsets() {
        let raw = "[1000,2000]<0,500,0>Hello <500,500,0>world";
        let lines = parse_krc(raw);
        assert_eq!(lines.len(), 1);
        let l = &lines[0];
        assert_eq!(l.time_ms, 1000);
        assert_eq!(l.text, "Hello world");
        assert_eq!(l.words.len(), 2);
        assert_eq!(l.words[0].text, "Hello ");
        assert_eq!(l.words[0].time_ms, 1000);
        assert_eq!(l.words[0].end_ms, 1500);
        assert_eq!(l.words[1].text, "world");
        assert_eq!(l.words[1].time_ms, 1500);
        assert_eq!(l.words[1].end_ms, 2000);
    }

    #[test]
    fn parses_and_attaches_translations() {
        // base64 json for: {"content":[{"type":1,"lyricContent":[["你好 世界"]]}]}
        let trans_json = r#"{"content":[{"type":1,"lyricContent":[["你好 世界"]]}]}"#;
        let b64 = base64::engine::general_purpose::STANDARD.encode(trans_json.as_bytes());
        let raw = format!("[language:{b64}]\n[1000,2000]<0,500,0>Hello <500,500,0>world");

        let lines = parse_krc(&raw);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].translation.as_deref(), Some("你好 世界"));
    }

    #[test]
    fn skips_metadata_headers() {
        let raw = "[ti:Test Title]\n[ar:Test Artist]\n[offset:0]\n[500,1000]<0,500,0>Line <500,500,0>one";
        let lines = parse_krc(raw);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].time_ms, 500);
        assert_eq!(lines[0].text, "Line one");
    }

    #[test]
    fn decrypts_valid_krc_payload() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let plain = "[100,500]<0,500,0>Test";
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(plain.as_bytes()).unwrap();
        let compressed = enc.finish().unwrap();

        let mut xored = Vec::with_capacity(4 + compressed.len());
        xored.extend_from_slice(b"krc1");
        for (i, &b) in compressed.iter().enumerate() {
            xored.push(b ^ KRC_KEY[i % 16]);
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&xored);

        let decrypted = decrypt_krc(&b64).expect("decrypt should succeed");
        assert_eq!(decrypted, plain);
    }
}
