// Audio format (itag) selection.
//
// Codec choice is constrained by what we can actually decode. `Cargo.lock`
// already pulls `symphonia-codec-aac` + `symphonia-format-isomp4` transitively
// through rodio, so AAC-in-MP4 decodes with zero new dependencies. Opus (itags
// 249/250/251) is the better codec at these bitrates and YouTube always offers
// it, but `symphonia-codec-opus` is an unimplemented stub and every real Opus
// decoder is a C binding - so Opus stays out until someone decides that
// dependency is worth it. Tracked in task_plan.md.

use serde::{Deserialize, Serialize};

/// One entry from `streamingData.adaptiveFormats`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawFormat {
    pub itag:            u32,
    /// Absent when YouTube returns a `signatureCipher` blob instead, which only
    /// happens on the clients we deliberately do not use.
    pub url:             Option<String>,
    pub mime_type:       Option<String>,
    pub bitrate:         Option<u64>,
    /// Present on video formats, absent on audio-only. The cheapest reliable
    /// way to tell the two apart.
    pub width:           Option<u32>,
    #[serde(default, deserialize_with = "de_opt_u64_str")]
    pub content_length:  Option<u64>,
    pub audio_sample_rate: Option<serde_json::Value>,
    pub loudness_db:     Option<f64>,
    pub audio_track:     Option<AudioTrack>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub display_name:  Option<String>,
    /// Set on YouTube's machine-generated dubs. A dubbed track is the wrong
    /// audio for a music match, so these are filtered out entirely.
    pub is_auto_dubbed: Option<bool>,
}

/// `contentLength` arrives as a JSON *string* ("3449447"), not a number.
fn de_opt_u64_str<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(match v {
        Some(serde_json::Value::String(s)) => s.parse().ok(),
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        _ => None,
    })
}

/// A format we have committed to playing.
#[derive(Debug, Clone, Serialize)]
pub struct AudioFormat {
    pub itag:           u32,
    pub url:            String,
    pub mime_type:      String,
    pub bitrate:        u64,
    pub content_length: Option<u64>,
    /// YouTube's per-track normalisation figure, in dB relative to its target.
    /// Carried through so playback can match Spotify's loudness behaviour
    /// instead of jumping in volume between backends.
    pub loudness_db:    Option<f64>,
}

impl RawFormat {
    fn is_audio(&self) -> bool {
        self.width.is_none()
            && self.mime_type.as_deref().is_some_and(|m| m.starts_with("audio/"))
    }

    fn is_original_language(&self) -> bool {
        !self.audio_track.as_ref().and_then(|t| t.is_auto_dubbed).unwrap_or(false)
    }
}

/// Preference order, best first. 140 is ~130 kbps AAC-LC and is present on
/// essentially every track; 139 is ~50 kbps and exists as a genuine last resort
/// rather than a quality tier we would choose.
const ITAG_PREFERENCE: &[u32] = &[140, 139];

/// Pick the best decodable audio format, or `None` if the response only offers
/// codecs we cannot play.
///
/// Returning `None` rather than falling back to *some* format is deliberate:
/// handing an Opus URL to an AAC decoder produces a confusing runtime failure
/// far from its cause, and the caller can fall through to the next client.
pub fn select(formats: &[RawFormat]) -> Option<AudioFormat> {
    ITAG_PREFERENCE.iter().find_map(|&want| {
        formats
            .iter()
            .find(|f| f.itag == want && f.is_audio() && f.is_original_language() && f.url.is_some())
            .map(|f| AudioFormat {
                itag:           f.itag,
                url:            f.url.clone().expect("filtered on is_some"),
                mime_type:      f.mime_type.clone().unwrap_or_else(|| "audio/mp4".into()),
                bitrate:        f.bitrate.unwrap_or(0),
                content_length: f.content_length,
                loudness_db:    f.loudness_db,
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(itag: u32, mime: &str, url: Option<&str>, width: Option<u32>) -> RawFormat {
        RawFormat {
            itag,
            url: url.map(String::from),
            mime_type: Some(mime.into()),
            bitrate: Some(1000),
            width,
            content_length: Some(123),
            audio_sample_rate: None,
            loudness_db: None,
            audio_track: None,
        }
    }

    #[test]
    fn prefers_140_over_139() {
        let f = vec![
            fmt(139, "audio/mp4; codecs=\"mp4a.40.5\"", Some("u139"), None),
            fmt(140, "audio/mp4; codecs=\"mp4a.40.2\"", Some("u140"), None),
        ];
        assert_eq!(select(&f).unwrap().itag, 140);
    }

    #[test]
    fn falls_back_to_139_when_140_absent() {
        let f = vec![fmt(139, "audio/mp4", Some("u139"), None)];
        assert_eq!(select(&f).unwrap().itag, 139);
    }

    /// Opus-only responses must fail loudly, not silently mis-decode.
    #[test]
    fn rejects_opus_only() {
        let f = vec![
            fmt(251, "audio/webm; codecs=\"opus\"", Some("u251"), None),
            fmt(250, "audio/webm; codecs=\"opus\"", Some("u250"), None),
        ];
        assert!(select(&f).is_none());
    }

    #[test]
    fn rejects_video_and_ciphered_formats() {
        let mut video = fmt(140, "video/mp4", Some("v"), Some(1920));
        video.mime_type = Some("video/mp4".into());
        let ciphered = fmt(140, "audio/mp4", None, None);
        assert!(select(&[video, ciphered]).is_none());
    }

    #[test]
    fn rejects_auto_dubbed_audio() {
        let mut dubbed = fmt(140, "audio/mp4", Some("u"), None);
        dubbed.audio_track = Some(AudioTrack {
            display_name:   Some("Spanish (dubbed)".into()),
            is_auto_dubbed: Some(true),
        });
        assert!(select(&[dubbed]).is_none());
    }

    #[test]
    fn parses_string_content_length() {
        let raw: RawFormat = serde_json::from_str(
            r#"{"itag":140,"url":"u","mimeType":"audio/mp4","contentLength":"3449447"}"#,
        )
        .unwrap();
        assert_eq!(raw.content_length, Some(3449447));
    }
}
