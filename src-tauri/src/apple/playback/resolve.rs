//! Apple Music stream resolution: `webPlayback` lookup, quality selection,
//! and HLS media playlist parsing (`#EXT-X-MAP` + Widevine/CENC `#EXT-X-KEY`).

use std::time::Duration;

use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
    Engine as _,
};
use serde_json::Value;
use url::Url;

use crate::errors::AppError;

/// Apple Music web playback endpoint.
const WEB_PLAYBACK_URL: &str =
    "https://play.music.apple.com/WebObjects/MZPlay.woa/wa/webPlayback";

/// Safari macOS User-Agent expected by Apple's web playback and CDN endpoints.
pub(crate) const SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15";

/// Widevine-compatible CENC asset flavors ordered by bitrate.
pub(crate) const FLAVOR_AAC_256: &str = "28:ctrp256";
pub(crate) const FLAVOR_HEAAC_64: &str = "32:ctrp64";

const CENC_METHOD: &str = "ISO-23001-7";
const MAX_PLAYLIST_BYTES: usize = 1024 * 1024;

/// Quality preference mapped from Musique's `audio_quality` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppleQuality {
    /// 256 kbps AAC-LC (`28:ctrp256`), used for High / Very High / Lossless settings.
    High256,
    /// 64 kbps HE-AAC (`32:ctrp64`), used for Normal / Low (`"96"`) settings.
    Low64,
}

impl AppleQuality {
    /// Maps Musique's `audio_quality` setting (`"96"`, `"160"`, `"320"`) to an Apple flavor tier.
    pub fn from_setting(setting: Option<&str>) -> Self {
        match setting.map(str::trim) {
            Some("96") => Self::Low64,
            _ => Self::High256,
        }
    }

    fn preference_order(self) -> [&'static str; 2] {
        match self {
            Self::High256 => [FLAVOR_AAC_256, FLAVOR_HEAAC_64],
            Self::Low64 => [FLAVOR_HEAAC_64, FLAVOR_AAC_256],
        }
    }
}

/// Which audio flavor was selected for a resolved track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlavorKind {
    Aac256,
    HeAac64,
    Plain,
}

impl FlavorKind {
    pub fn kbps(self) -> Option<u32> {
        match self {
            Self::Aac256 => Some(256),
            Self::HeAac64 => Some(64),
            Self::Plain => None,
        }
    }
}

/// Widevine key metadata needed to license a CENC stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    /// Full `#EXT-X-KEY` URI passed verbatim to `acquireWebPlaybackLicense`.
    pub uri:     String,
    /// 16-byte key id extracted from `uri`.
    pub id:      Vec<u8>,
    /// Catalog `songId` returned by `webPlayback` (the license endpoint rejects `i.*` library ids).
    pub song_id: String,
}

/// Resolved playback target for an Apple Music track or library upload.
#[derive(Debug, Clone)]
pub struct ResolvedTrack {
    pub asset_url:  String,
    pub key:        Option<KeyInfo>,
    pub duration:   Option<Duration>,
    pub flavor:     FlavorKind,
    pub is_library: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SelectedAsset {
    Ctrp {
        flavor:      &'static str,
        url:         String,
        duration_ms: Option<u64>,
    },
    Plain {
        url:         String,
        duration_ms: Option<u64>,
    },
}

/// Whether `id` is an Apple Music user-library identifier (`i.*`, `l.*`, `r.*`, `p.*`)
/// rather than a numeric catalog Adam ID.
pub fn is_library_id(id: &str) -> bool {
    let mut chars = id.chars();
    matches!(chars.next(), Some('i' | 'l' | 'r' | 'p')) && chars.next() == Some('.')
}

/// Attaches the browser-identity and Apple auth headers required by `webPlayback`
/// and `acquireWebPlaybackLicense`.
pub(crate) fn apply_apple_headers(
    req: reqwest::RequestBuilder,
    bearer: &str,
    user_token: &str,
) -> reqwest::RequestBuilder {
    req.header(reqwest::header::USER_AGENT, SAFARI_USER_AGENT)
        .header(reqwest::header::ORIGIN, "https://music.apple.com")
        .header(reqwest::header::REFERER, "https://music.apple.com/")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {bearer}"))
        .header("x-apple-music-user-token", user_token)
}

/// Sends `request` and retries once if opening the connection itself failed
/// (Apple play edges occasionally drop or time out on TCP/TLS connect).
pub(crate) async fn send_reconnecting(
    request: reqwest::RequestBuilder,
    context: &str,
) -> Result<reqwest::Response, AppError> {
    let retry = request.try_clone();
    match request.send().await {
        Ok(resp) => Ok(resp),
        Err(err) if err.is_connect() => {
            let Some(retry) = retry else {
                return Err(AppError::Network(format!("{context}: connect failed")));
            };
            log::info!("apple: {context} connect failed, retrying once");
            retry
                .send()
                .await
                .map_err(|e| AppError::Network(format!("{context}: {e}")))
        }
        Err(err) => Err(AppError::Network(format!("{context}: {err}"))),
    }
}

/// Resolves an Apple Music catalog or library track id into a downloadable asset URL
/// and (for DRM-protected streams) its Widevine key metadata.
pub async fn resolve(
    http: &reqwest::Client,
    track_id: &str,
    bearer: &str,
    user_token: &str,
    quality: AppleQuality,
) -> Result<ResolvedTrack, AppError> {
    let is_library = is_library_id(track_id);
    let body = if is_library {
        serde_json::json!({ "universalLibraryId": track_id })
    } else {
        serde_json::json!({ "salableAdamId": track_id })
    };

    let req = apply_apple_headers(http.post(WEB_PLAYBACK_URL), bearer, user_token).json(&body);
    let resp = send_reconnecting(req, "webPlayback").await?;
    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| {
        AppError::Playback(format!("cannot parse webPlayback response (http {status}): {e}"))
    })?;

    if !status.is_success() {
        return Err(AppError::Playback(format!(
            "apple webPlayback refused track: http {status}"
        )));
    }
    if let Some(failure) = json
        .get("failureType")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        return Err(AppError::Playback(format!(
            "apple webPlayback refused track: {failure}"
        )));
    }

    let song = json
        .get("songList")
        .and_then(Value::as_array)
        .and_then(|list| list.first())
        .ok_or_else(|| AppError::NotFound("apple webPlayback returned an empty songList".into()))?;

    let selected = select_asset(song, quality)?;
    match selected {
        SelectedAsset::Ctrp {
            flavor,
            url: playlist_url,
            duration_ms: asset_duration_ms,
        } => {
            log::info!("apple: resolved track {track_id} using flavor {flavor}");
            let playlist = fetch_playlist(http, &playlist_url).await?;
            let (file_url, key_uri) = parse_playlist(&playlist, &playlist_url)?;
            let key_id = decode_key_id(&key_uri)?;
            if key_id.len() != 16 {
                log::warn!("apple: widevine key id is {} bytes (expected 16)", key_id.len());
            }
            let song_id = extract_song_id(song).unwrap_or_else(|| track_id.to_owned());
            let duration = asset_duration_ms
                .or_else(|| extract_duration_ms(song))
                .map(Duration::from_millis);
            let flavor_kind = if flavor == FLAVOR_HEAAC_64 {
                FlavorKind::HeAac64
            } else {
                FlavorKind::Aac256
            };

            Ok(ResolvedTrack {
                asset_url: file_url,
                key: Some(KeyInfo {
                    uri: key_uri,
                    id: key_id,
                    song_id,
                }),
                duration,
                flavor: flavor_kind,
                is_library,
            })
        }
        SelectedAsset::Plain { url, duration_ms } => {
            log::info!("apple: resolved plain audio upload for track {track_id}");
            let duration = duration_ms
                .or_else(|| extract_duration_ms(song))
                .map(Duration::from_millis);
            Ok(ResolvedTrack {
                asset_url: url,
                key: None,
                duration,
                flavor: FlavorKind::Plain,
                is_library,
            })
        }
    }
}

async fn fetch_playlist(http: &reqwest::Client, url: &str) -> Result<String, AppError> {
    let req = http
        .get(url)
        .header(reqwest::header::USER_AGENT, SAFARI_USER_AGENT);
    let resp = send_reconnecting(req, "media playlist").await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::Playback(format!(
            "apple refused media playlist: http {status}"
        )));
    }
    if resp
        .content_length()
        .is_some_and(|len| len > MAX_PLAYLIST_BYTES as u64)
    {
        return Err(AppError::Playback("media playlist exceeds size limit".into()));
    }
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Network(format!("cannot read media playlist: {e}")))?;
    if text.len() > MAX_PLAYLIST_BYTES {
        return Err(AppError::Playback("media playlist exceeds size limit".into()));
    }
    Ok(text)
}

/// Picks the best playable Widevine CENC flavor (`28:ctrp256` or `32:ctrp64`)
/// according to `quality`, falling back to the other CENC flavor or a plain upload.
pub(crate) fn select_asset(song: &Value, quality: AppleQuality) -> Result<SelectedAsset, AppError> {
    if song
        .get("hls-playlist-url")
        .and_then(Value::as_str)
        .is_some_and(|u| !u.is_empty())
    {
        return Err(AppError::Playback(
            "track uses unsupported FairPlay HLS master playlist".into(),
        ));
    }

    let assets = song
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::NotFound("song carries no playback assets".into()))?;

    for flavor in quality.preference_order() {
        if let Some(asset) = assets
            .iter()
            .find(|a| a.get("flavor").and_then(Value::as_str) == Some(flavor))
        {
            if let Some(url) = asset
                .get("URL")
                .and_then(Value::as_str)
                .filter(|u| !u.is_empty())
            {
                let duration_ms = asset_duration_ms(asset);
                return Ok(SelectedAsset::Ctrp {
                    flavor,
                    url: url.to_owned(),
                    duration_ms,
                });
            }
        }
    }

    // Plain user-uploaded audio without FairPlay or Widevine DRM.
    for asset in assets {
        let flavor = asset.get("flavor").and_then(Value::as_str);
        if !matches!(flavor, None | Some("item:audio")) {
            continue;
        }
        let Some(url) = asset
            .get("URL")
            .and_then(Value::as_str)
            .filter(|u| !u.is_empty())
        else {
            continue;
        };
        let path = url.split_once('?').map_or(url, |(p, _)| p);
        if path.ends_with(".m3u8") {
            continue;
        }
        return Ok(SelectedAsset::Plain {
            url:         url.to_owned(),
            duration_ms: asset_duration_ms(asset),
        });
    }

    Err(AppError::Playback(
        "track has no Widevine CENC or plain audio asset".into(),
    ))
}

fn asset_duration_ms(asset: &Value) -> Option<u64> {
    asset
        .pointer("/metadata/durationInMillis")
        .or_else(|| asset.pointer("/metadata/duration"))
        .and_then(Value::as_u64)
        .filter(|&ms| ms > 0)
}

pub(crate) fn extract_duration_ms(song: &Value) -> Option<u64> {
    [
        song.pointer("/assets/0/metadata/durationInMillis"),
        song.pointer("/assets/0/metadata/duration"),
        song.pointer("/metadata/durationInMillis"),
        song.get("durationInMillis"),
    ]
    .into_iter()
    .flatten()
    .find_map(Value::as_u64)
    .filter(|&ms| ms > 0)
}

pub(crate) fn extract_song_id(song: &Value) -> Option<String> {
    match song.get("songId")? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Extracts `(resolved_fmp4_url, key_uri)` from a media playlist.
///
/// Prefers an `#EXT-X-KEY` matching `widevine::KEY_FORMAT` and otherwise accepts
/// `METHOD=ISO-23001-7`, skipping FairPlay `skd://` entries.
pub(crate) fn parse_playlist(playlist: &str, base_url: &str) -> Result<(String, String), AppError> {
    let mut widevine_key: Option<String> = None;
    let mut cenc_key: Option<String> = None;
    let mut map_uri: Option<String> = None;

    for raw_line in playlist.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix("#EXT-X-KEY:") {
            let attrs = parse_attributes(rest);
            let attr = |wanted: &str| -> Option<&str> {
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(wanted))
                    .map(|(_, v)| v.as_str())
            };
            let Some(uri) = attr("URI").filter(|u| !u.is_empty() && !u.starts_with("skd://")) else {
                continue;
            };
            if attr("KEYFORMAT") == Some(crate::widevine::KEY_FORMAT) {
                widevine_key = Some(uri.to_owned());
            } else if attr("METHOD") == Some(CENC_METHOD) {
                cenc_key = Some(uri.to_owned());
            }
        } else if let Some(rest) = line.strip_prefix("#EXT-X-MAP:") {
            let attrs = parse_attributes(rest);
            if let Some((_, uri)) = attrs
                .into_iter()
                .find(|(k, v)| k.eq_ignore_ascii_case("URI") && !v.is_empty())
            {
                map_uri = Some(uri);
            }
        }
    }

    let key_uri = widevine_key
        .or(cenc_key)
        .ok_or_else(|| AppError::Playback("media playlist contains no Widevine/CENC key".into()))?;
    let map = map_uri
        .ok_or_else(|| AppError::Playback("media playlist contains no #EXT-X-MAP URI".into()))?;
    let resolved_map = resolve_uri(&map, base_url)?;
    Ok((resolved_map, key_uri))
}

/// Parses comma-separated `KEY=VALUE` HLS tag attributes while preserving commas inside quotes.
pub(crate) fn parse_attributes(input: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    let mut rest = input.trim();

    while !rest.is_empty() {
        let Some((raw_name, after_eq)) = rest.split_once('=') else {
            break;
        };
        let name = raw_name.trim().to_owned();
        let after_eq = after_eq.trim_start();
        let (value, tail) = if let Some(quoted) = after_eq.strip_prefix('"') {
            match quoted.split_once('"') {
                Some((v, rem)) => (v.to_owned(), rem),
                None => (quoted.to_owned(), ""),
            }
        } else {
            match after_eq.split_once(',') {
                Some((v, rem)) => (v.trim().to_owned(), rem),
                None => (after_eq.trim().to_owned(), ""),
            }
        };
        if !name.is_empty() {
            attrs.push((name, value));
        }
        rest = tail.trim_start_matches(|c: char| c == ',' || c.is_ascii_whitespace());
    }

    attrs
}

pub(crate) fn resolve_uri(uri: &str, base_url: &str) -> Result<String, AppError> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return Ok(uri.to_owned());
    }
    if let Ok(base) = Url::parse(base_url) {
        if let Ok(joined) = base.join(uri) {
            return Ok(joined.into());
        }
    }
    match base_url.rsplit_once('/') {
        Some((dir, _)) => Ok(format!("{dir}/{uri}")),
        None => Ok(uri.to_owned()),
    }
}

/// Decodes the base64 key id trailing the last comma in an HLS `data:` key URI.
pub(crate) fn decode_key_id(key_uri: &str) -> Result<Vec<u8>, AppError> {
    let encoded = key_uri
        .rsplit_once(',')
        .map(|(_, b64)| b64.trim())
        .filter(|b64| !b64.is_empty())
        .ok_or_else(|| AppError::Playback("Widevine key URI carries no base64 key id".into()))?;
    let bytes = STANDARD
        .decode(encoded)
        .or_else(|_| STANDARD_NO_PAD.decode(encoded))
        .map_err(|e| AppError::Playback(format!("invalid base64 key id: {e}")))?;
    if bytes.is_empty() {
        return Err(AppError::Playback("Widevine key id is empty".into()));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENC_PLAYLIST: &str = r#"#EXTM3U
#EXT-X-VERSION:6
#EXT-X-TARGETDURATION:6
#EXT-X-MAP:URI="audio.mp4"
#EXT-X-KEY:METHOD=ISO-23001-7,URI="data:;base64,AAAAABozqhkAHa+aTGXMNg=="
#EXTINF:6.00000,
#EXT-X-BYTERANGE:12345@0
audio.mp4
#EXT-X-ENDLIST
"#;

    const FAIRPLAY_PLAYLIST: &str = r#"#EXTM3U
#EXT-X-MAP:URI="audio.mp4"
#EXT-X-KEY:METHOD=SAMPLE-AES,URI="skd://itunes.apple.com/P000000000/s1/e1",KEYFORMAT="com.apple.streamingkeydelivery",KEYFORMATVERSIONS="1"
#EXT-X-ENDLIST
"#;

    #[test]
    fn parses_quoted_attributes_with_commas_and_spaces() {
        let parsed = parse_attributes(
            r#"METHOD=SAMPLE-AES-CTR, URI="data:text/plain;base64,AA==", KEYFORMAT="urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed", IV=0x01"#,
        );
        assert_eq!(
            parsed,
            vec![
                ("METHOD".into(), "SAMPLE-AES-CTR".into()),
                ("URI".into(), "data:text/plain;base64,AA==".into()),
                (
                    "KEYFORMAT".into(),
                    "urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed".into()
                ),
                ("IV".into(), "0x01".into()),
            ]
        );
    }

    #[test]
    fn parses_cenc_and_widevine_playlists_and_rejects_fairplay() {
        let (file, key) = parse_playlist(
            CENC_PLAYLIST,
            "https://aod.itunes.apple.com/x/y/playlist.m3u8",
        )
        .unwrap();
        assert_eq!(file, "https://aod.itunes.apple.com/x/y/audio.mp4");
        assert_eq!(key, "data:;base64,AAAAABozqhkAHa+aTGXMNg==");

        let kid = decode_key_id(&key).unwrap();
        assert_eq!(kid.len(), 16);

        let mixed = format!(
            "{}\n#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"skd://x\",KEYFORMAT=\"com.apple.streamingkeydelivery\"\n",
            CENC_PLAYLIST.replace(
                "METHOD=ISO-23001-7,",
                r#"METHOD=SAMPLE-AES-CTR,KEYFORMAT="urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed","#,
            )
        );
        let (_, wv_key) = parse_playlist(&mixed, "https://cdn.example/sub/p.m3u8").unwrap();
        assert_eq!(wv_key, "data:;base64,AAAAABozqhkAHa+aTGXMNg==");

        assert!(parse_playlist(FAIRPLAY_PLAYLIST, "https://cdn.example/p.m3u8").is_err());
    }

    #[test]
    fn resolves_relative_and_absolute_map_uris() {
        assert_eq!(
            resolve_uri(
                "https://other.cdn.example/audio.mp4",
                "https://cdn.example/x/p.m3u8"
            )
            .unwrap(),
            "https://other.cdn.example/audio.mp4"
        );
        assert_eq!(
            resolve_uri("media/seg.mp4", "https://cdn.example/x/y/p.m3u8?tok=1").unwrap(),
            "https://cdn.example/x/y/media/seg.mp4"
        );
    }

    #[test]
    fn selects_flavor_by_quality_with_fallback_and_plain_uploads() {
        let song = serde_json::json!({
            "songId": 1440857781u64,
            "assets": [
                { "flavor": "32:cbcp64", "URL": "https://x/cbcp.m3u8" },
                { "flavor": "32:ctrp64", "URL": "https://x/64.m3u8", "metadata": { "durationInMillis": 180_000 } },
                { "flavor": "28:ctrp256", "URL": "https://x/256.m3u8", "metadata": { "durationInMillis": 180_000 } },
            ]
        });
        assert_eq!(
            select_asset(&song, AppleQuality::High256).unwrap(),
            SelectedAsset::Ctrp {
                flavor:      "28:ctrp256",
                url:         "https://x/256.m3u8".into(),
                duration_ms: Some(180_000),
            }
        );
        assert_eq!(
            select_asset(&song, AppleQuality::Low64).unwrap(),
            SelectedAsset::Ctrp {
                flavor:      "32:ctrp64",
                url:         "https://x/64.m3u8".into(),
                duration_ms: Some(180_000),
            }
        );

        let fallback_only = serde_json::json!({
            "assets": [{ "flavor": "32:ctrp64", "URL": "https://x/64.m3u8" }]
        });
        assert_eq!(
            select_asset(&fallback_only, AppleQuality::High256).unwrap(),
            SelectedAsset::Ctrp {
                flavor:      "32:ctrp64",
                url:         "https://x/64.m3u8".into(),
                duration_ms: None,
            }
        );

        let upload = serde_json::json!({
            "assets": [{ "flavor": "item:audio", "URL": "https://x/upload.m4a?sig=1" }]
        });
        assert_eq!(
            select_asset(&upload, AppleQuality::High256).unwrap(),
            SelectedAsset::Plain {
                url:         "https://x/upload.m4a?sig=1".into(),
                duration_ms: None,
            }
        );

        let fairplay_only = serde_json::json!({
            "assets": [
                { "flavor": "32:cbcp64", "URL": "https://x/cbcp.m3u8" },
                { "flavor": "28:ibhp256", "URL": "https://x/ibhp.m3u8" }
            ]
        });
        assert!(select_asset(&fairplay_only, AppleQuality::High256).is_err());
    }

    #[test]
    fn maps_quality_settings_and_library_ids() {
        assert_eq!(AppleQuality::from_setting(Some("96")), AppleQuality::Low64);
        assert_eq!(AppleQuality::from_setting(Some("160")), AppleQuality::High256);
        assert_eq!(AppleQuality::from_setting(Some("320")), AppleQuality::High256);
        assert_eq!(AppleQuality::from_setting(None), AppleQuality::High256);

        assert!(is_library_id("i.abc123"));
        assert!(is_library_id("l.xyz"));
        assert!(!is_library_id("1440857781"));
        assert!(!is_library_id("ixyz"));
    }
}
