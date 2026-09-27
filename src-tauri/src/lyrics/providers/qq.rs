//! qq music (y.qq.com).
//!
//! qq music serves word-level timed lyrics in its proprietary encrypted qrc format.
//! the payload is triple-des encrypted in ecb mode using well-known static qq keys,
//! then zlib compressed. decrypting it gives xml containing real character/word timings.
//!
//! all requests strictly require `Referer: https://y.qq.com/` or qq's gateway
//! rejects them with 403 / 401.

use std::io::Read;

use base64::Engine;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use des::Des;
use flate2::read::ZlibDecoder;
use serde::Deserialize;

use super::{duration_ok, text_matches, UA};
use crate::lyrics::parse::{scrub_spaces, strip_noise};
use crate::lyrics::types::{Candidate, LyricLine, LyricWord, TrackRef};

const SEARCH_URL: &str = "https://c.y.qq.com/soso/fcgi-bin/client_search_cp";
const LYRIC_URL: &str = "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_yqq.fcg";
const MUSICU_URL: &str = "https://u.y.qq.com/cgi-bin/musicu.fcg";
const REFERER: &str = "https://y.qq.com/";

/// static 3des keys for qrc decryption (in ede order: d(k1) -> e(k2) -> d(k3)).
const KEY1: &[u8; 8] = b"!@#)(NHL";
const KEY2: &[u8; 8] = b"123ZXC!@";
const KEY3: &[u8; 8] = b"!@#)(*$%";

#[derive(Deserialize, Default)]
struct SearchResp {
    #[serde(default)]
    data: Option<SearchData>,
}

#[derive(Deserialize, Default)]
struct SearchData {
    #[serde(default)]
    song: Option<SongData>,
}

#[derive(Deserialize, Default)]
struct SongData {
    #[serde(default)]
    list: Vec<SongItem>,
}

#[derive(Deserialize, Default)]
struct SongItem {
    #[serde(default)]
    songmid:  String,
    #[serde(default)]
    songname: String,
    #[serde(default)]
    interval: i64, // seconds, not ms
    #[serde(default)]
    singer:   Vec<SingerItem>,
}

#[derive(Deserialize, Default)]
struct SingerItem {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Default)]
struct FcgLyricResp {
    #[serde(default)]
    qrc:   Option<String>,
    #[serde(default)]
    lyric: Option<String>,
}

#[derive(Deserialize, Default)]
struct MusicuResp {
    #[serde(default)]
    lyric: Option<MusicuLyricModule>,
}

#[derive(Deserialize, Default)]
struct MusicuLyricModule {
    #[serde(default)]
    data: Option<MusicuLyricData>,
}

#[derive(Deserialize, Default)]
struct MusicuLyricData {
    #[serde(default)]
    qrc:   Option<String>,
    #[serde(default)]
    lyric: Option<String>,
}

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    let client = crate::http::cookie_client();

    // 1. search for song mid. clean name avoids noise like "- Remastered"
    let clean = track.clean_name();
    let query = if track.artist.is_empty() {
        clean.clone()
    } else {
        format!("{clean} {}", track.artist)
    };

    let mut url = url::Url::parse(SEARCH_URL).ok()?;
    url.query_pairs_mut()
        .append_pair("w", &query)
        .append_pair("format", "json")
        .append_pair("p", "1")
        .append_pair("n", "10");

    let raw = client
        .get(url)
        .header("User-Agent", UA)
        .header("Referer", REFERER)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let search_resp: SearchResp = serde_json::from_str(&raw).ok()?;
    let songs = search_resp.data?.song?.list;
    let song = best_hit(&songs, track, &clean)?;

    // 2. fetch encrypted qrc. try standard lyric endpoint first, then musicu gateway
    let raw_payload = fetch_lyric_payload(client, &song.songmid).await?;

    // 3. decrypt: hex / base64 decode -> 3des ede -> zlib inflate
    let decrypted = decrypt_qrc(&raw_payload)?;

    // 4. parse word-level timestamps from qrc xml
    let lines = strip_noise(parse_qrc(&decrypted));
    if lines.is_empty() {
        return None;
    }

    Some(Candidate::new("qq", lines))
}

/// fetch encrypted lyric string using fcg endpoint, falling back to musicu
async fn fetch_lyric_payload(client: &reqwest::Client, songmid: &str) -> Option<String> {
    // try fcg_query_lyric_yqq first
    if let Ok(mut url) = url::Url::parse(LYRIC_URL) {
        url.query_pairs_mut()
            .append_pair("songmid", songmid)
            .append_pair("format", "json")
            .append_pair("nobase64", "0")
            .append_pair("g_tk", "5381");

        if let Ok(resp) = client
            .get(url)
            .header("User-Agent", UA)
            .header("Referer", REFERER)
            .send()
            .await
        {
            if let Ok(text) = resp.text().await {
                if let Ok(fcg) = serde_json::from_str::<FcgLyricResp>(&text) {
                    if let Some(qrc) = fcg.qrc.filter(|s| !s.trim().is_empty()) {
                        return Some(qrc);
                    }
                    if let Some(lyric) = fcg.lyric.filter(|s| !s.trim().is_empty()) {
                        return Some(lyric);
                    }
                }
            }
        }
    }

    // fallback to musicu GetPlayLyricInfo
    let post_body = serde_json::json!({
        "comm": { "ct": 24, "cv": 0 },
        "lyric": {
            "module": "music.musichallSong.PlayLyricInfo",
            "method": "GetPlayLyricInfo",
            "param": {
                "songMID": songmid,
                "qrc": 1,
                "roma": 1,
                "trans": 1
            }
        }
    });

    let resp = client
        .post(MUSICU_URL)
        .header("User-Agent", UA)
        .header("Referer", REFERER)
        .json(&post_body)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let musicu: MusicuResp = serde_json::from_str(&resp).ok()?;
    let data = musicu.lyric?.data?;
    data.qrc.filter(|s| !s.trim().is_empty()).or(data.lyric)
}

/// pick the closest duration hit that agrees on title. interval is reported in seconds
fn best_hit<'a>(songs: &'a [SongItem], track: &TrackRef, clean: &str) -> Option<&'a SongItem> {
    let mut best: Option<((bool, i64), &'a SongItem)> = None;

    for s in songs {
        let duration_ms = s.interval * 1000;
        if !duration_ok(duration_ms, track.duration_ms) {
            continue;
        }
        if !text_matches(&s.songname, clean) {
            continue;
        }

        let missed = !track.artist.is_empty()
            && !s.singer.iter().any(|singer| text_matches(&singer.name, &track.artist));
        let rank = (missed, (duration_ms - track.duration_ms).abs());

        if best.as_ref().map(|(b, _)| &rank < b).unwrap_or(true) {
            best = Some((rank, s));
        }
    }

    best.map(|(_, s)| s)
}

/// decode payload from hex (standard qq format) or base64 (alternate format)
pub fn decode_raw_payload(raw: &str) -> Option<Vec<u8>> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }

    // hex is the primary format served by qq
    if s.len() % 2 == 0 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        let mut bytes = Vec::with_capacity(s.len() / 2);
        for i in (0..s.len()).step_by(2) {
            let b = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
            bytes.push(b);
        }
        return Some(bytes);
    }

    // fallback to base64
    base64::engine::general_purpose::STANDARD.decode(s.as_bytes()).ok()
}

/// decrypt qrc ciphertext using 3des ede mode and zlib inflate.
/// tries qq's custom s-box 3des first, then standard des crate 3des as fallback
pub fn decrypt_qrc(raw: &str) -> Option<String> {
    let ciphertext = decode_raw_payload(raw)?;
    if ciphertext.len() < 8 || ciphertext.len() % 8 != 0 {
        return None;
    }

    // 1. try qq modified 3des: d(k1) -> e(k2) -> d(k3)
    if let Some(text) = try_qq_3des(&ciphertext, KEY1, KEY2, KEY3) {
        return Some(text);
    }
    if let Some(text) = try_qq_3des(&ciphertext, KEY3, KEY2, KEY1) {
        return Some(text);
    }

    // 2. try standard 3des from `des` crate: d(k1) -> e(k2) -> d(k3)
    if let Some(text) = try_std_3des(&ciphertext, KEY1, KEY2, KEY3) {
        return Some(text);
    }
    if let Some(text) = try_std_3des(&ciphertext, KEY3, KEY2, KEY1) {
        return Some(text);
    }

    None
}

fn try_qq_3des(ciphertext: &[u8], k1: &[u8; 8], k2: &[u8; 8], k3: &[u8; 8]) -> Option<String> {
    let mut buf = ciphertext.to_vec();
    let des1 = qq_des::QqDes::new(k1, false);
    let des2 = qq_des::QqDes::new(k2, true);
    let des3 = qq_des::QqDes::new(k3, false);

    des1.transform_bytes(&mut buf);
    des2.transform_bytes(&mut buf);
    des3.transform_bytes(&mut buf);

    inflate(&buf)
}

fn try_std_3des(ciphertext: &[u8], k1: &[u8; 8], k2: &[u8; 8], k3: &[u8; 8]) -> Option<String> {
    let mut buf = ciphertext.to_vec();
    let des1 = Des::new_from_slice(k1).ok()?;
    let des2 = Des::new_from_slice(k2).ok()?;
    let des3 = Des::new_from_slice(k3).ok()?;

    for chunk in buf.chunks_exact_mut(8) {
        let block = GenericArray::from_mut_slice(chunk);
        des1.decrypt_block(block);
        des2.encrypt_block(block);
        des3.decrypt_block(block);
    }

    inflate(&buf)
}

fn inflate(bytes: &[u8]) -> Option<String> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;
    String::from_utf8(decompressed).ok()
}

/// qq-specific des implementation with modified non-standard s-boxes
mod qq_des {
    const KEY_RND_SHIFTS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];
    const LARGE_STATE_SHIFTS: [u8; 8] = [26, 20, 14, 8, 58, 52, 46, 40];

    const SBOXES: [[u8; 64]; 8] = [
        [
            14, 0, 4, 15, 13, 7, 1, 4, 2, 14, 15, 2, 11, 13, 8, 1, 3, 10, 10, 6, 6, 12, 12, 11, 5, 9,
            9, 5, 0, 3, 7, 8, 4, 15, 1, 12, 14, 8, 8, 2, 13, 4, 6, 9, 2, 1, 11, 7, 15, 5, 12, 11, 9, 3,
            7, 14, 3, 10, 10, 0, 5, 6, 0, 13,
        ],
        [
            15, 3, 1, 13, 8, 4, 14, 7, 6, 15, 11, 2, 3, 8, 4, 15, 9, 12, 7, 0, 2, 1, 13, 10, 12, 6, 0,
            9, 5, 11, 10, 5, 0, 13, 14, 8, 7, 10, 11, 1, 10, 3, 4, 15, 13, 4, 1, 2, 5, 11, 8, 6, 12, 7,
            6, 12, 9, 0, 3, 5, 2, 14, 15, 9,
        ],
        [
            10, 13, 0, 7, 9, 0, 14, 9, 6, 3, 3, 4, 15, 6, 5, 10, 1, 2, 13, 8, 12, 5, 7, 14, 11, 12, 4,
            11, 2, 15, 8, 1, 13, 1, 6, 10, 4, 13, 9, 0, 8, 6, 15, 9, 3, 8, 0, 7, 11, 4, 1, 15, 2, 14,
            12, 3, 5, 11, 10, 5, 14, 2, 7, 12,
        ],
        [
            7, 13, 13, 8, 14, 11, 3, 5, 0, 6, 6, 15, 9, 0, 10, 3, 1, 4, 2, 7, 8, 2, 5, 12, 11, 1, 12,
            10, 4, 14, 15, 9, 10, 3, 6, 15, 9, 0, 0, 6, 12, 10, 11, 10, 7, 13, 13, 8, 15, 9, 1, 4, 3,
            5, 14, 11, 5, 12, 2, 7, 8, 2, 4, 14,
        ],
        [
            2, 14, 12, 11, 4, 2, 1, 12, 7, 4, 10, 7, 11, 13, 6, 1, 8, 5, 5, 0, 3, 15, 15, 10, 13, 3, 0,
            9, 14, 8, 9, 6, 4, 11, 2, 8, 1, 12, 11, 7, 10, 1, 13, 14, 7, 2, 8, 13, 15, 6, 9, 15, 12, 0,
            5, 9, 6, 10, 3, 4, 0, 5, 14, 3,
        ],
        [
            12, 10, 1, 15, 10, 4, 15, 2, 9, 7, 2, 12, 6, 9, 8, 5, 0, 6, 13, 1, 3, 13, 4, 14, 14, 0, 7,
            11, 5, 3, 11, 8, 9, 4, 14, 3, 15, 2, 5, 12, 2, 9, 8, 5, 12, 15, 3, 10, 7, 11, 0, 14, 4, 1,
            10, 7, 1, 6, 13, 0, 11, 8, 6, 13,
        ],
        [
            4, 13, 11, 0, 2, 11, 14, 7, 15, 4, 0, 9, 8, 1, 13, 10, 3, 14, 12, 3, 9, 5, 7, 12, 5, 2, 10,
            15, 6, 8, 1, 6, 1, 6, 4, 11, 11, 13, 13, 8, 12, 1, 3, 4, 7, 10, 14, 7, 10, 9, 15, 5, 6, 0,
            8, 15, 0, 14, 5, 2, 9, 3, 2, 12,
        ],
        [
            13, 1, 2, 15, 8, 13, 4, 8, 6, 10, 15, 3, 11, 7, 1, 4, 10, 12, 9, 5, 3, 6, 14, 11, 5, 0, 0,
            14, 12, 9, 7, 2, 7, 2, 11, 1, 4, 14, 1, 7, 9, 4, 12, 10, 14, 8, 2, 13, 0, 15, 6, 12, 10, 9,
            13, 0, 15, 3, 3, 5, 5, 6, 8, 11,
        ],
    ];

    const P_BOX: [u8; 32] = [
        15, 6, 19, 20, 28, 11, 27, 16, 0, 14, 22, 25, 4, 17, 30, 9, 1, 7, 23, 13, 31, 26, 2, 8, 18,
        12, 29, 5, 21, 10, 3, 24,
    ];

    const IP: [u8; 64] = [
        57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3, 61, 53, 45, 37, 29, 21, 13, 5,
        63, 55, 47, 39, 31, 23, 15, 7, 56, 48, 40, 32, 24, 16, 8, 0, 58, 50, 42, 34, 26, 18, 10, 2,
        60, 52, 44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6,
    ];

    const IP_INV: [u8; 64] = [
        39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30, 37, 5, 45, 13, 53, 21, 61,
        29, 36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27, 34, 2, 42, 10, 50, 18,
        58, 26, 33, 1, 41, 9, 49, 17, 57, 25, 32, 0, 40, 8, 48, 16, 56, 24,
    ];

    const KEY_PERMUTATION_TABLE: [u8; 56] = [
        56, 48, 40, 32, 24, 16, 8, 0, 57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18, 10, 2,
        59, 51, 43, 35, 62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 60, 52, 44,
        36, 28, 20, 12, 4, 27, 19, 11, 3,
    ];

    const KEY_COMPRESSION: [u8; 48] = [
        13, 16, 10, 23, 0, 4, 2, 27, 14, 5, 20, 9, 22, 18, 11, 3, 25, 7, 15, 6, 26, 19, 12, 1, 45,
        56, 35, 41, 51, 59, 34, 44, 55, 49, 37, 52, 48, 53, 43, 60, 38, 57, 50, 46, 54, 40, 33, 36,
    ];

    const KEY_EXPANSION: [u8; 48] = [
        31, 0, 1, 2, 3, 4, 3, 4, 5, 6, 7, 8, 7, 8, 9, 10, 11, 12, 11, 12, 13, 14, 15, 16, 15, 16,
        17, 18, 19, 20, 19, 20, 21, 22, 23, 24, 23, 24, 25, 26, 27, 28, 27, 28, 29, 30, 31, 0,
    ];

    fn get_cache(k: u8) -> u64 {
        let k = (k & 0x3F) as usize;
        if k < 32 {
            1u64 << (31 - k)
        } else {
            1u64 << (63 - (k - 32))
        }
    }

    fn map_bit(result: &mut u64, src: u64, check: u8, set: u8) {
        if (get_cache(check) & src) != 0 {
            *result |= get_cache(set);
        }
    }

    fn map_u32_bits(src_value: u32, table: &[u8]) -> u32 {
        let mut result = 0u64;
        for (i, &v) in table.iter().enumerate() {
            map_bit(&mut result, src_value as u64, v, i as u8);
        }
        result as u32
    }

    fn map_u64(src_value: u64, table: &[u8]) -> u64 {
        let mid_idx = table.len() / 2;
        let (table_lo, table_hi) = table.split_at(mid_idx);

        let mut lo32 = 0u64;
        let mut hi32 = 0u64;

        for (i, &v) in table_lo.iter().enumerate() {
            map_bit(&mut lo32, src_value, v, i as u8);
        }
        for (i, &v) in table_hi.iter().enumerate() {
            map_bit(&mut hi32, src_value, v, i as u8);
        }

        ((hi32 as u32 as u64) << 32) | (lo32 as u32 as u64)
    }

    fn update_param(param: &mut u32, shift_left: u8) {
        let shift_right = 28 - shift_left;
        *param = ((*param << shift_left) | ((*param >> shift_right) & 0xFFFFFFF0)) & 0xFFFFFFF0;
    }

    pub struct QqDes {
        subkeys: [u64; 16],
    }

    impl QqDes {
        pub fn new(key_bytes: &[u8; 8], mode_encrypt: bool) -> Self {
            let key = u64::from_le_bytes(*key_bytes);
            let param = map_u64(key, &KEY_PERMUTATION_TABLE);
            let mut param_c = param as u32;
            let mut param_d = (param >> 32) as u32;

            let mut subkeys = [0u64; 16];
            for (i, &shift_left) in KEY_RND_SHIFTS.iter().enumerate() {
                let subkey_idx = if mode_encrypt { i } else { 15 - i };
                update_param(&mut param_c, shift_left);
                update_param(&mut param_d, shift_left);
                let combined = ((param_d as u64) << 32) | (param_c as u64);
                subkeys[subkey_idx] = map_u64(combined, &KEY_COMPRESSION);
            }

            Self { subkeys }
        }

        pub fn transform_block(&self, data: u64) -> u64 {
            let mut state = map_u64(data, &IP);
            for &key in &self.subkeys {
                let state_hi = (state >> 32) as u32;
                let state_lo = state as u32;

                let combined = ((state_hi as u64) << 32) | (state_hi as u64);
                let expanded = map_u64(combined, &KEY_EXPANSION) ^ key;

                let mut sbox_res = 0u32;
                for (i, &shift) in LARGE_STATE_SHIFTS.iter().enumerate() {
                    let sbox_idx = ((expanded >> shift) & 0x3F) as usize;
                    sbox_res = (sbox_res << 4) | (SBOXES[i][sbox_idx] as u32);
                }

                let next_lo = map_u32_bits(sbox_res, &P_BOX) ^ state_lo;
                state = ((next_lo as u64) << 32) | (state_hi as u64);
            }
            let state = (state >> 32) | (state << 32);
            map_u64(state, &IP_INV)
        }

        pub fn transform_bytes(&self, data: &mut [u8]) {
            for chunk in data.chunks_exact_mut(8) {
                let mut block = [0u8; 8];
                block.copy_from_slice(chunk);
                let val = u64::from_le_bytes(block);
                let out = self.transform_block(val);
                chunk.copy_from_slice(&out.to_le_bytes());
            }
        }
    }
}

/// parse qrc lyric body into word-timed lines.
/// format per line: `[lineStartMs,lineDurMs]word(wordStartMs,wordDurMs)word(wordStartMs,wordDurMs)`
pub fn parse_qrc(raw: &str) -> Vec<LyricLine> {
    // extract LyricContent attribute if wrapped in xml tags
    let content = if let Some(open) = raw.find("LyricContent=\"") {
        let after = &raw[open + "LyricContent=\"".len()..];
        if let Some(close) = after.find('"') {
            unescape_xml(&after[..close])
        } else {
            raw.to_string()
        }
    } else if let Some(open) = raw.find("LyricContent='") {
        let after = &raw[open + "LyricContent='".len()..];
        if let Some(close) = after.find('\'') {
            unescape_xml(&after[..close])
        } else {
            raw.to_string()
        }
    } else {
        raw.to_string()
    };

    let mut lines = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }

        let Some(close_bracket) = line.find(']') else {
            continue;
        };
        let header = &line[1..close_bracket];
        let Some((start_s, _)) = header.split_once(',') else {
            continue; // skips non-timing metadata tags: [ti:..], [ar:..], [offset:..]
        };
        let Ok(line_start) = start_s.trim().parse::<i64>() else {
            continue;
        };

        let mut rest = &line[close_bracket + 1..];
        let mut words = Vec::new();

        // qrc has the inverse order of netease yrc: text comes BEFORE its (start,dur) pair
        while let Some(open) = rest.find('(') {
            let Some(close) = rest[open..].find(')') else { break };
            let close = open + close;
            let word_text = &rest[..open];
            let meta = &rest[open + 1..close];

            let mut parts = meta.split(',');
            let start = parts.next().and_then(|s| s.trim().parse::<i64>().ok());
            let dur = parts.next().and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0);

            if let Some(start) = start {
                let clean = scrub_spaces(word_text);
                if !clean.is_empty() {
                    words.push(LyricWord {
                        time_ms: start,
                        end_ms:  start + dur.max(0),
                        text:    clean,
                    });
                }
            }
            rest = &rest[close + 1..];
        }

        if !words.is_empty() {
            lines.push(LyricLine::worded(line_start, None, words));
        } else {
            let clean = scrub_spaces(rest).trim().to_string();
            if !clean.is_empty() {
                lines.push(LyricLine::line(line_start, clean));
            }
        }
    }

    lines.sort_by_key(|l| l.time_ms);
    lines
}

/// decode xml entities inside LyricContent attribute values
fn unescape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp..];
        if let Some(semi) = after.find(';') {
            let entity = &after[1..semi];
            match entity {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                "nbsp" => out.push(' '),
                _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                    if let Ok(cp) = u32::from_str_radix(&entity[2..], 16) {
                        if let Some(c) = char::from_u32(cp) {
                            out.push(c);
                        }
                    }
                }
                _ if entity.starts_with('#') => {
                    if let Ok(cp) = entity[1..].parse::<u32>() {
                        if let Some(c) = char::from_u32(cp) {
                            out.push(c);
                        }
                    }
                }
                _ => {
                    out.push('&');
                    out.push_str(entity);
                    out.push(';');
                }
            }
            rest = &after[semi + 1..];
        } else {
            out.push('&');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_qrc_words_text_before_timings() {
        let raw = "[1000,2000]Hello (1000,500)world(1500,500)";
        let lines = parse_qrc(raw);
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
    fn parses_xml_wrapped_qrc_and_skips_metadata() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<QrcInfos>
<LyricInfo>
<Lyric_1 LyricType="1" LyricContent="[ti:Title]&#10;[ar:Artist]&#10;[1000,2000]I (1000,300)do (1300,700)&#10;[3000,1000]what it takes"/>
</LyricInfo>
</QrcInfos>"#;
        let lines = parse_qrc(xml);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].time_ms, 1000);
        assert_eq!(lines[0].words.len(), 2);
        assert_eq!(lines[1].time_ms, 3000);
        assert_eq!(lines[1].text, "what it takes");
    }

    #[test]
    fn decrypts_qq_qrc_payload() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let plain = "[1000,2000]Test (1000,1000)";
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(plain.as_bytes()).unwrap();
        let mut compressed = enc.finish().unwrap();

        let rem = compressed.len() % 8;
        if rem != 0 {
            compressed.resize(compressed.len() + 8 - rem, 0);
        }

        // encrypt with qq_des: E(KEY3) -> D(KEY2) -> E(KEY1)
        let enc3 = qq_des::QqDes::new(KEY3, true);
        let enc2 = qq_des::QqDes::new(KEY2, false);
        let enc1 = qq_des::QqDes::new(KEY1, true);

        enc3.transform_bytes(&mut compressed);
        enc2.transform_bytes(&mut compressed);
        enc1.transform_bytes(&mut compressed);

        let hex_str: String = compressed.iter().map(|b| format!("{b:02x}")).collect();
        let decrypted = decrypt_qrc(&hex_str).expect("decrypt should succeed");
        assert_eq!(decrypted, plain);
    }
}
