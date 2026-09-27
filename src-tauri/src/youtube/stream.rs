// Media fetch for extracted YouTube URLs.
//
// The one rule here is: **never issue an unbounded GET**.
//
// YouTube applies its `n`-parameter throttle to requests that ask for the whole
// file. Measured against a 3,449,447-byte track (findings.md has the full
// table): an unbounded GET returned HTTP 200 and delivered 966,656 bytes in 30
// seconds - roughly 0.25x realtime, i.e. unusable. The same URL served with
// `Range` headers returned HTTP 206 and the full file at line speed, including
// ranges past the 1 MiB boundary where the throttle normally bites.
//
// Solving the throttle "properly" means deobfuscating `base.js` to compute the
// `n` transform, which needs a JavaScript engine. Chunking sidesteps it with a
// header. This is what innertubex's `requireBoundedRange` / `rangeChunkSizeBytes`
// / `useRangeChunks` flags are doing, and it is the single reason this feature
// needs no JS runtime.

use crate::{errors::AppError, http};

use super::player::ExtractedStream;

/// Bytes per ranged request. 1 MiB was the chunk size measured in findings.md:
/// large enough that per-request overhead is noise, small enough to stay well
/// inside the throttle's tolerance and to let the first chunk arrive quickly.
const CHUNK_SIZE: u64 = 1024 * 1024;

/// How many ranged requests are allowed in flight at once.
///
/// Four covers a typical 3-4 MB track in a single window, so the whole file
/// costs roughly one round trip instead of one per chunk. Kept low on purpose:
/// the pooled HTTP client allows 8 idle connections per host and this shares
/// that pool with Spotify API traffic, so a larger fan-out would start
/// evicting connections the rest of the app is using.
const CONCURRENCY: usize = 4;

/// Upper bound on a single track download. A 10-minute AAC-LC track at 130 kbps
/// is ~10 MB; this is a guard against a pathological `contentLength`, not a
/// quality limit.
const MAX_TRACK_BYTES: u64 = 96 * 1024 * 1024;

/// Download a complete audio stream into memory via sequential ranged requests.
///
/// Returns the full encoded file, ready to hand to a decoder. Buffering the
/// whole track rather than streaming it is a deliberate simplification: at
/// ~3-4 MB per track the memory cost is trivial next to the WebView, and it
/// makes seeking free and exact instead of requiring range bookkeeping in the
/// decoder. Progressive start-on-first-chunk is a later optimisation and is
/// tracked in task_plan.md.
pub async fn download(stream: &ExtractedStream) -> Result<Vec<u8>, AppError> {
    let total = stream.format.content_length.ok_or_else(|| {
        // Without contentLength we cannot bound the ranges, and an unbounded
        // fetch is throttled into uselessness - so this is a hard failure
        // rather than a fall back to a plain GET.
        AppError::Playback(format!(
            "{}: no contentLength on itag {}, cannot fetch safely",
            stream.video_id, stream.format.itag
        ))
    })?;

    if total == 0 || total > MAX_TRACK_BYTES {
        return Err(AppError::Playback(format!(
            "{}: implausible contentLength {total}",
            stream.video_id
        )));
    }

    // Ranges are fetched in parallel windows rather than one after another.
    // Each request costs a round trip before any bytes arrive, and a 4-minute
    // track is 3-4 chunks, so serially that is 3-4 round trips of pure latency
    // stacked in front of playback. Profiling a cold play put this at ~800ms of
    // the ~1s gap before audio started, which is what made switching tracks
    // feel slow. Windows keep the order (results are collected in sequence)
    // and bound how many requests are in flight at once.
    let ranges: Vec<(u64, u64)> = (0..)
        .map(|i| i * CHUNK_SIZE)
        .take_while(|&start| start < total)
        .map(|start| (start, (start + CHUNK_SIZE - 1).min(total - 1)))
        .collect();

    let mut buf = Vec::with_capacity(total as usize);

    for window in ranges.chunks(CONCURRENCY) {
        let mut inflight = Vec::with_capacity(window.len());
        for &(start, end) in window {
            let url = stream.format.url.clone();
            let ua = stream.user_agent;
            let id = stream.video_id.clone();
            inflight.push(tokio::spawn(async move {
                fetch_range(&url, ua, &id, start, end).await
            }));
        }

        for (handle, &(start, _)) in inflight.into_iter().zip(window) {
            let chunk = handle
                .await
                .map_err(|e| AppError::Playback(format!("media fetch task: {e}")))??;
            if chunk.is_empty() {
                return Err(AppError::Playback(format!(
                    "{}: empty chunk at byte {start} of {total}",
                    stream.video_id
                )));
            }
            buf.extend_from_slice(&chunk);
        }
    }

    if buf.len() as u64 != total {
        return Err(AppError::Playback(format!(
            "{}: short read, got {} of {total} bytes",
            stream.video_id,
            buf.len()
        )));
    }

    Ok(buf)
}

/// Fetch a single inclusive byte range.
///
/// Takes owned/'static parameters rather than `&ExtractedStream` so callers can
/// move it into a spawned task.
async fn fetch_range(
    url: &str,
    user_agent: &'static str,
    video_id: &str,
    start: u64,
    end: u64,
) -> Result<Vec<u8>, AppError> {
    let res = http::client()
        .get(url)
        .header("Range", format!("bytes={start}-{end}"))
        // Match the identity that minted the URL. The CDN does not appear to
        // enforce this today, but a coherent request costs nothing.
        .header("User-Agent", user_agent)
        .send()
        .await
        .map_err(|e| AppError::Network(format!("media fetch {start}-{end}: {e}")))?;

    let status = res.status();
    // 206 is the expected answer. A 200 means the server ignored the Range
    // header, which also means we are being throttled - worth flagging loudly
    // because it is the failure mode this whole module exists to avoid.
    if status.as_u16() == 200 {
        eprintln!(
            "[youtube] {video_id}: server ignored Range {start}-{end} (HTTP 200); \
             expect throttled transfer"
        );
    } else if !status.is_success() {
        return Err(AppError::Network(format!(
            "media fetch {start}-{end} returned {status}"
        )));
    }

    res.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| AppError::Network(format!("media body {start}-{end}: {e}")))
}
