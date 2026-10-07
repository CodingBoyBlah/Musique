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

use futures_util::{stream, StreamExt};

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

/// Compute inclusive byte ranges of size `chunk_size` for `total` bytes.
fn compute_ranges(total: u64, chunk_size: u64) -> Vec<(u64, u64)> {
    if total == 0 || chunk_size == 0 {
        return Vec::new();
    }
    (0..)
        .map(|i| i * chunk_size)
        .take_while(|&start| start < total)
        .map(|start| (start, (start + chunk_size - 1).min(total - 1)))
        .collect()
}

/// Download a complete audio stream through bounded concurrent range requests.
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

    let ranges = compute_ranges(total, CHUNK_SIZE);

    // Fetch ranges concurrently using `buffered(CONCURRENCY)`.
    // This bounds active in-flight requests and retained buffers, yields
    // chunks in strictly ordered sequence, and immediately drops all outstanding
    // HTTP requests if the returned future is dropped / cancelled or on error.
    let mut chunk_stream = stream::iter(ranges.iter().copied().map(|(start, end)| {
        let url = &stream.format.url;
        let ua = stream.user_agent;
        let id = &stream.video_id;
        async move {
            let chunk = fetch_range(url, ua, id, start, end).await?;
            Ok::<(u64, bytes::Bytes), AppError>((start, chunk))
        }
    }))
    .buffered(CONCURRENCY);

    let mut buf = Vec::with_capacity(total as usize);

    while let Some(res) = chunk_stream.next().await {
        let (start, chunk) = res?;
        if chunk.is_empty() {
            return Err(AppError::Playback(format!(
                "{}: empty chunk at byte {start} of {total}",
                stream.video_id
            )));
        }
        buf.extend_from_slice(&chunk);
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

/// Fetch a single inclusive range without copying the response into another Vec.
async fn fetch_range(
    url: &str,
    user_agent: &'static str,
    video_id: &str,
    start: u64,
    end: u64,
) -> Result<bytes::Bytes, AppError> {
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
        .map_err(|e| AppError::Network(format!("media body {start}-{end}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::Notify, task::{JoinHandle, JoinSet}, time::{sleep, timeout, Duration}};

    #[derive(Clone, Copy)]
    enum Scenario { OutOfOrder, Gated, RejectFirst }

    struct Server {
        url: String,
        requests: Arc<AtomicUsize>,
        events: Arc<Mutex<Vec<(u64, bool)>>>,
        gate: Arc<Notify>,
        task: JoinHandle<()>,
    }

    impl Drop for Server {
        fn drop(&mut self) { self.task.abort(); }
    }

    async fn server(scenario: Scenario) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/audio", listener.local_addr().unwrap());
        let requests = Arc::new(AtomicUsize::new(0));
        let events = Arc::new(Mutex::new(Vec::new()));
        let gate = Arc::new(Notify::new());
        let (count, log, ready) = (requests.clone(), events.clone(), gate.clone());
        let task = tokio::spawn(async move {
            let mut handlers = JoinSet::new();
            loop {
                tokio::select! {
                    socket = listener.accept() => {
                        let (mut socket, _) = socket.unwrap();
                        let (count, log, ready) = (count.clone(), log.clone(), ready.clone());
                        handlers.spawn(async move {
                            let mut request = Vec::new();
                            let mut block = [0; 1024];
                            while !request.ends_with(b"\r\n\r\n") {
                                let n = socket.read(&mut block).await.unwrap();
                                if n == 0 { return; }
                                request.extend_from_slice(&block[..n]);
                            }
                            let request = String::from_utf8(request).unwrap().to_ascii_lowercase();
                            let range = request.lines().find_map(|line| line.strip_prefix("range: bytes=")).unwrap();
                            let (start, end) = range.split_once('-').unwrap();
                            let (start, end) = (start.parse::<u64>().unwrap(), end.parse::<u64>().unwrap());
                            count.fetch_add(1, Ordering::SeqCst);
                            log.lock().unwrap().push((start, false));
                            match scenario {
                                Scenario::Gated => ready.notified().await,
                                Scenario::OutOfOrder => sleep(Duration::from_millis(if start == CHUNK_SIZE * 3 { 200 } else if start == 0 { 40 } else { 5 })).await,
                                Scenario::RejectFirst if start != 0 => sleep(Duration::from_millis(40)).await,
                                Scenario::RejectFirst => {},
                            }
                            let rejected = matches!(scenario, Scenario::RejectFirst) && start == 0;
                            let body = if rejected { Vec::new() } else { vec![(start / CHUNK_SIZE) as u8; (end - start + 1) as usize] };
                            let status = if rejected { "500 Internal Server Error" } else { "206 Partial Content" };
                            let header = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                            if socket.write_all(header.as_bytes()).await.is_ok() {
                                let _ = socket.write_all(&body).await;
                            }
                            log.lock().unwrap().push((start, true));
                        });
                    }
                    _ = handlers.join_next(), if !handlers.is_empty() => {},
                }
            }
        });
        Server { url, requests, events, gate, task }
    }

    fn extracted(url: String) -> ExtractedStream {
        ExtractedStream {
            video_id: "local-test".into(), title: None, author: None, duration_ms: None,
            format: super::super::format::AudioFormat {
                itag: 140, url, mime_type: "audio/mp4".into(), bitrate: 130_000,
                content_length: Some(CHUNK_SIZE * 6 + 31), loudness_db: None,
            },
            user_agent: "range-test", client_name: "test".into(), expires_in: 300,
        }
    }

    #[tokio::test]
    async fn download_preserves_bytes_and_keeps_a_rolling_bounded_window() {
        let server = server(Scenario::OutOfOrder).await;
        let data = timeout(Duration::from_secs(5), download(&extracted(server.url.clone()))).await.unwrap().unwrap();
        assert_eq!(data.len(), (CHUNK_SIZE * 6 + 31) as usize);
        for (i, chunk) in data.chunks(CHUNK_SIZE as usize).enumerate() {
            assert!(chunk.iter().all(|&byte| byte == i as u8));
        }
        let events = server.events.lock().unwrap();
        let slow_done = events.iter().position(|&event| event == (CHUNK_SIZE * 3, true)).unwrap();
        let replacement_started = events.iter().position(|&event| event == (CHUNK_SIZE * 4, false)).unwrap();
        assert!(replacement_started < slow_done, "must refill before the slow window tail finishes");
    }

    #[tokio::test]
    async fn cancelling_download_does_not_leave_detached_range_jobs() {
        let server = server(Scenario::Gated).await;
        let track = extracted(server.url.clone());
        let mut pending = Box::pin(download(&track));
        timeout(Duration::from_secs(5), async {
            loop {
                tokio::select! {
                    result = &mut pending => panic!("gated download finished: {result:?}"),
                    _ = sleep(Duration::from_millis(5)) => {
                        if server.requests.load(Ordering::SeqCst) == CONCURRENCY { break; }
                    }
                }
            }
        }).await.unwrap();
        drop(pending);
        server.gate.notify_waiters();
        sleep(Duration::from_millis(100)).await;
        assert_eq!(server.requests.load(Ordering::SeqCst), CONCURRENCY);
    }

    #[tokio::test]
    async fn download_failure_cancels_unconsumed_range_jobs() {
        let server = server(Scenario::RejectFirst).await;
        let error = download(&extracted(server.url.clone())).await.unwrap_err();
        assert!(error.to_string().contains("media fetch 0-1048575 returned 500"));
        sleep(Duration::from_millis(100)).await;
        assert!(server.requests.load(Ordering::SeqCst) <= CONCURRENCY);
    }

    #[test]
    fn compute_ranges_handles_exact_multiple() {
        let ranges = compute_ranges(2 * 1024 * 1024, 1024 * 1024);
        assert_eq!(ranges, vec![(0, 1024 * 1024 - 1), (1024 * 1024, 2 * 1024 * 1024 - 1)]);
    }

    #[test]
    fn compute_ranges_handles_remainder() {
        let ranges = compute_ranges(2_500_000, 1024 * 1024);
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0], (0, 1024 * 1024 - 1));
        assert_eq!(ranges[1], (1024 * 1024, 2 * 1024 * 1024 - 1));
        assert_eq!(ranges[2], (2 * 1024 * 1024, 2_499_999));
    }

    #[test]
    fn compute_ranges_handles_empty() {
        assert!(compute_ranges(0, 1024 * 1024).is_empty());
    }
}
