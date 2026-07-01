//! lyrics, grabbed from LRCLIB (free, open, no key) + a sqlite cache
//!
//! LRCLIB (https://lrclib.net) is a community lyrics db made for third party
//! players. gives back LRC-format synced lyrics + plain text, cleanest legal source
//! ngl, no spotify internal scraping, no paid musixmatch license. we cache every
//! lookup (misses too, just briefly) so a replay is instant + works offline

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;

// how long we trust a "no lyrics found" result before hitting the network again
// (someone might add lyrics for the track later)
const NEG_TTL_SECS: i64 = 60 * 60 * 24 * 7; // 7 days
// lrclib wants clients to say who they are
const USER_AGENT: &str = "spotify-desktop-client (Tauri; personal use)";

// public types, these cross the ipc boundary

#[derive(Debug, Clone, Serialize)]
pub struct LyricWord {
    pub time_ms: i64, // start, ms from track start
    pub end_ms:  i64, // end
    pub text:    String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LyricLine {
    pub time_ms: i64,
    pub text:    String,
    // real per word timings when the source has em (netease yrc). empty for
    // line level sources (lrclib) so the ui has no word data then
    pub words:   Vec<LyricWord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Lyrics {
    pub track_id:     String,
    pub lines:        Vec<LyricLine>, // time synced lines, empty if theres none
    pub plain:        Option<String>, // unsynced fallback text
    pub synced:       bool,
    pub word_level:   bool,           // true when the lines carry real word timings
    pub instrumental: bool,
    pub source:       String,         // netease | lrclib | none
    pub found:        bool,
}

// lrclib wire types

#[derive(Deserialize)]
struct LrcLibResp {
    #[serde(default)]
    instrumental: bool,
    #[serde(rename = "plainLyrics", default)]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics", default)]
    synced_lyrics: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

// lrc parsing

// parse an lrc body into time sorted lines. handles multiple timestamps on one
// line ([00:12.00][01:30.50] text) and skips the metadata tags ([ar:...])
pub fn parse_lrc(raw: &str) -> Vec<LyricLine> {
    let mut out: Vec<LyricLine> = Vec::new();

    for line in raw.lines() {
        let mut rest = line;
        let mut stamps: Vec<i64> = Vec::new();
        let mut is_metadata = false;

        loop {
            let r = rest.trim_start();
            if !r.starts_with('[') {
                rest = r;
                break;
            }
            let Some(end) = r.find(']') else { rest = r; break; };
            let tag = &r[1..end];
            rest = &r[end + 1..];
            match parse_stamp(tag) {
                Some(ms) => stamps.push(ms),
                None => { is_metadata = true; break; } // [ar:..]/[ti:..]/etc stuff
            }
        }

        if is_metadata || stamps.is_empty() {
            continue;
        }
        let text = rest.trim().to_string();
        for ms in stamps {
            out.push(LyricLine { time_ms: ms, text: text.clone(), words: Vec::new() });
        }
    }

    out.sort_by_key(|l| l.time_ms);
    out
}

// mm:ss.xx -> millis. None for tags that arent timestamps
fn parse_stamp(tag: &str) -> Option<i64> {
    let (mm, rest) = tag.split_once(':')?;
    let mm: i64 = mm.trim().parse().ok()?;
    let (ss, frac) = rest.split_once('.').unwrap_or((rest, "0"));
    let ss: i64 = ss.trim().parse().ok()?;
    let digits: String = frac.trim().chars().take(3).collect();
    let val: i64 = digits.parse().ok()?;
    let frac_ms = match digits.len() {
        0 => 0,
        1 => val * 100,
        2 => val * 10,
        _ => val,
    };
    Some(mm * 60_000 + ss * 1000 + frac_ms)
}

// yrc (netease word-by-word) parsing

// parse a netease yrc body into lines with REAL per word timings
// format per line: [lineStart,lineDur](wStart,wDur,0)word(wStart,wDur,0)word...
// where every time is absolute ms. json metadata lines (start with {) get
// skipped. this is the actual sung timing, no guessing
pub fn parse_yrc(raw: &str) -> Vec<LyricLine> {
    let mut out: Vec<LyricLine> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('{') {
            continue;
        }

        let mut rest = line;
        let mut line_start: Option<i64> = None;

        // optional [start,dur] line header thing
        if rest.starts_with('[') {
            if let Some(end) = rest.find(']') {
                let inner = &rest[1..end];
                line_start = inner.split(',').next().and_then(|s| s.trim().parse().ok());
                rest = &rest[end + 1..];
            }
        }

        // (start,dur,0)text tokens. times/parens are ascii so byte slicing is
        // safe even across multibyte word text
        let mut words: Vec<LyricWord> = Vec::new();
        loop {
            let Some(open) = rest.find('(') else { break };
            let Some(close_rel) = rest[open..].find(')') else { break };
            let close = open + close_rel;

            let mut meta = rest[open + 1..close].split(',');
            let start: Option<i64> = meta.next().and_then(|s| s.trim().parse().ok());
            let dur:   i64         = meta.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);

            let after    = &rest[close + 1..];
            let text_end = after.find('(').unwrap_or(after.len());
            let word     = &after[..text_end];

            if let Some(start) = start {
                if !word.is_empty() {
                    words.push(LyricWord { time_ms: start, end_ms: start + dur.max(0), text: word.to_string() });
                }
            }
            rest = &after[text_end..];
        }

        if words.is_empty() {
            continue;
        }
        let time_ms = line_start.unwrap_or(words[0].time_ms);
        let text    = words.iter().map(|w| w.text.as_str()).collect::<String>().trim().to_string();
        out.push(LyricLine { time_ms, text, words });
    }

    out.sort_by_key(|l| l.time_ms);
    out
}

// musixmatch richsync parsing

// parse a musixmatch richsync_body (json string) into word timed lines
// shape: [{"ts":9.71,"te":13.2,"l":[{"c":"word ","o":0.0},...],"x":"full line"}]
// ts/te are the line start/end in seconds, each `l` chunk is a word fragment with
// offset `o` seconds from ts. real sung timing, massive catalogue
pub fn parse_richsync(raw: &str) -> Vec<LyricLine> {
    let Ok(arr) = serde_json::from_str::<serde_json::Value>(raw) else { return Vec::new() };
    let Some(arr) = arr.as_array() else { return Vec::new() };

    let mut out: Vec<LyricLine> = Vec::new();
    for line in arr {
        let ts = line.get("ts").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let te = line.get("te").and_then(|v| v.as_f64()).unwrap_or(ts);
        let Some(chunks) = line.get("l").and_then(|v| v.as_array()) else { continue };

        let mut words: Vec<LyricWord> = Vec::new();
        for (i, ch) in chunks.iter().enumerate() {
            let c = ch.get("c").and_then(|v| v.as_str()).unwrap_or("");
            if c.is_empty() { continue; }
            let o      = ch.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let next_o = chunks.get(i + 1).and_then(|n| n.get("o")).and_then(|v| v.as_f64());
            let start  = ((ts + o) * 1000.0).round() as i64;
            let end    = (next_o.map(|no| ts + no).unwrap_or(te) * 1000.0).round() as i64;
            words.push(LyricWord { time_ms: start, end_ms: end.max(start), text: c.to_string() });
        }
        if words.is_empty() { continue; }

        let text = line.get("x").and_then(|v| v.as_str()).map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| words.iter().map(|w| w.text.as_str()).collect::<String>().trim().to_string());

        out.push(LyricLine { time_ms: (ts * 1000.0).round() as i64, text, words });
    }
    out.sort_by_key(|l| l.time_ms);
    out
}

// network stuff

const NE_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Deserialize)]
struct NeSearch { result: Option<NeResult> }
#[derive(Deserialize)]
struct NeResult { #[serde(default)] songs: Vec<NeSong> }
#[derive(Deserialize)]
struct NeSong {
    id: i64,
    #[serde(default)] duration: i64, // ms
}
#[derive(Deserialize)]
struct NeLyric { #[serde(default)] yrc: Option<NeLyricBody> }
#[derive(Deserialize)]
struct NeLyricBody { #[serde(default)] lyric: Option<String> }

// best effort netease word level lookup. any failure (network, no match, no
// yrc) just returns None so the caller falls back to lrclib, never errors the whole
// request. matches by closest duration within 6s so we dont grab the wrong song
async fn fetch_netease_yrc(
    client: &reqwest::Client,
    name: &str, artist: &str, dur_ms: i64,
) -> Option<String> {
    let query = format!("{name} {artist}");

    let mut search_url = url::Url::parse("https://music.163.com/api/search/get").ok()?;
    search_url.query_pairs_mut()
        .append_pair("s", &query)
        .append_pair("type", "1")
        .append_pair("limit", "10");

    let body = client
        .get(search_url)
        .header("User-Agent", NE_UA)
        .header("Referer", "https://music.163.com")
        .header("Cookie", "os=pc; appver=8.9.70")
        .send().await.ok()?
        .text().await.ok()?;

    let songs = serde_json::from_str::<NeSearch>(&body).ok()?.result?.songs;

    let mut best: Option<(i64, i64)> = None; // (gap, id)
    for s in &songs {
        let gap = (s.duration - dur_ms).abs();
        if gap <= 6000 && best.map(|(b, _)| gap < b).unwrap_or(true) {
            best = Some((gap, s.id));
        }
    }
    let id = best?.1;

    // interface3 host reliably serves the word level yrc field
    let mut lyric_url = url::Url::parse("https://interface3.music.163.com/api/song/lyric/v1").ok()?;
    lyric_url.query_pairs_mut()
        .append_pair("id", &id.to_string())
        .append_pair("cp", "false")
        .append_pair("lv", "1").append_pair("kv", "1").append_pair("tv", "1")
        .append_pair("yv", "1").append_pair("ytv", "1").append_pair("yrc", "1");

    let lbody = client
        .get(lyric_url)
        .header("User-Agent", NE_UA)
        .header("Referer", "https://music.163.com")
        .header("Cookie", "os=pc; appver=8.9.70")
        .send().await.ok()?
        .text().await.ok()?;

    let yrc = serde_json::from_str::<NeLyric>(&lbody).ok()?.yrc?.lyric?;
    if yrc.trim().is_empty() { None } else { Some(yrc) }
}

// best effort musixmatch word level (richsync) lookup, the biggest karaoke db.
// uses the public web desktop app token. the client MUST have a cookie store
// enabled, token.get sets the x-mxm-* cookies the macro call needs otherwise it
// 401s "renew". any failure just gives None and falls back
async fn fetch_musixmatch(
    client: &reqwest::Client,
    name: &str, artist: &str, dur_sec: i64,
) -> Option<String> {
    // 1. token, carried forward thru the shared cookie store
    let tok_body = client
        .get("https://apic-desktop.musixmatch.com/ws/1.1/token.get?app_id=web-desktop-app-v1.0&format=json")
        .header("User-Agent", NE_UA)
        .send().await.ok()?
        .text().await.ok()?;
    let tok_json: serde_json::Value = serde_json::from_str(&tok_body).ok()?;
    let token = tok_json["message"]["body"]["user_token"].as_str()?;
    if token.is_empty() || token.starts_with("Upgrade") {
        return None;
    }

    // 2. richsync thru the macro endpoint, matched by track/artist/duration
    let mut url = url::Url::parse("https://apic-desktop.musixmatch.com/ws/1.1/macro.subtitles.get").ok()?;
    url.query_pairs_mut()
        .append_pair("format", "json")
        .append_pair("namespace", "lyrics_richsynced")
        .append_pair("subtitle_format", "mxm")
        .append_pair("app_id", "web-desktop-app-v1.0")
        .append_pair("usertoken", token)
        .append_pair("q_track", name)
        .append_pair("q_artist", artist)
        .append_pair("q_duration", &dur_sec.to_string());

    let body = client
        .get(url)
        .header("User-Agent", NE_UA)
        .send().await.ok()?
        .text().await.ok()?;

    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    let rich = &v["message"]["body"]["macro_calls"]["track.richsync.get"]["message"]["body"]["richsync"]["richsync_body"];
    let rich = rich.as_str()?;
    if rich.trim().is_empty() || parse_richsync(rich).is_empty() { None } else { Some(rich.to_string()) }
}


async fn fetch_get(
    client: &reqwest::Client,
    track: &str, artist: &str, album: &str, dur_sec: i64,
) -> Result<Option<LrcLibResp>, AppError> {
    let mut url = url::Url::parse("https://lrclib.net/api/get").unwrap();
    url.query_pairs_mut()
        .append_pair("track_name",  track)
        .append_pair("artist_name", artist)
        .append_pair("album_name",  album)
        .append_pair("duration",    &dur_sec.to_string());

    let resp = client.get(url).header("User-Agent", USER_AGENT).send().await?;
    if !resp.status().is_success() {
        return Ok(None); // 404 means no exact match so fall thru to search
    }
    let text = resp.text().await.map_err(|e| AppError::Network(e.to_string()))?;
    Ok(serde_json::from_str::<LrcLibResp>(&text).ok())
}

async fn fetch_search(
    client: &reqwest::Client,
    track: &str, artist: &str, dur_sec: i64,
) -> Result<Option<LrcLibResp>, AppError> {
    let mut url = url::Url::parse("https://lrclib.net/api/search").unwrap();
    url.query_pairs_mut()
        .append_pair("track_name",  track)
        .append_pair("artist_name", artist);

    let resp = client.get(url).header("User-Agent", USER_AGENT).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let text    = resp.text().await.map_err(|e| AppError::Network(e.to_string()))?;
    let results: Vec<LrcLibResp> = serde_json::from_str(&text).unwrap_or_default();

    // grab the closest duration, really preferring entries that actually have
    // synced lyrics
    let mut best: Option<(f64, LrcLibResp)> = None;
    for r in results {
        if r.synced_lyrics.is_none() && r.plain_lyrics.is_none() && !r.instrumental {
            continue;
        }
        let dur_gap = (r.duration.unwrap_or(0.0) - dur_sec as f64).abs();
        let score   = if r.synced_lyrics.is_some() { dur_gap } else { dur_gap + 600.0 };
        if best.as_ref().map(|(b, _)| score < *b).unwrap_or(true) {
            best = Some((score, r));
        }
    }
    Ok(best.map(|(_, r)| r))
}

// cache stuff

type CacheRow = (Option<String>, Option<String>, String, i64, i64, i64);

async fn read_cache(pool: &SqlitePool, track_id: &str) -> Option<CacheRow> {
    sqlx::query_as::<_, CacheRow>(
        "SELECT synced_lrc, plain, source, instrumental, found, fetched_at
         FROM lyrics WHERE track_id = ?",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

#[allow(clippy::too_many_arguments)]
async fn write_cache(
    pool: &SqlitePool, track_id: &str,
    synced: Option<&str>, plain: Option<&str>,
    source: &str, instrumental: bool, found: bool,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO lyrics (track_id, synced_lrc, plain, source, instrumental, found, fetched_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(track_id) DO UPDATE SET
            synced_lrc   = excluded.synced_lrc,
            plain        = excluded.plain,
            source       = excluded.source,
            instrumental = excluded.instrumental,
            found        = excluded.found,
            fetched_at   = excluded.fetched_at",
    )
    .bind(track_id)
    .bind(synced)
    .bind(plain)
    .bind(source)
    .bind(instrumental as i64)
    .bind(found as i64)
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}

fn build(track_id: &str, synced: Option<String>, plain: Option<String>, instrumental: bool, found: bool, source: &str) -> Lyrics {
    // each word level source stores its own body format, lrclib stores lrc
    let lines = match (source, synced.as_deref()) {
        ("musixmatch", Some(s)) => parse_richsync(s),
        ("netease",    Some(s)) => parse_yrc(s),
        (_,            Some(s)) => parse_lrc(s),
        (_,            None)    => Vec::new(),
    };
    let word_level = lines.iter().any(|l| !l.words.is_empty());
    Lyrics {
        track_id:     track_id.to_string(),
        synced:       !lines.is_empty(),
        word_level,
        lines,
        plain,
        instrumental,
        source:       source.to_string(),
        found,
    }
}

// entry point, this is the main one

#[allow(clippy::too_many_arguments)]
pub async fn get_or_fetch(
    pool: &SqlitePool,
    track_id: &str,
    name: &str,
    artist: &str,
    album: &str,
    duration_ms: i64,
    force: bool,
) -> Result<Lyrics, AppError> {
    if !force {
        if let Some((synced, plain, source, instrumental, found, fetched_at)) = read_cache(pool, track_id).await {
            let fresh = found == 1 || (now() - fetched_at) < NEG_TTL_SECS;
            if fresh {
                return Ok(build(track_id, synced, plain, instrumental == 1, found == 1, &source));
            }
        }
    }

    if name.trim().is_empty() || artist.trim().is_empty() {
        return Ok(build(track_id, None, None, false, false, "none"));
    }

    // cookie store on so musixmatchs token cookies carry over to its macro call
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(9))
        .cookie_store(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let dur_sec = (duration_ms as f64 / 1000.0).round() as i64;

    // provider chain, best word-by-word coverage first:
    //   1. musixmatch richsync - biggest karaoke db (word level)
    //   2. netease yrc         - word level fallback
    //   3. lrclib              - line level only (NO guessing, line by line ui)
    let mut synced: Option<String> = None;
    let mut plain:  Option<String> = None;
    let mut instrumental = false;
    let mut found  = false;
    let mut source = "none";

    // 1. musixmatch (word level)
    if let Some(rich) = fetch_musixmatch(&client, name, artist, dur_sec).await {
        synced = Some(rich);
        found  = true;
        source = "musixmatch";
    }

    // 2. netease yrc (word level)
    if synced.is_none() {
        if let Some(yrc) = fetch_netease_yrc(&client, name, artist, duration_ms).await {
            if !parse_yrc(&yrc).is_empty() {
                synced = Some(yrc);
                found  = true;
                source = "netease";
            }
        }
    }

    // 3. lrclib line level fallback
    if synced.is_none() {
        let resp = match fetch_get(&client, name, artist, album, dur_sec).await? {
            Some(r) => Some(r),
            None    => fetch_search(&client, name, artist, dur_sec).await?,
        };
        if let Some(r) = resp {
            let s = r.synced_lyrics.filter(|s| !s.trim().is_empty());
            let p = r.plain_lyrics.filter(|s| !s.trim().is_empty());
            let f = r.instrumental || s.is_some() || p.is_some();
            synced       = s;
            plain        = p;
            instrumental = r.instrumental;
            found        = f;
            source       = if f { "lrclib" } else { "none" };
        }
    }

    write_cache(pool, track_id, synced.as_deref(), plain.as_deref(), source, instrumental, found).await?;
    Ok(build(track_id, synced, plain, instrumental, found, source))
}

#[cfg(test)]
mod yrc_tests {
    use super::*;

    #[test]
    fn parses_word_timings() {
        let raw = "{\"t\":0,\"c\":[{\"tx\":\"meta\"}]}\n[630,1950](630,180,0)I (810,360,0)do (1170,210,0)what (1380,90,0)it (1470,1110,0)takes";
        let lines = parse_yrc(raw);
        assert_eq!(lines.len(), 1);
        let l = &lines[0];
        assert_eq!(l.time_ms, 630);
        assert_eq!(l.text, "I do what it takes");
        assert_eq!(l.words.len(), 5);
        assert_eq!(l.words[0].text, "I ");
        assert_eq!(l.words[0].time_ms, 630);
        assert_eq!(l.words[0].end_ms, 810);
        assert_eq!(l.words[4].text, "takes");
        assert_eq!(l.words[4].end_ms, 2580);
    }

    #[test]
    fn handles_cjk_without_spaces() {
        let raw = "[0,500](0,250,0)\u{4f60}(250,250,0)\u{597d}";
        let lines = parse_yrc(raw);
        assert_eq!(lines[0].text, "\u{4f60}\u{597d}");
        assert_eq!(lines[0].words.len(), 2);
    }

    #[test]
    fn parses_richsync_words() {
        let raw = r#"[{"ts":9.71,"te":11.0,"l":[{"c":"I'm ","o":0.0},{"c":"in ","o":0.4},{"c":"love","o":0.8}],"x":"I'm in love"}]"#;
        let lines = parse_richsync(raw);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "I'm in love");
        assert_eq!(lines[0].words.len(), 3);
        assert_eq!(lines[0].words[0].time_ms, 9710);
        assert_eq!(lines[0].words[0].end_ms, 10110); // 9.71 + 0.4 lol
        assert_eq!(lines[0].words[2].time_ms, 10510);
        assert_eq!(lines[0].words[2].end_ms, 11000); // last one just uses te
    }
}
