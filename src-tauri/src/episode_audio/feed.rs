//! reading a podcast's rss feed and picking out the episode spotify is
//! showing. nearly every show on spotify is also published as a plain rss
//! feed (that's how it got onto spotify in the first place), and the feed's
//! `<enclosure>` is a direct link to the same audio file.
//!
//! matching is by title first, then release day and length: feeds and spotify
//! usually carry the exact same title, but some shows number their episodes
//! differently in one place, so a close title plus the same day (or the same
//! length) also counts.

#[derive(Debug, Clone, PartialEq)]
pub struct FeedItem {
    pub title:       String,
    pub url:         String,
    pub mime:        Option<String>,
    /// (year, month, day), from `<pubDate>`
    pub date:        Option<(i32, u32, u32)>,
    pub duration_ms: Option<u64>,
}

/// every playable `<item>` in the feed, in feed order
pub fn parse(xml: &str) -> Vec<FeedItem> {
    let opts = roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() };
    let Ok(doc) = roxmltree::Document::parse_with_options(xml, opts) else {
        return Vec::new();
    };
    doc.descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "item")
        .filter_map(|item| {
            // the plain rss child, not `itunes:title` / `itunes:duration`'s cousins
            let child = |name: &str, ns: Option<&str>| {
                item.children().find(|c| {
                    c.is_element()
                        && c.tag_name().name() == name
                        && match ns {
                            None => c.tag_name().namespace().is_none(),
                            Some(frag) => c.tag_name().namespace().is_some_and(|n| n.contains(frag)),
                        }
                })
            };
            let enclosure = child("enclosure", None)?;
            let url = enclosure.attribute("url")?.trim().to_string();
            if url.is_empty() {
                return None;
            }
            let title = child("title", None)
                .or_else(|| child("title", Some("itunes")))
                .and_then(|t| t.text())
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(FeedItem {
                title,
                url,
                mime: enclosure.attribute("type").map(str::to_string).filter(|m| !m.is_empty()),
                date: child("pubDate", None).and_then(|d| d.text()).and_then(parse_rfc2822_day),
                duration_ms: child("duration", Some("itunes")).and_then(|d| d.text()).and_then(parse_duration),
            })
        })
        .collect()
}

/// the day out of an rfc 2822 date ("Mon, 28 Sep 2026 08:00:00 +0000"); the
/// weekday is optional and some feeds get it wrong, so it's ignored
pub fn parse_rfc2822_day(s: &str) -> Option<(i32, u32, u32)> {
    const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let tokens: Vec<&str> = s.split(|c: char| c.is_whitespace() || c == ',').filter(|t| !t.is_empty()).collect();
    let i = tokens.iter().position(|t| t.len() <= 2 && t.chars().all(|c| c.is_ascii_digit()))?;
    let day: u32 = tokens[i].parse().ok()?;
    let month = tokens.get(i + 1)?.to_ascii_lowercase();
    let month = MONTHS.iter().position(|m| month.starts_with(m))? as u32 + 1;
    let year: i32 = tokens.get(i + 2)?.parse().ok()?;
    (1..=31).contains(&day).then_some((year, month, day))
}

/// spotify's `release_date` ("2026-09-28"; older shows can be just a year)
pub fn parse_iso_day(s: &str) -> Option<(i32, u32, u32)> {
    let mut it = s.split('-');
    let y = it.next()?.parse().ok()?;
    let m = it.next()?.parse().ok()?;
    let d = it.next()?.get(..2)?.parse().ok()?;
    Some((y, m, d))
}

/// `itunes:duration`: seconds ("3600"), "mm:ss" or "hh:mm:ss"
pub fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut total = 0u64;
    for part in s.split(':') {
        let secs: f64 = part.trim().parse().ok()?;
        total = total * 60 + secs as u64;
    }
    (total > 0).then_some(total * 1000)
}

fn days(d: (i32, u32, u32)) -> i64 {
    // days since the epoch (civil-from-days, Howard Hinnant)
    let (y, m, d) = (d.0 as i64 - if d.1 <= 2 { 1 } else { 0 }, d.1 as i64, d.2 as i64);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// lowercase words, punctuation dropped
pub fn normalize(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_lowercase().next().unwrap_or(c) } else { ' ' })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 1.0 for the same title; otherwise how many words they share (dice)
pub fn title_similarity(a: &str, b: &str) -> f64 {
    let (a, b) = (normalize(a), normalize(b));
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    if a.len() >= 12 && b.len() >= 12 && (a.contains(&b) || b.contains(&a)) {
        return 0.9;
    }
    let wa: std::collections::HashSet<&str> = a.split(' ').collect();
    let wb: std::collections::HashSet<&str> = b.split(' ').collect();
    let shared = wa.intersection(&wb).count() as f64;
    2.0 * shared / (wa.len() + wb.len()) as f64
}

fn close_length(a: u64, b: u64) -> bool {
    // ads get stitched in differently per platform, so allow a minute or 3%
    a.abs_diff(b) <= 60_000.max(a / 33)
}

/// the feed item for this episode, if one clearly is it
pub fn best_match<'a>(
    items: &'a [FeedItem],
    title: &str,
    date: Option<(i32, u32, u32)>,
    duration_ms: u64,
) -> Option<&'a FeedItem> {
    let mut best: Option<(f64, &FeedItem)> = None;
    for item in items {
        let t = title_similarity(title, &item.title);
        let day_gap = match (date, item.date) {
            (Some(a), Some(b)) => Some((days(a) - days(b)).abs()),
            _ => None,
        };
        let same_day = day_gap.is_some_and(|g| g <= 1);
        let length = match item.duration_ms {
            Some(d) if duration_ms > 0 => Some(close_length(d, duration_ms)),
            _ => None,
        };
        let accept = if t >= 0.95 {
            // an identical title is it, unless it's plainly a different
            // airing (a rerun years apart with a different cut)
            !(day_gap.is_some_and(|g| g > 7) && length == Some(false))
        } else if t >= 0.6 {
            same_day || length == Some(true)
        } else {
            t >= 0.3 && same_day && length == Some(true)
        };
        if !accept {
            continue;
        }
        let score = t * 2.0 + if same_day { 1.0 } else { 0.0 } + if length == Some(true) { 1.0 } else { 0.0 };
        if best.is_none_or(|(s, _)| score > s) {
            best = Some((score, item));
        }
    }
    best.map(|(_, item)| item)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
<channel>
  <title>Show</title>
  <item>
    <title>Episode 12: The Sleep Episode</title>
    <itunes:title>The Sleep Episode</itunes:title>
    <pubDate>Mon, 28 Sep 2026 08:00:00 +0000</pubDate>
    <itunes:duration>01:02:03</itunes:duration>
    <enclosure url="https://cdn.example.com/12.mp3" length="0" type="audio/mpeg"/>
  </item>
  <item>
    <title><![CDATA[Episode 11 & Friends]]></title>
    <pubDate>21 Sep 2026 08:00:00 GMT</pubDate>
    <itunes:duration>3600</itunes:duration>
    <enclosure url="https://cdn.example.com/11.mp3" type="audio/mpeg"/>
  </item>
  <item>
    <title>No audio here</title>
  </item>
</channel>
</rss>"#;

    #[test]
    fn parses_items() {
        let items = parse(FEED);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Episode 12: The Sleep Episode");
        assert_eq!(items[0].url, "https://cdn.example.com/12.mp3");
        assert_eq!(items[0].mime.as_deref(), Some("audio/mpeg"));
        assert_eq!(items[0].date, Some((2026, 9, 28)));
        assert_eq!(items[0].duration_ms, Some(3_723_000));
        assert_eq!(items[1].title, "Episode 11 & Friends");
        assert_eq!(items[1].date, Some((2026, 9, 21)));
        assert_eq!(items[1].duration_ms, Some(3_600_000));
    }

    #[test]
    fn junk_is_empty_not_a_panic() {
        assert!(parse("not xml").is_empty());
        assert!(parse("<rss><channel></channel></rss>").is_empty());
    }

    #[test]
    fn dates_and_durations() {
        assert_eq!(parse_iso_day("2026-09-28"), Some((2026, 9, 28)));
        assert_eq!(parse_iso_day("2019"), None);
        assert_eq!(parse_duration("45:10"), Some(2_710_000));
        assert_eq!(parse_duration(""), None);
        assert_eq!(days((1970, 1, 1)), 0);
        assert_eq!(days((2026, 3, 1)) - days((2026, 2, 28)), 1);
    }

    #[test]
    fn matches_by_title() {
        let items = parse(FEED);
        let m = best_match(&items, "Episode 12: The Sleep Episode", Some((2026, 9, 28)), 3_700_000).unwrap();
        assert_eq!(m.url, "https://cdn.example.com/12.mp3");
        // punctuation and case don't matter
        let m = best_match(&items, "episode 11 and friends", None, 0);
        assert!(m.is_none(), "'and' vs '&' alone, with nothing else agreeing, isn't enough");
        let m = best_match(&items, "Episode 11 & friends!", None, 0).unwrap();
        assert_eq!(m.url, "https://cdn.example.com/11.mp3");
    }

    #[test]
    fn renamed_episode_matches_on_day_and_length() {
        let items = parse(FEED);
        let m = best_match(&items, "#12 - Sleep", Some((2026, 9, 28)), 3_725_000);
        assert_eq!(m.map(|m| m.url.as_str()), Some("https://cdn.example.com/12.mp3"));
    }

    #[test]
    fn no_match_for_other_episodes() {
        let items = parse(FEED);
        assert!(best_match(&items, "Something else entirely", Some((2026, 1, 3)), 1_000_000).is_none());
    }
}
