//! shared lyric-body parsers.
//!
//! every provider hands back some text format; these turn them into
//! `Vec<LyricLine>` with absolute-ms timings. format-specific parsers that only
//! one provider uses (ttml, qrc, krc) live next to that provider instead.

use super::types::{LyricLine, LyricWord};

// lrc (line level)

/// parse an lrc body into time-sorted lines. handles multiple timestamps on one
/// line (`[00:12.00][01:30.50] text`) and skips the metadata tags (`[ar:...]`)
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
            let Some(end) = r.find(']') else {
                rest = r;
                break;
            };
            let tag = &r[1..end];
            rest = &r[end + 1..];
            match parse_stamp(tag) {
                Some(ms) => stamps.push(ms),
                None => {
                    is_metadata = true;
                    break;
                } // [ar:..]/[ti:..]/etc
            }
        }

        if is_metadata || stamps.is_empty() {
            continue;
        }
        let text = rest.trim().to_string();
        for ms in stamps {
            out.push(LyricLine::line(ms, text.clone()));
        }
    }

    out.sort_by_key(|l| l.time_ms);
    out
}

/// `mm:ss.xx` -> millis. `None` for tags that aren't timestamps
pub fn parse_stamp(tag: &str) -> Option<i64> {
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

/// merge a parallel lrc body (netease `tlyric` / `romalrc`) onto already-parsed
/// lines by nearest timestamp. the secondary body has the same line onsets as
/// the primary, so a tight tolerance is enough and never mismatches
pub fn merge_parallel(lines: &mut [LyricLine], secondary: &str, into_roman: bool) {
    let extra = parse_lrc(secondary);
    if extra.is_empty() {
        return;
    }
    const TOL_MS: i64 = 350;

    for l in lines.iter_mut() {
        // nearest secondary line by onset
        let mut best: Option<(i64, &LyricLine)> = None;
        for e in &extra {
            let gap = (e.time_ms - l.time_ms).abs();
            if gap <= TOL_MS && best.map(|(b, _)| gap < b).unwrap_or(true) {
                best = Some((gap, e));
            }
        }
        let Some((_, e)) = best else { continue };
        if e.text.trim().is_empty() || e.text.trim() == l.text.trim() {
            continue;
        }
        if into_roman {
            l.roman = Some(e.text.clone());
        } else {
            l.translation = Some(e.text.clone());
        }
    }
}

// yrc (netease word-by-word)

/// parse a netease yrc body into lines with REAL per-word timings.
/// format per line: `[lineStart,lineDur](wStart,wDur,0)word(wStart,wDur,0)word...`
/// where every time is absolute ms. json metadata lines (starting `{`) get
/// skipped. this is the actual sung timing, no guessing
pub fn parse_yrc(raw: &str) -> Vec<LyricLine> {
    let mut out: Vec<LyricLine> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('{') {
            continue;
        }

        let mut rest = line;
        let mut line_start: Option<i64> = None;

        // optional [start,dur] line header
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
            let dur: i64 = meta.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);

            let after = &rest[close + 1..];
            let text_end = after.find('(').unwrap_or(after.len());
            let word = &after[..text_end];

            if let Some(start) = start {
                let clean_word = scrub_spaces(word);
                if !clean_word.is_empty() {
                    words.push(LyricWord {
                        time_ms: start,
                        end_ms:  start + dur.max(0),
                        text:    clean_word,
                    });
                }
            }
            rest = &after[text_end..];
        }

        if words.is_empty() {
            continue;
        }
        let time_ms = line_start.unwrap_or(words[0].time_ms);
        out.push(LyricLine::worded(time_ms, None, words));
    }

    out.sort_by_key(|l| l.time_ms);
    out
}

// musixmatch richsync

/// parse a musixmatch `richsync_body` (json string) into word-timed lines.
/// shape: `[{"ts":9.71,"te":13.2,"l":[{"c":"word ","o":0.0},...],"x":"full line"}]`
/// ts/te are the line start/end in seconds, each `l` chunk is a word fragment
/// with offset `o` seconds from ts. real sung timing, massive catalogue
pub fn parse_richsync(raw: &str) -> Vec<LyricLine> {
    let Ok(arr) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let Some(arr) = arr.as_array() else {
        return Vec::new();
    };

    let mut out: Vec<LyricLine> = Vec::new();
    for line in arr {
        let ts = line.get("ts").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let te = line.get("te").and_then(|v| v.as_f64()).unwrap_or(ts);
        let Some(chunks) = line.get("l").and_then(|v| v.as_array()) else {
            continue;
        };

        let mut words: Vec<LyricWord> = Vec::new();
        for (i, ch) in chunks.iter().enumerate() {
            let c = ch.get("c").and_then(|v| v.as_str()).unwrap_or("");
            if c.is_empty() {
                continue;
            }
            let o = ch.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let next_o = chunks.get(i + 1).and_then(|n| n.get("o")).and_then(|v| v.as_f64());
            let start = ((ts + o) * 1000.0).round() as i64;
            let end = (next_o.map(|no| ts + no).unwrap_or(te) * 1000.0).round() as i64;
            words.push(LyricWord { time_ms: start, end_ms: end.max(start), text: c.to_string() });
        }
        if words.is_empty() {
            continue;
        }

        let text = line.get("x").and_then(|v| v.as_str()).map(|s| s.trim().to_string());
        out.push(LyricLine::worded((ts * 1000.0).round() as i64, text, words));
    }
    out.sort_by_key(|l| l.time_ms);
    out
}

// helpers

/// providers love exotic unicode spaces; normalise them so word joining and
/// trimming behave
pub fn scrub_spaces(s: &str) -> String {
    s.replace('\u{a0}', " ").replace('\u{202f}', " ").replace('\u{feff}', "")
}

/// drop leading/trailing empty lines and any line whose text is a bare
/// provider watermark
pub fn strip_noise(mut lines: Vec<LyricLine>) -> Vec<LyricLine> {
    const NOISE: [&str; 6] = [
        "lyrics licensed",
        "lyrics provided by",
        "作词",
        "作曲",
        "编曲",
        "制作人",
    ];
    lines.retain(|l| {
        let t = l.text.trim();
        if t.is_empty() {
            return !l.words.is_empty();
        }
        let low = t.to_ascii_lowercase();
        !NOISE.iter().any(|n| low.starts_with(n) || t.starts_with(n))
    });
    lines
}

#[cfg(test)]
mod tests {
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
        assert_eq!(lines[0].words[0].end_ms, 10110);
        assert_eq!(lines[0].words[2].time_ms, 10510);
        assert_eq!(lines[0].words[2].end_ms, 11000); // last one just uses te
    }

    #[test]
    fn parses_lrc_lines() {
        let lines = parse_lrc("[ar:Someone]\n[00:12.34]hello\n[00:15.00][00:20.50]twice");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].time_ms, 12_340);
        assert_eq!(lines[0].text, "hello");
        assert_eq!(lines[2].time_ms, 20_500);
    }

    #[test]
    fn merges_translation_by_timestamp() {
        let mut lines = parse_lrc("[00:10.00]hello\n[00:20.00]world");
        merge_parallel(&mut lines, "[00:10.00]bonjour\n[00:20.00]monde", false);
        assert_eq!(lines[0].translation.as_deref(), Some("bonjour"));
        assert_eq!(lines[1].translation.as_deref(), Some("monde"));
    }
}
