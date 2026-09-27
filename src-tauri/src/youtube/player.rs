// Stream extraction: videoId -> playable audio URL.
//
// Walks the client ladder from `client.rs` until one returns a playable
// response with a decodable audio format. Each client is a separate InnerTube
// identity, so a rejection from one says nothing about the next - which is the
// entire point of trying several.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::errors::AppError;

use super::{
    client::{YtClient, EXTRACTION_LADDER},
    format::{self, AudioFormat, RawFormat},
    innertube,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    playability_status: Option<PlayabilityStatus>,
    streaming_data:     Option<StreamingData>,
    video_details:      Option<VideoDetails>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayabilityStatus {
    status: String,
    reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingData {
    #[serde(default)]
    adaptive_formats:    Vec<RawFormat>,
    expires_in_seconds:  Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoDetails {
    video_id:       String,
    title:          Option<String>,
    author:         Option<String>,
    length_seconds: Option<String>,
}

/// Everything needed to fetch and play one track's audio.
#[derive(Debug, Clone, Serialize)]
pub struct ExtractedStream {
    pub video_id:    String,
    pub title:       Option<String>,
    pub author:      Option<String>,
    pub duration_ms: Option<u64>,
    pub format:      AudioFormat,
    /// UA that minted the URL. The CDN does not appear to enforce this today,
    /// but re-using the minting client's UA on the media fetch is free and
    /// keeps the request coherent.
    pub user_agent:  &'static str,
    /// Which ladder rung succeeded - surfaced purely for diagnostics.
    pub client_name: String,
    /// Seconds until the CDN URL stops working. YouTube typically issues ~6h.
    pub expires_in:  u64,
}

/// Default when YouTube omits `expiresInSeconds`. Deliberately short so a stale
/// URL is re-extracted rather than failing mid-playback.
const DEFAULT_EXPIRY_SECS: u64 = 5 * 60;

/// Resolve a YouTube video id to a playable audio stream.
///
/// Errors carry the *last* client's failure reason. Earlier failures are logged
/// rather than accumulated: the ladder failing entirely almost always means one
/// underlying cause (video removed, region locked), not four distinct ones.
pub async fn extract(video_id: &str) -> Result<ExtractedStream, AppError> {
    let mut last_err = None;
    // Only worth refreshing the visitor identity once per extraction: if a
    // freshly minted token is still challenged, the cause is the video or the
    // client profile, and re-scraping for every rung would just be slow.
    let mut refreshed = false;

    for client in EXTRACTION_LADDER {
        let mut attempt = try_client(client, video_id).await;

        // A bot challenge usually means a missing or stale visitor identity
        // rather than genuinely gated content - see visitor.rs. Mint a new one
        // and give this same client another go before writing it off.
        if !refreshed && matches!(&attempt, Err(e) if is_bot_challenge(e)) {
            refreshed = true;
            eprintln!("[youtube] {video_id}: bot-challenged, refreshing visitor id");
            if super::visitor::refresh().await.is_some() {
                attempt = try_client(client, video_id).await;
            }
        }

        match attempt {
            Ok(stream) => {
                if last_err.is_some() || refreshed {
                    eprintln!(
                        "[youtube] {video_id}: recovered on {}",
                        stream.client_name
                    );
                }
                return Ok(stream);
            }
            Err(e) => {
                eprintln!("[youtube] {video_id}: {} failed: {e}", client.name);
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        AppError::Playback(format!("no extraction client available for {video_id}"))
    }))
}

async fn try_client(client: &YtClient, video_id: &str) -> Result<ExtractedStream, AppError> {
    let body = json!({
        "videoId":        video_id,
        // Both must be true or YouTube withholds streams for anything flagged
        // as mature - which covers a large slice of ordinary explicit music.
        "contentCheckOk": true,
        "racyCheckOk":    true,
    });

    let raw = innertube::post(client, "player", body).await?;
    let res: PlayerResponse = serde_json::from_value(raw)
        .map_err(|e| AppError::Playback(format!("player response decode: {e}")))?;

    if let Some(status) = &res.playability_status {
        if !status.status.eq_ignore_ascii_case("OK") {
            let reason = status.reason.as_deref().unwrap_or("no reason given");
            return Err(AppError::Playback(format!(
                "{} unplayable ({}): {reason}",
                video_id, status.status
            )));
        }
    }

    let streaming = res
        .streaming_data
        .ok_or_else(|| AppError::Playback(format!("{video_id}: no streamingData")))?;

    let format = format::select(&streaming.adaptive_formats).ok_or_else(|| {
        AppError::Playback(format!(
            "{video_id}: no decodable audio format among {} offered",
            streaming.adaptive_formats.len()
        ))
    })?;

    let details = res.video_details;
    Ok(ExtractedStream {
        video_id:    details.as_ref().map(|d| d.video_id.clone()).unwrap_or_else(|| video_id.into()),
        title:       details.as_ref().and_then(|d| d.title.clone()),
        author:      details.as_ref().and_then(|d| d.author.clone()),
        duration_ms: details
            .as_ref()
            .and_then(|d| d.length_seconds.as_ref())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|s| s * 1000),
        format,
        user_agent:  client.user_agent,
        client_name: format!("{} {}", client.name, client.version),
        expires_in:  streaming
            .expires_in_seconds
            .as_ref()
            .and_then(json_to_u64)
            .unwrap_or(DEFAULT_EXPIRY_SECS),
    })
}

/// Whether a failure is YouTube's anonymous bot challenge rather than the
/// video genuinely being unavailable.
///
/// Observed in two shapes for the same video at the same moment:
/// `LOGIN_REQUIRED` on most clients, but `UNPLAYABLE` on visionOS 0.1 - both
/// carrying "Sign in to confirm you're not a bot". So the status alone is not
/// a reliable signal and the reason text has to be checked too.
///
/// Matching on "not a bot" rather than the full sentence is deliberate:
/// YouTube's copy uses a typographic apostrophe, and the wording around it has
/// changed before.
fn is_bot_challenge(e: &AppError) -> bool {
    let msg = e.to_string();
    msg.contains("LOGIN_REQUIRED") || msg.contains("not a bot")
}

/// `expiresInSeconds` is a string on some clients and a number on others.
fn json_to_u64(v: &serde_json::Value) -> Option<u64> {
    match v {
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Number(n) => n.as_u64(),
        _ => None,
    }
}
