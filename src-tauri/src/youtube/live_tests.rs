// End-to-end checks against the real YouTube Music endpoints.
//
// `#[ignore]` by design: these are network tests. They will fail on a plane,
// and more importantly they will fail the day YouTube retires a client
// identity or reshapes a response - which is exactly what they exist to
// detect. Run them deliberately:
//
//     cargo test --lib youtube::live -- --ignored --nocapture
//
// Treat a failure here as "YouTube changed something", not "the code is
// broken", and start by re-running the probes in findings.md.

use super::*;
use crate::youtube::{matching::TrackQuery, search};

/// A well-known track that should resolve cleanly. Deliberately one with a
/// famously crowded result page (instrumentals, sped-up edits, live cuts) so
/// the matcher's gates are actually exercised rather than trivially satisfied.
fn weeknd_query() -> TrackQuery {
    TrackQuery {
        title:       "Blinding Lights".into(),
        artists:     vec!["The Weeknd".into()],
        album:       Some("After Hours".into()),
        duration_ms: 200_040,
        explicit:    false,
    }
}

#[tokio::test]
#[ignore = "network"]
async fn search_returns_parsable_songs() {
    let results = search::songs("Blinding Lights The Weeknd")
        .await
        .expect("search should succeed");

    assert!(!results.is_empty(), "no results parsed - response shape likely changed");

    // Every row must carry the fields the matcher gates on. If YouTube stops
    // sending durations or video types, matching silently loses its teeth, so
    // assert on them rather than just on the row count.
    for r in results.iter().take(5) {
        println!(
            "{:<14} {:<40} {:?} | {:?} | {:?}",
            r.video_id, r.title, r.artists, r.duration_ms, r.video_type
        );
        assert!(!r.video_id.is_empty());
        assert!(!r.title.is_empty());
        assert!(r.duration_ms.is_some(), "row {:?} lost its duration", r.title);
        assert!(!r.artists.is_empty(), "row {:?} lost its artists", r.title);
    }
}

#[tokio::test]
#[ignore = "network"]
async fn matcher_picks_the_studio_recording() {
    let q = weeknd_query();
    let results = search::songs("Blinding Lights The Weeknd").await.unwrap();
    let m = matching::best_match(&q, &results[..results.len().min(10)])
        .expect("should find an acceptable match");

    println!("matched {} score {:.3} ({})", m.video_id, m.score, m.reason);
    // Duration is the check that would actually catch a wrong pick here.
    let delta = m.duration_ms.unwrap().abs_diff(q.duration_ms);
    assert!(delta <= 3_000, "matched a track {delta}ms away from the Spotify duration");
}

/// The negative case, and the one that matters most: a track that does not
/// exist must produce no match rather than the nearest plausible thing.
#[tokio::test]
#[ignore = "network"]
async fn nonexistent_track_matches_nothing() {
    let q = TrackQuery {
        title:       "Qxzzt Vrblnd Nonexistent Song".into(),
        artists:     vec!["Nonexistent Artist Qxzzt".into()],
        album:       None,
        duration_ms: 213_000,
        explicit:    false,
    };
    let results = search::songs("Qxzzt Vrblnd Nonexistent Song Nonexistent Artist Qxzzt")
        .await
        .unwrap();
    println!("search returned {} rows for a nonsense query", results.len());

    let m = matching::best_match(&q, &results[..results.len().min(10)]);
    assert!(m.is_none(), "matched {:?} for a nonexistent track", m.map(|x| x.video_id));
}

#[tokio::test]
#[ignore = "network"]
async fn extracts_a_playable_stream() {
    let stream = player::extract("dQw4w9WgXcQ")
        .await
        .expect("extraction should succeed");

    println!(
        "client={} itag={} mime={} bytes={:?} expires_in={}s",
        stream.client_name,
        stream.format.itag,
        stream.format.mime_type,
        stream.format.content_length,
        stream.expires_in
    );

    assert!(stream.format.url.starts_with("https://"));
    assert!(
        stream.format.mime_type.starts_with("audio/"),
        "selected a non-audio format: {}",
        stream.format.mime_type
    );
    assert!(stream.format.content_length.unwrap_or(0) > 0);
}

/// Guards the finding this whole module is built on: ranged requests are served
/// at full speed while an unbounded GET is throttled. If this ever starts
/// failing, the chunking strategy needs revisiting before anything else.
#[tokio::test]
#[ignore = "network"]
async fn ranged_fetch_serves_full_speed_past_the_throttle_boundary() {
    let stream = player::extract("dQw4w9WgXcQ").await.unwrap();
    let total = stream.format.content_length.unwrap();

    let started = std::time::Instant::now();
    let bytes = stream::download(&stream).await.expect("download should succeed");
    let elapsed = started.elapsed();

    println!(
        "downloaded {} bytes in {:.2}s ({:.1} MB/s)",
        bytes.len(),
        elapsed.as_secs_f64(),
        bytes.len() as f64 / 1e6 / elapsed.as_secs_f64().max(0.001)
    );

    assert_eq!(bytes.len() as u64, total, "short download");
    // The file is >1 MiB, so an unthrottled transfer proves ranges bypassed the
    // n-param throttle. The throttled path measured ~1 MB per 30s.
    assert!(total > 1024 * 1024, "test asset too small to prove anything");
    assert!(
        elapsed.as_secs() < 25,
        "download took {elapsed:?} - throttle may no longer be range-avoidable"
    );

    // MP4 sanity: an ISO-BMFF file starts with a box size then 'ftyp'.
    assert_eq!(&bytes[4..8], b"ftyp", "not an MP4 container - decoder would reject this");
}

/// Match the real library against YouTube Music and report the outcome.
///
/// Synthetic fixtures can only prove the matcher does what I expected; this
/// runs it over an actual Spotify catalog, where the awkward cases live -
/// features in titles, remixes, non-Latin names, regional releases, tracks with
/// no Art Track at all.
///
/// It deliberately asserts almost nothing about the *rate*. A refusal is a
/// correct outcome, so a low match rate is not a failure, and asserting on one
/// would just encourage loosening the gates. What it does assert is the thing
/// that must never happen: that every ACCEPTED match satisfies the duration
/// gate. The printed report is for human review of the rejections.
///
/// Point it at a database copy:
///   MUSIQUE_DB=/path/to/spotify-client.db \
///     cargo test --lib matches_real_library -- --ignored --nocapture
#[tokio::test]
#[ignore = "network + needs a library database"]
async fn matches_real_library() {
    let Ok(db) = std::env::var("MUSIQUE_DB") else {
        eprintln!("MUSIQUE_DB not set - skipping");
        return;
    };

    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{db}?mode=ro"))
        .await
        .expect("open library database read-only");

    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT t.id FROM tracks t
          WHERE EXISTS (SELECT 1 FROM track_artists ta WHERE ta.track_id = t.id)
          ORDER BY t.id
          LIMIT 25",
    )
    .fetch_all(&pool)
    .await
    .expect("read tracks");

    let (mut matched, mut refused) = (0usize, 0usize);

    for id in &ids {
        let q = match query_for_track(&pool, id).await {
            Ok(q) => q,
            Err(e) => {
                println!("SKIP  {id}: {e}");
                continue;
            }
        };
        let terms = format!("{} {}", q.title, q.artists[0]);
        let results = match search::songs(terms.trim()).await {
            Ok(r) => r,
            Err(e) => {
                println!("ERR   {}: search failed: {e}", q.title);
                continue;
            }
        };

        match matching::best_match(&q, &results[..results.len().min(10)]) {
            Some(m) => {
                matched += 1;
                let delta = m.duration_ms.unwrap_or(0).abs_diff(q.duration_ms);
                // The invariant. A match outside the gate means the gate leaked.
                assert!(
                    delta <= 3_000,
                    "ACCEPTED a match {delta}ms off for {:?} -> {}",
                    q.title,
                    m.video_id
                );
                println!(
                    "OK    {:<44} -> {} ({:.2}, {}ms off)",
                    truncate(&q.title, 44),
                    m.video_id,
                    m.score,
                    delta
                );
            }
            None => {
                refused += 1;
                println!(
                    "NONE  {:<44}    ({} candidates, all rejected)",
                    truncate(&q.title, 44),
                    results.len().min(10)
                );
            }
        }
    }

    println!("\n{matched} matched, {refused} refused, {} total", ids.len());
    assert!(matched + refused > 0, "no tracks were processed at all");
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).collect::<String>() + "\u{2026}"
    }
}

/// A video that YouTube bot-challenges without a visitor identity.
///
/// `_T2JFb0tPtI` was reported failing in the app with "LOGIN_REQUIRED: Sign in
/// to confirm you're not a bot" while a control video succeeded from the same
/// machine at the same moment. Sending a scraped `visitorData` flipped it to
/// OK. This guards that fix: if the visitor layer regresses, this fails while
/// `extracts_a_playable_stream` keeps passing, which localises the cause.
#[tokio::test]
#[ignore = "network"]
async fn extracts_a_bot_challenged_video() {
    let stream = player::extract("_T2JFb0tPtI")
        .await
        .expect("visitor identity should clear the bot challenge");

    println!(
        "client={} itag={} bytes={:?}",
        stream.client_name, stream.format.itag, stream.format.content_length
    );
    assert!(stream.format.url.starts_with("https://"));
    assert!(stream.format.content_length.unwrap_or(0) > 0);
}

/// The visitor token has to actually parse out of a live YouTube page - the
/// scrape is the single point of failure for the whole bot-challenge fix.
#[tokio::test]
#[ignore = "network"]
async fn scrapes_a_visitor_id() {
    let token = super::visitor::get().await.expect("should scrape a visitor id");
    println!("visitorData: {}…  ({} chars)", &token[..token.len().min(32)], token.len());

    // Real tokens are long base64url protobufs; a short or HTML-ish result
    // means the scrape matched the wrong thing.
    assert!(token.len() > 20, "implausibly short token: {token:?}");
    assert!(!token.contains('<'), "scraped markup, not a token: {token:?}");
    assert!(!token.contains("\\u"), "token still JSON-escaped: {token:?}");
}

/// Where the time actually goes on a cold play.
///
/// Run before optimising anything here - the stages have very different costs
/// and the expensive one is not the obvious one.
#[tokio::test]
#[ignore = "network"]
async fn profiles_a_cold_play() {
    use std::time::Instant;

    let q = weeknd_query();

    let t0 = Instant::now();
    let _ = super::visitor::get().await;
    let visitor = t0.elapsed();

    let t = Instant::now();
    let results = search::songs("Blinding Lights The Weeknd").await.unwrap();
    let search_ms = t.elapsed();

    let t = Instant::now();
    let m = matching::best_match(&q, &results[..results.len().min(10)]).unwrap();
    let match_ms = t.elapsed();

    let t = Instant::now();
    let stream = player::extract(&m.video_id).await.unwrap();
    let extract_ms = t.elapsed();

    let t = Instant::now();
    let bytes = stream::download(&stream).await.unwrap();
    let download_ms = t.elapsed();

    let t = Instant::now();
    let _ = rodio::Decoder::new_mp4(std::io::Cursor::new(bytes.clone())).unwrap();
    let decode_ms = t.elapsed();

    let total = visitor + search_ms + match_ms + extract_ms + download_ms + decode_ms;
    println!("\n--- cold play profile ({} bytes) ---", bytes.len());
    println!("  visitor scrape  {:>7.0}ms", visitor.as_secs_f64() * 1000.0);
    println!("  YTM search      {:>7.0}ms", search_ms.as_secs_f64() * 1000.0);
    println!("  matching        {:>7.1}ms", match_ms.as_secs_f64() * 1000.0);
    println!("  player extract  {:>7.0}ms", extract_ms.as_secs_f64() * 1000.0);
    println!("  download (full) {:>7.0}ms  <-- serial 1MiB chunks", download_ms.as_secs_f64() * 1000.0);
    println!("  decoder open    {:>7.0}ms", decode_ms.as_secs_f64() * 1000.0);
    println!("  TOTAL           {:>7.0}ms", total.as_secs_f64() * 1000.0);
}

/// What prefetching actually buys.
///
/// The download is bandwidth-bound (parallelising the ranged requests changed
/// nothing measurable), so the ~1s cost of starting a track cannot be shrunk -
/// it can only be moved off the critical path. This measures that move: the
/// second fetch should be a cache hit and effectively free.
#[tokio::test]
#[ignore = "network"]
async fn prefetch_makes_the_second_fetch_free() {
    use std::time::Instant;

    let video = "dQw4w9WgXcQ";

    let t = Instant::now();
    let stream = player::extract(video).await.unwrap();
    let audio: std::sync::Arc<[u8]> = stream::download(&stream).await.unwrap().into();
    let cold = t.elapsed();

    // Stand in for the cache round trip: what playback does with a prepared
    // track is clone an Arc and open a decoder over it.
    let t = Instant::now();
    let shared = std::sync::Arc::clone(&audio);
    let _ = rodio::Decoder::new_mp4(std::io::Cursor::new(shared)).unwrap();
    let warm = t.elapsed();

    println!(
        "\n  cold (extract + download + decode) {:>6.0}ms\n  prepared (Arc clone + decode)      {:>6.1}ms\n",
        cold.as_secs_f64() * 1000.0,
        warm.as_secs_f64() * 1000.0
    );

    assert!(!audio.is_empty());
    // The point of the Arc<[u8]>: handing a prepared track to the decoder must
    // not copy 3-4 MB. If this regresses to a memcpy it will show up here.
    assert!(
        warm.as_millis() < 100,
        "prepared start took {warm:?} - expected near-instant"
    );
}

/// The decode path, end to end, on real bytes.
///
/// This is the claim the whole codec choice rests on: that the symphonia
/// AAC + isomp4 decoders already vendored via rodio can play what YouTube
/// hands back for itag 140, with no new dependency. Asserting on the container
/// bytes is not enough - only actually decoding proves it.
///
/// Deliberately decodes rather than plays: opening an audio device in a test
/// is environment-dependent and would make this flaky for no extra coverage.
#[tokio::test]
#[ignore = "network"]
async fn real_stream_decodes_to_audio_samples() {
    use rodio::Source;

    let stream = player::extract("dQw4w9WgXcQ").await.unwrap();
    let bytes = stream::download(&stream).await.unwrap();

    assert!(
        stream.format.mime_type.contains("mp4"),
        "expected an MP4 container, got {}",
        stream.format.mime_type
    );

    let decoder = rodio::Decoder::new_mp4(std::io::Cursor::new(bytes))
        .expect("symphonia should decode YouTube's itag 140");

    let channels    = decoder.channels();
    let sample_rate = decoder.sample_rate();
    println!("decoded: {channels}ch @ {sample_rate}Hz");

    assert!((1..=8).contains(&channels), "implausible channel count {channels}");
    assert!(
        (8_000..=192_000).contains(&sample_rate),
        "implausible sample rate {sample_rate}"
    );

    // Pull real audio through. A container that parses but yields silence (or
    // nothing) would still be a broken playback path.
    let taken: Vec<f32> = decoder.take(48_000).collect();
    assert!(
        taken.len() > 40_000,
        "decoder produced only {} samples - stream is truncated or unplayable",
        taken.len()
    );
    assert!(
        taken.iter().any(|s| s.abs() > 0.001),
        "decoded {} samples but all were silence",
        taken.len()
    );
}
