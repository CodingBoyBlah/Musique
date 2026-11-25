//! Apple Music URL parser (`music.apple.com`, `geo.music.apple.com`, `itunes.apple.com`).

use url::Url;

const APPLE_HOSTS: &[&str] = &["music.apple.com", "geo.music.apple.com", "itunes.apple.com"];

/// Parses an Apple Music link into `("track" | "album" | "artist" | "playlist", id)`.
///
/// An album link carrying `?i=<song_id>` resolves to `("track", song_id)`.
pub fn parse(raw: &str) -> Option<(&'static str, String)> {
    let url = Url::parse(raw.trim()).ok()?;
    if !matches!(url.scheme(), "https" | "http" | "music" | "itms" | "itmss") {
        return None;
    }

    let host = url.host_str()?;
    let bare_host = host
        .strip_prefix("www.")
        .or_else(|| host.strip_prefix("m."))
        .unwrap_or(host)
        .to_ascii_lowercase();
    if !APPLE_HOSTS.contains(&bare_host.as_str()) {
        return None;
    }

    let segments: Vec<&str> = url
        .path_segments()?
        .filter(|seg| !seg.is_empty())
        .collect();
    let kind_idx = segments
        .iter()
        .position(|seg| matches!(*seg, "album" | "song" | "artist" | "playlist"))?;

    let tail = segments.get(kind_idx + 1..)?;
    let id = tail.last()?.trim();
    if id.is_empty() {
        return None;
    }

    match segments[kind_idx] {
        "album" => {
            let track_id = url
                .query_pairs()
                .find(|(k, v)| k == "i" && !v.trim().is_empty())
                .map(|(_, v)| v.trim().to_owned());
            match track_id {
                Some(song_id) => Some(("track", song_id)),
                None => Some(("album", id.to_owned())),
            }
        }
        "song" => Some(("track", id.to_owned())),
        "artist" => Some(("artist", id.to_owned())),
        "playlist" => Some(("playlist", id.to_owned())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_album_song_artist_and_playlist_links() {
        assert_eq!(
            parse("https://music.apple.com/us/album/substance/617154241"),
            Some(("album", "617154241".to_owned()))
        );
        assert_eq!(
            parse("https://music.apple.com/us/album/substance/617154241?i=1440857781"),
            Some(("track", "1440857781".to_owned()))
        );
        assert_eq!(
            parse("https://music.apple.com/us/song/blue-monday/1440857781"),
            Some(("track", "1440857781".to_owned()))
        );
        assert_eq!(
            parse("https://geo.music.apple.com/gb/artist/daft-punk/5468295"),
            Some(("artist", "5468295".to_owned()))
        );
        assert_eq!(
            parse("https://music.apple.com/us/playlist/today-hits/pl.f4d106fed2bd41149aaacabb233eb5eb"),
            Some(("playlist", "pl.f4d106fed2bd41149aaacabb233eb5eb".to_owned()))
        );
    }

    #[test]
    fn ignores_empty_track_query_and_handles_missing_slug_or_non_apple_hosts() {
        assert_eq!(
            parse("  https://itunes.apple.com/album/617154241?i=  "),
            Some(("album", "617154241".to_owned()))
        );
        assert_eq!(
            parse("music://music.apple.com/us/album/123"),
            Some(("album", "123".to_owned()))
        );
        assert_eq!(parse("https://music.apple.com/us/album/"), None);
        assert_eq!(parse("https://open.spotify.com/track/123"), None);
        assert_eq!(parse("not a url"), None);
    }
}
