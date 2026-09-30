// YouTube Music search, restricted to the "Songs" tab.
//
// The songs filter matters for correctness, not just tidiness: it restricts
// results to Art Tracks (`MUSIC_VIDEO_TYPE_ATV`), YouTube's auto-generated
// official audio uploads that are tied to a real release. Unfiltered search
// mixes in music videos, live sets, fan uploads and lyric videos, all of which
// are the wrong audio even when the title is a perfect match.
//
// Each row is parsed by walking its runs and keying off the *endpoints* rather
// than the rendered text. Artists carry a `UC…` browseId, albums carry an
// `MPRE…` browseId, and duration is the run shaped like `3:22`. Splitting the
// column on the "·" separator would be shorter and would break on any artist
// or album name containing that character.

use serde_json::json;

use crate::errors::AppError;

use super::{client::WEB_REMIX, innertube};

/// Opaque InnerTube filter token for the Songs tab.
const PARAMS_SONGS: &str = "EgWKAQIIAWoKEAoQCRADEAUQBA%3D%3D";

#[derive(Debug, Clone)]
pub struct SongResult {
    pub video_id:    String,
    pub title:       String,
    pub artists:     Vec<String>,
    pub album:       Option<String>,
    pub duration_ms: Option<u64>,
    pub explicit:    bool,
    /// `MUSIC_VIDEO_TYPE_ATV` for an Art Track. Retained because the filter is
    /// a YouTube-side behaviour we do not control and should verify, not trust.
    pub video_type:  Option<String>,
}

/// Search YouTube Music for songs matching `query`.
pub async fn songs(query: &str) -> Result<Vec<SongResult>, AppError> {
    let body = json!({ "query": query, "params": PARAMS_SONGS });
    let res = innertube::post(&WEB_REMIX, "search", body).await?;

    let mut rows = Vec::new();
    innertube::find_all(&res, "musicResponsiveListItemRenderer", &mut rows);

    Ok(rows.into_iter().filter_map(parse_row).collect())
}

/// Opaque InnerTube filter token for the Episodes tab (podcast episodes).
const PARAMS_EPISODES: &str = "EgWKAQJIAWoKEAoQCRADEAUQBA%3D%3D";

/// Search YouTube Music's podcast episodes. Rows come back in the same shape
/// as songs; `artists` is usually empty (the show isn't an artist) and the
/// duration is often missing from the row, so callers check the video itself.
pub async fn episodes(query: &str) -> Result<Vec<SongResult>, AppError> {
    let body = json!({ "query": query, "params": PARAMS_EPISODES });
    let res = innertube::post(&WEB_REMIX, "search", body).await?;

    let mut rows = Vec::new();
    innertube::find_all(&res, "musicResponsiveListItemRenderer", &mut rows);
    innertube::find_all(&res, "musicMultiRowListItemRenderer", &mut rows);

    Ok(rows.into_iter().filter_map(parse_row).collect())
}

fn parse_row(row: &serde_json::Value) -> Option<SongResult> {
    // The watchEndpoint is what actually plays; a row without one is a header
    // or a navigation shelf, not a song.
    let watch = innertube::find_first(row, "watchEndpoint")?;
    let video_id = watch.get("videoId")?.as_str()?.to_string();

    let video_type = watch
        .get("watchEndpointMusicSupportedConfigs")
        .and_then(|c| c.get("watchEndpointMusicConfig"))
        .and_then(|c| c.get("musicVideoType"))
        .and_then(|t| t.as_str())
        .map(String::from);

    let columns: Vec<&serde_json::Value> = row
        .get("flexColumns")?
        .as_array()?
        .iter()
        .filter_map(|c| c.get("musicResponsiveListItemFlexColumnRenderer")?.get("text"))
        .collect();

    let title = innertube::runs_text(columns.first()?);
    if title.is_empty() {
        return None;
    }

    let mut artists = Vec::new();
    let mut album = None;
    let mut duration_ms = None;

    // Everything after the title column: artist/album/duration live here, and
    // which column they land in varies, so scan them all.
    for col in columns.iter().skip(1) {
        let Some(runs) = col.get("runs").and_then(|r| r.as_array()) else { continue };
        for run in runs {
            let Some(text) = run.get("text").and_then(|t| t.as_str()) else { continue };
            let text = text.trim();
            if text.is_empty() || text == "\u{00b7}" {
                continue;
            }

            let browse_id = run
                .get("navigationEndpoint")
                .and_then(|n| n.get("browseEndpoint"))
                .and_then(|b| b.get("browseId"))
                .and_then(|b| b.as_str());

            match browse_id {
                Some(id) if id.starts_with("UC") => artists.push(text.to_string()),
                Some(id) if id.starts_with("MPRE") => album = Some(text.to_string()),
                _ => {
                    if let Some(ms) = parse_duration(text) {
                        duration_ms = Some(ms);
                    }
                }
            }
        }
    }

    let explicit = {
        let mut badges = Vec::new();
        innertube::find_all(row, "musicInlineBadgeRenderer", &mut badges);
        badges.iter().any(|b| {
            innertube::find_first(b, "icon")
                .and_then(|i| i.get("iconType"))
                .and_then(|t| t.as_str())
                .is_some_and(|t| t.contains("EXPLICIT"))
        })
    };

    Some(SongResult { video_id, title, artists, album, duration_ms, explicit, video_type })
}

/// Parse `m:ss` or `h:mm:ss` into milliseconds.
fn parse_duration(text: &str) -> Option<u64> {
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let mut total = 0u64;
    for p in &parts {
        // Reject "1:2b" and similar so stray text can't parse as a duration.
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        total = total * 60 + p.parse::<u64>().ok()?;
    }
    Some(total * 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_durations() {
        assert_eq!(parse_duration("3:22"), Some(202_000));
        assert_eq!(parse_duration("1:02:03"), Some(3_723_000));
        assert_eq!(parse_duration("0:45"), Some(45_000));
    }

    #[test]
    fn rejects_non_durations() {
        // "2020" (a year) and "3.6B plays" must never read as a duration.
        assert_eq!(parse_duration("2020"), None);
        assert_eq!(parse_duration("3.6B plays"), None);
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("1:2:3:4"), None);
        assert_eq!(parse_duration("a:bb"), None);
    }

    #[test]
    fn parses_a_real_row() {
        // Trimmed to the shape the parser relies on.
        let row = json!({
            "flexColumns": [
                { "musicResponsiveListItemFlexColumnRenderer": {
                    "text": { "runs": [{ "text": "Blinding Lights" }] } } },
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                    { "text": "The Weeknd", "navigationEndpoint": {
                        "browseEndpoint": { "browseId": "UClYV6hHlupm_S_ObS1W-DYw" } } },
                    { "text": " \u{00b7} " },
                    { "text": "After Hours", "navigationEndpoint": {
                        "browseEndpoint": { "browseId": "MPREb_4U7yfKKFZLv" } } },
                    { "text": " \u{00b7} " },
                    { "text": "3:22" }
                ] } } }
            ],
            "badges": [{ "musicInlineBadgeRenderer": {
                "icon": { "iconType": "MUSIC_EXPLICIT_BADGE" } } }],
            "overlay": { "watchEndpoint": {
                "videoId": "J7p4bzqLvCw",
                "watchEndpointMusicSupportedConfigs": { "watchEndpointMusicConfig": {
                    "musicVideoType": "MUSIC_VIDEO_TYPE_ATV" } } } }
        });

        let r = parse_row(&row).expect("row should parse");
        assert_eq!(r.video_id, "J7p4bzqLvCw");
        assert_eq!(r.title, "Blinding Lights");
        assert_eq!(r.artists, vec!["The Weeknd"]);
        assert_eq!(r.album.as_deref(), Some("After Hours"));
        assert_eq!(r.duration_ms, Some(202_000));
        assert!(r.explicit);
        assert_eq!(r.video_type.as_deref(), Some("MUSIC_VIDEO_TYPE_ATV"));
    }

    #[test]
    fn skips_rows_without_a_watch_endpoint() {
        let row = json!({ "flexColumns": [
            { "musicResponsiveListItemFlexColumnRenderer": {
                "text": { "runs": [{ "text": "Some header" }] } } }] });
        assert!(parse_row(&row).is_none());
    }
}
