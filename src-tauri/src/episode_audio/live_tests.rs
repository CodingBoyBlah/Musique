// network checks for the episode audio path, `#[ignore]` like the youtube
// ones:
//
//     cargo test --lib episode_audio::live -- --ignored --nocapture

use std::time::{Duration, Instant};

use rodio::Source;

use super::*;

fn huberman() -> EpisodeInfo {
    EpisodeInfo {
        name: String::new(),
        show: "Huberman Lab".into(),
        publisher: "Scicomm Media".into(),
        release_date: None,
        duration_ms: 0,
    }
}

async fn memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE extension_cache (entity_uri TEXT NOT NULL, kind TEXT NOT NULL, payload BLOB NOT NULL, fetched_at INTEGER NOT NULL, PRIMARY KEY (entity_uri, kind))",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
#[ignore = "network"]
async fn finds_a_feed_and_matches_its_latest_episode() {
    let pool = memory_pool().await;
    let feeds = feeds_for(&pool, "Huberman Lab", "Scicomm Media").await;
    println!("feeds: {feeds:?}");
    assert!(!feeds.is_empty());
    let items = feed_items(&feeds[0]).await.unwrap();
    assert!(items.len() > 10, "only {} items", items.len());
    let latest = &items[0];
    println!("latest: {latest:?}");

    let mut ep = huberman();
    ep.name = latest.title.clone();
    ep.release_date = latest.date.map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"));
    ep.duration_ms = latest.duration_ms.unwrap_or(0);
    let Some(Pick::Rss { url, .. }) = rss_pick(&pool, &ep).await else { panic!("no rss match") };
    assert_eq!(url, latest.url);
}

#[tokio::test]
#[ignore = "network"]
async fn youtube_episode_search_parses() {
    let rows = crate::youtube::search::episodes("Huberman Lab sleep").await.unwrap();
    for r in rows.iter().take(8) {
        println!("{} | {} | {:?} | {:?}", r.video_id, r.title, r.duration_ms, r.artists);
    }
    assert!(!rows.is_empty(), "no episode rows parsed");
}

/// the whole playback chain minus the speakers: stream a long episode, decode
/// the start, then jump near the end and decode there - without the download
/// ever having to reach it in order
#[tokio::test(flavor = "multi_thread")]
#[ignore = "network"]
async fn streams_and_seeks_a_long_episode() {
    let items = feed_items("https://feeds.megaphone.fm/hubermanlab").await.unwrap();
    let item = items.iter().find(|i| i.duration_ms.unwrap_or(0) > 60 * 60_000).expect("an hour+ episode");
    println!("{} ({:?} ms) {}", item.title, item.duration_ms, item.url);

    let t = Instant::now();
    let opened = remote::open(&item.url, None).await.unwrap();
    println!("opened {} bytes, type {:?}, in {:?}", opened.len, opened.content_type, t.elapsed());
    let handle = opened.handle.clone();
    let len = opened.len;
    let dur = item.duration_ms.unwrap();

    let far = Duration::from_millis(dur - 5 * 60_000);
    let byte = len * far.as_millis() as u64 / dur;
    let t = Instant::now();
    handle.prefetch(byte.saturating_sub(64 * 1024), Duration::from_secs(8)).await;
    println!("prefetched the far spot in {:?}", t.elapsed());

    let reader = opened.reader;
    tokio::task::spawn_blocking(move || {
        let _ = len;
        let dec = source::EpisodeDecoder::open(reader, Some("audio/mpeg".into())).expect("decoder");
        println!("{} Hz, {} ch", dec.sample_rate(), dec.channels());
        let clock = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let mut src = source::EpisodeSource::new(dec, clock.clone(), 0);
        let t = Instant::now();
        let n = src.by_ref().take(44_100 * 2 * 5).filter(|s| s.abs() > 0.0).count();
        println!("decoded 5s from the start ({n} non-silent samples) in {:?}", t.elapsed());
        assert!(n > 1000);

        let t = Instant::now();
        src.try_seek(far).expect("seek");
        let n = src.by_ref().take(44_100 * 2 * 5).filter(|s| s.abs() > 0.0).count();
        println!("seeked to {far:?} and decoded 5s ({n} non-silent) in {:?}", t.elapsed());
        assert!(n > 1000);
        let at = clock.load(std::sync::atomic::Ordering::Relaxed);
        assert!(at >= far.as_millis() as u64, "clock {at}");
    })
    .await
    .unwrap();
}

/// the youtube fallback: an mp4 stream through the same reader, with a seek
/// past the halfway mark
#[tokio::test(flavor = "multi_thread")]
#[ignore = "network"]
async fn streams_and_seeks_a_youtube_episode() {
    let rows = crate::youtube::search::episodes("Huberman Lab Using Deliberate Cold Exposure").await.unwrap();
    let row = rows.first().expect("a row");
    let stream = crate::youtube::player::extract(&row.video_id).await.unwrap();
    println!("{} {:?} itag {} {}", row.title, stream.duration_ms, stream.format.itag, stream.format.mime_type);
    let dur = stream.duration_ms.unwrap();

    let t = Instant::now();
    let opened = remote::open(&stream.format.url, Some(stream.user_agent)).await.unwrap();
    println!("opened {} bytes in {:?}", opened.len, t.elapsed());
    let len = opened.len;
    let reader = opened.reader;
    let mime = stream.format.mime_type.split(';').next().unwrap().trim().to_string();
    tokio::task::spawn_blocking(move || {
        let t = Instant::now();
        let _ = len;
        let dec = source::EpisodeDecoder::open(reader, Some(mime)).expect("decoder");
        assert!(dec.segments().is_some(), "no fragment index");
        println!("probed in {:?}: {} Hz, {} ch", t.elapsed(), dec.sample_rate(), dec.channels());
        let clock = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let mut src = source::EpisodeSource::new(dec, clock.clone(), 0);
        let n = src.by_ref().take(44_100 * 2 * 3).filter(|s| s.abs() > 0.0).count();
        assert!(n > 1000);
        let far = Duration::from_millis(dur * 2 / 3);
        let t = Instant::now();
        src.try_seek(far).expect("seek");
        let n = src.by_ref().take(44_100 * 2 * 3).filter(|s| s.abs() > 0.0).count();
        println!("seeked to {far:?} and decoded 3s ({n} non-silent) in {:?}", t.elapsed());
        assert!(n > 1000);
    })
    .await
    .unwrap();
}

