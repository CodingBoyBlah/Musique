//! Apple-Music-style TTML.
//!
//! Three different providers hand us this exact format - the AMLL community
//! database, Apple's own documents relayed by the ISRC-keyed mirror, and
//! BetterLyrics - so the parser lives here rather than next to any one of them.
//!
//! It is the richest lyric format in play by a distance: per-word timings,
//! background vocals marked with `ttm:role="x-bg"`, duet voices separated by
//! `ttm:agent`, plus translations and romanisations. That structure is exactly
//! what Apple Music renders, which is why documents in this format are the ones
//! worth reaching for first.
//!
//! Hand-rolled rather than pulling in an xml crate: the subset in play is tiny
//! (`<p>`, `<span>`, five attributes) and the files are machine-written by a
//! handful of exporters, so a tolerant scanner is smaller than the dependency
//! and cannot fail on a document a strict parser would reject.

use super::parse::scrub_spaces;
use super::types::{LineRole, LyricLine, LyricWord};


/// scan a ttml document into timed lines. everything outside `<p>` elements
/// (head, metadata, agents, div wrappers) is ignored, so layout changes in the
/// exporter can't break us
pub fn parse_ttml(raw: &str) -> Vec<LyricLine> {
    let mut out: Vec<LyricLine> = Vec::new();
    // the first agent seen is the lead voice; anything else is the other half
    // of a duet. the ids aren't stable ("v1"/"v2", sometimes "singer1")
    let mut lead: Option<String> = None;
    let mut i = 0usize;

    while let Some(rel) = raw[i..].find("<p") {
        let open = i + rel;
        let Some(next) = raw[open + 2..].chars().next() else { break };
        if !next.is_whitespace() && next != '>' {
            i = open + 2; // `<pre`, `<path`, ... - not a lyric line
            continue;
        }
        let Some(gt_rel) = raw[open..].find('>') else { break };
        let gt = open + gt_rel;
        let Some(close_rel) = raw[gt..].find("</p>") else { break };
        let close = gt + close_rel;

        let tag = &raw[open + 1..gt];
        let inner = &raw[gt + 1..close];
        i = close + 4;

        let parsed = scan(inner);
        let begin = attr(tag, "begin").and_then(|v| parse_time(&v));

        let mut line = match (parsed.words.is_empty(), begin) {
            // word-timed line: the words carry the truth, `begin` is a hint
            (false, b) => {
                let start = b.unwrap_or(parsed.words[0].time_ms);
                LyricLine::worded(start, None, parsed.words)
            }
            // line-level `<p>` with bare text inside
            (true, Some(b)) => {
                let text = parsed.text.trim().to_string();
                if text.is_empty() && parsed.bg.is_none() {
                    continue;
                }
                LyricLine::line(b, text)
            }
            (true, None) => continue,
        };

        if let Some(agent) = attr(tag, "agent") {
            match &lead {
                None => lead = Some(agent),
                Some(first) if *first != agent => line.role = LineRole::Duet,
                Some(_) => {}
            }
        }
        line.translation = parsed.translation;
        line.roman = parsed.roman;
        line.bg = parsed.bg.map(Box::new);
        out.push(line);
    }

    out.sort_by_key(|l| l.time_ms);
    out
}

#[derive(Default)]
struct Parsed {
    words:       Vec<LyricWord>,
    text:        String, // only used when there are no word spans
    translation: Option<String>,
    roman:       Option<String>,
    bg:          Option<LyricLine>,
}

/// walk the children of one `<p>` (or of a bg span, which has the same shape).
/// plain `<span begin end>` children are words; the `ttm:role` ones are the
/// out-of-band tracks (background vocals, translation, romanisation)
fn scan(inner: &str) -> Parsed {
    let mut p = Parsed::default();
    // text between spans - the exporter puts the separating space *outside*
    // the span, so it has to be re-attached or every english line loses its
    // word gaps
    let mut pending = String::new();
    let mut i = 0usize;

    while i < inner.len() {
        let Some(rel) = inner[i..].find('<') else {
            pending.push_str(&inner[i..]);
            break;
        };
        pending.push_str(&inner[i..i + rel]);
        let lt = i + rel;
        let Some(gt_rel) = inner[lt..].find('>') else { break };
        let gt = lt + gt_rel;
        let tag = &inner[lt + 1..gt];
        i = gt + 1;

        let name = tag.trim_start_matches('/');
        let name = &name[..name.find(|c: char| c.is_whitespace() || c == '/').unwrap_or(name.len())];

        if name == "br" {
            pending.push(' ');
            continue;
        }
        if name != "span" || tag.starts_with('/') || tag.ends_with('/') {
            continue; // closing or self-closing tag, nothing to read
        }

        let (content_end, after) = span_content(inner, i);
        let content = &inner[i..content_end];
        i = after;

        match attr(tag, "role").as_deref() {
            Some("x-bg") => {
                if p.bg.is_none() {
                    p.bg = bg_line(tag, content);
                }
            }
            Some("x-translation") => p.translation = non_empty(plain_text(content)),
            Some("x-roman") => p.roman = non_empty(plain_text(content)),
            _ => match attr(tag, "begin").and_then(|v| parse_time(&v)) {
                Some(start) => {
                    let text = plain_text(content);
                    if text.is_empty() {
                        continue;
                    }
                    let end = attr(tag, "end").and_then(|v| parse_time(&v)).unwrap_or(start);
                    flush(&mut p, &mut pending);
                    p.words.push(LyricWord { time_ms: start, end_ms: end.max(start), text });
                }
                // an untimed wrapper span: descend into it rather than losing
                // whatever it holds
                None => {
                    let nested = scan(content);
                    flush(&mut p, &mut pending);
                    p.words.extend(nested.words);
                    p.text.push_str(&nested.text);
                    p.translation = p.translation.take().or(nested.translation);
                    p.roman = p.roman.take().or(nested.roman);
                }
            },
        }
    }

    flush(&mut p, &mut pending);
    p
}

/// whitespace that sat between two word spans belongs to the word before it -
/// the yrc parser shapes words the same way ("I " not " I"), which keeps the
/// joined line text correct and the karaoke highlight from leading by a space
fn flush(p: &mut Parsed, pending: &mut String) {
    // this text came straight from the document, unlike the word texts that
    // went through plain_text - so it still needs decoding. and xml's default
    // whitespace handling applies: a run of indentation between two spans is
    // one space, not a newline plus twelve columns
    let text = collapse_ws(&unescape(&scrub_spaces(pending)));
    pending.clear();
    if text.is_empty() {
        return;
    }
    match p.words.last_mut() {
        Some(w) => w.text.push_str(&text),
        None => p.text.push_str(&text),
    }
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            in_ws = true;
            continue;
        }
        if in_ws && !out.is_empty() {
            out.push(' ');
        }
        in_ws = false;
        out.push(c);
    }
    if in_ws && !out.is_empty() {
        out.push(' ');
    }
    // a run that was *only* whitespace still separates two words
    if out.is_empty() && !s.is_empty() {
        out.push(' ');
    }
    out
}

/// build the backing-vocal line hanging off a `<p>`
fn bg_line(tag: &str, content: &str) -> Option<LyricLine> {
    let parsed = scan(content);
    let begin = attr(tag, "begin").and_then(|v| parse_time(&v));

    let mut line = if parsed.words.is_empty() {
        LyricLine::line(begin?, non_empty(parsed.text.trim().to_string())?)
    } else {
        LyricLine::worded(begin.unwrap_or(parsed.words[0].time_ms), None, parsed.words)
    };

    line.role = LineRole::Bg;
    line.translation = parsed.translation;
    line.roman = parsed.roman;
    unwrap_parens(&mut line);
    Some(line)
}

/// bg lines are written wrapped in parens by convention - `(ooh)`. drop them
/// when they wrap the whole line, since the ui already styles backing vocals;
/// a paren in the middle of a line is real punctuation and stays
fn unwrap_parens(line: &mut LyricLine) {
    const OPEN: &[char] = &['(', '\u{ff08}'];
    const CLOSE: &[char] = &[')', '\u{ff09}'];

    // gate on the whole line, so `sing (loud) now` keeps its parens
    let text = line.text.trim().to_string();
    if !(text.starts_with(OPEN) && text.ends_with(CLOSE) && text.chars().count() > 2) {
        return;
    }

    if line.words.is_empty() {
        line.text = text.trim_start_matches(OPEN).trim_end_matches(CLOSE).trim().to_string();
        return;
    }

    if let Some(w) = line.words.first_mut() {
        w.text = w.text.trim_start().trim_start_matches(OPEN).to_string();
    }
    if let Some(w) = line.words.last_mut() {
        w.text = w.text.trim_end().trim_end_matches(CLOSE).to_string();
    }
    line.words.retain(|w| !w.text.is_empty());
    line.text = line.words.iter().map(|w| w.text.as_str()).collect::<String>().trim().to_string();
}

/// end of a span's content and the index just past its `</span>`, counting
/// nested opens so a bg span isn't closed early by its own word spans
fn span_content(s: &str, from: usize) -> (usize, usize) {
    let mut depth = 1usize;
    let mut i = from;

    loop {
        let Some(close) = s[i..].find("</span>").map(|r| i + r) else {
            return (s.len(), s.len()); // truncated file - take what's there
        };
        let open = s[i..].find("<span").map(|r| i + r).filter(|o| *o < close);

        match open {
            Some(o) => {
                // a self-closing `<span .../>` never needs a matching close
                let gt = s[o..].find('>').map(|r| o + r).unwrap_or(close);
                if !s[o..gt].ends_with('/') {
                    depth += 1;
                }
                i = gt + 1;
            }
            None => {
                depth -= 1;
                if depth == 0 {
                    return (close, close + "</span>".len());
                }
                i = close + "</span>".len();
            }
        }
    }
}

/// everything inside a span with the markup removed and entities decoded
fn plain_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;

    while i < s.len() {
        let Some(rel) = s[i..].find('<') else {
            out.push_str(&s[i..]);
            break;
        };
        out.push_str(&s[i..i + rel]);
        let lt = i + rel;
        let Some(gt_rel) = s[lt..].find('>') else { break };
        i = lt + gt_rel + 1;
    }

    unescape(&scrub_spaces(&out))
}

/// value of an attribute, matching it with or without a namespace prefix
/// (`ttm:role` and `role`). tolerant scanner because attribute order varies
/// between submissions - `ttm:role` often comes before `begin`
fn attr(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=\"");
    let mut from = 0usize;

    while let Some(rel) = tag[from..].find(&pat) {
        let at = from + rel;
        let starts_attr = at == 0
            || tag[..at].chars().next_back().map(|c| c.is_whitespace() || c == ':') == Some(true);
        let val = at + pat.len();
        let Some(end_rel) = tag[val..].find('"') else { return None };
        if starts_attr {
            return Some(unescape(&tag[val..val + end_rel]));
        }
        from = val + end_rel + 1;
    }
    None
}

/// ttml clock values, in every shape the db actually contains:
/// `HH:MM:SS.mmm`, `MM:SS.mmm`, `M:SS.mmm`, bare `SS.mmm`, and the offset form
/// `12.34s` (`250ms` / `1.5m` / `1h` come free with it)
fn parse_time(raw: &str) -> Option<i64> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }

    if !t.contains(':') {
        let num = t.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        let unit = &t[num.len()..];
        let v: f64 = num.parse().ok()?;
        let ms = match unit {
            "h" => v * 3_600_000.0,
            "m" => v * 60_000.0,
            "ms" => v,
            // frames and ticks need the document's frame rate, which we don't
            // track - better to drop the stamp than to invent one
            "f" | "t" => return None,
            _ => v * 1000.0, // "s" or a bare number of seconds
        };
        return Some(ms.round() as i64);
    }

    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let last = parts[parts.len() - 1];
    let (ss, frac) = last.split_once('.').unwrap_or((last, ""));

    let mut ms = ss.trim().parse::<i64>().ok()? * 1000 + frac_ms(frac);
    ms += parts[parts.len() - 2..parts.len() - 1]
        .first()
        .and_then(|m| m.trim().parse::<i64>().ok())
        .unwrap_or(0)
        * 60_000;
    if parts.len() == 3 {
        ms += parts[0].trim().parse::<i64>().ok()? * 3_600_000;
    }
    Some(ms)
}

/// `.5` -> 500ms, `.25` -> 250ms, `.123` -> 123ms
fn frac_ms(frac: &str) -> i64 {
    let digits: String = frac.chars().filter(|c| c.is_ascii_digit()).take(3).collect();
    let v: i64 = digits.parse().unwrap_or(0);
    match digits.len() {
        0 => 0,
        1 => v * 100,
        2 => v * 10,
        _ => v,
    }
}

/// the five named xml entities plus numeric refs. an unknown `&foo;` is left
/// alone rather than swallowed - it's more likely a lyric than an entity
fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;

    while i < s.len() {
        let Some(ch) = s[i..].chars().next() else { break };
        if ch != '&' {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let semi = s[i..].find(';').filter(|r| *r <= 12).map(|r| i + r);
        let decoded = semi.and_then(|semi| {
            let ent = &s[i + 1..semi];
            match ent {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                    u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32)
                }
                _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
                _ => None,
            }
        });

        match (decoded, semi) {
            (Some(c), Some(semi)) => {
                out.push(c);
                i = semi + 1;
            }
            _ => {
                out.push('&');
                i += 1;
            }
        }
    }
    out
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_word_level_line() {
        let raw = r#"<tt><body><div>
          <p begin="00:00:12.340" end="00:00:15.000" ttm:agent="v1">
            <span begin="00:00:12.340" end="00:00:12.600">Hello</span>
            <span begin="00:00:12.600" end="00:00:13.100">world</span>
          </p>
        </div></body></tt>"#;
        let lines = parse_ttml(raw);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].time_ms, 12_340);
        assert_eq!(lines[0].words.len(), 2);
        assert_eq!(lines[0].words[0].end_ms, 12_600);
        // the gap between the spans re-attaches to the word before it
        assert_eq!(lines[0].words[0].text, "Hello ");
        assert_eq!(lines[0].text, "Hello world");
        assert_eq!(lines[0].role, LineRole::Main);
    }

    #[test]
    fn reads_bg_translation_and_roman() {
        let raw = concat!(
            r#"<p begin="12.0" end="15.0" ttm:agent="v1">"#,
            r#"<span begin="12.0" end="12.5">Hey</span>"#,
            r#"<span ttm:role="x-bg" begin="13.0" end="13.8">"#,
            r#"<span begin="13.0" end="13.4">(ooh</span> <span begin="13.4" end="13.8">yeah)</span>"#,
            r#"</span>"#,
            r#"<span ttm:role="x-translation" xml:lang="zh-CN">&#20320;&#22909;</span>"#,
            r#"<span ttm:role="x-roman">ni hao</span>"#,
            r#"</p>"#,
        );
        let lines = parse_ttml(raw);
        assert_eq!(lines.len(), 1);

        // the role spans must not leak into the main line's words
        assert_eq!(lines[0].words.len(), 1);
        assert_eq!(lines[0].text, "Hey");
        assert_eq!(lines[0].translation.as_deref(), Some("\u{4f60}\u{597d}"));
        assert_eq!(lines[0].roman.as_deref(), Some("ni hao"));

        let bg = lines[0].bg.as_ref().expect("bg line");
        assert_eq!(bg.role, LineRole::Bg);
        assert_eq!(bg.time_ms, 13_000);
        assert_eq!(bg.words.len(), 2);
        assert_eq!(bg.text, "ooh yeah"); // wrapping parens stripped
    }

    #[test]
    fn second_agent_is_a_duet_line() {
        let raw = concat!(
            r#"<p begin="1.0" ttm:agent="v1"><span begin="1.0" end="2.0">lead</span></p>"#,
            r#"<p begin="3.0" ttm:agent="v2"><span begin="3.0" end="4.0">other</span></p>"#,
            r#"<p begin="5.0" ttm:agent="v1"><span begin="5.0" end="6.0">lead</span></p>"#,
        );
        let roles: Vec<LineRole> = parse_ttml(raw).iter().map(|l| l.role).collect();
        assert_eq!(roles, vec![LineRole::Main, LineRole::Duet, LineRole::Main]);
    }

    #[test]
    fn handles_every_timestamp_shape() {
        assert_eq!(parse_time("00:01:02.345"), Some(62_345));
        assert_eq!(parse_time("01:29.388"), Some(89_388));
        assert_eq!(parse_time("1:29.388"), Some(89_388));
        assert_eq!(parse_time("0.464"), Some(464));
        assert_eq!(parse_time("12.34s"), Some(12_340));
        assert_eq!(parse_time("250ms"), Some(250));
        assert_eq!(parse_time("1.5m"), Some(90_000));
        assert_eq!(parse_time("00:12.5"), Some(12_500)); // one-digit fraction
        assert_eq!(parse_time("00:12"), Some(12_000)); // no fraction at all
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("nonsense"), None);
    }

    #[test]
    fn line_level_p_without_spans() {
        let lines = parse_ttml(r#"<p begin="00:05.000">just a line &amp; more</p>"#);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].time_ms, 5_000);
        assert_eq!(lines[0].text, "just a line & more");
        assert!(lines[0].words.is_empty());
    }

    #[test]
    fn unescapes_entities() {
        assert_eq!(unescape("a &amp; b &lt;c&gt; &quot;d&quot; &apos;e&apos;"), "a & b <c> \"d\" 'e'");
        assert_eq!(unescape("&#72;&#105;"), "Hi");
        assert_eq!(unescape("&#x48;&#x69;"), "Hi");
        assert_eq!(unescape("rock & roll"), "rock & roll"); // bare ampersand survives
    }

    #[test]
    fn sorts_lines_and_skips_junk() {
        let raw = concat!(
            "<head><metadata><ttm:agent type=\"person\" xml:id=\"v1\"/></metadata></head>",
            r#"<p begin="00:20.000"><span begin="00:20.000" end="00:21.000">second</span></p>"#,
            r#"<p><span>no timing at all</span></p>"#,
            r#"<p begin="00:10.000"><span begin="00:10.000" end="00:11.000">first</span></p>"#,
        );
        let lines = parse_ttml(raw);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "first");
        assert_eq!(lines[1].text, "second");
    }

    #[test]
    fn empty_document_yields_nothing() {
        assert!(parse_ttml("").is_empty());
        assert!(parse_ttml("<tt><body></body></tt>").is_empty());
    }
}
