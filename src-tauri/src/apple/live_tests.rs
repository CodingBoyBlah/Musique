//! Live checks against Apple's public endpoints, which need no account: the web player's
//! bearer token can still be read off music.apple.com, and the catalog answers with it alone.
//! Ignored by default; run with `cargo test --lib apple::live_tests -- --ignored`.

use crate::apple::auth::{self, SAFARI_USER_AGENT};

#[tokio::test]
#[ignore = "talks to music.apple.com"]
async fn reads_the_web_players_bearer_token_and_searches_the_catalog() {
    let http = reqwest::Client::builder().user_agent(SAFARI_USER_AGENT).build().unwrap();
    let bearer = auth::scrape_bearer_from_test(&http, "https://music.apple.com").await.expect("bearer");
    assert_eq!(bearer.matches('.').count(), 2, "a JWT has three parts");

    let answer: serde_json::Value = http
        .get("https://amp-api.music.apple.com/v1/catalog/us/search")
        .query(&[("term", "daft punk one more time"), ("types", "songs"), ("limit", "3")])
        .header("Authorization", format!("Bearer {bearer}"))
        .header("Origin", "https://music.apple.com")
        .header("Referer", "https://music.apple.com/")
        .send()
        .await
        .expect("search request")
        .error_for_status()
        .expect("search status")
        .json()
        .await
        .expect("search json");
    let songs = answer.pointer("/results/songs/data").and_then(|v| v.as_array()).expect("songs");
    assert!(!songs.is_empty(), "the catalog found nothing");
    let first = &songs[0];
    println!(
        "first hit: {} by {} (isrc {})",
        first.pointer("/attributes/name").and_then(|v| v.as_str()).unwrap_or("?"),
        first.pointer("/attributes/artistName").and_then(|v| v.as_str()).unwrap_or("?"),
        first.pointer("/attributes/isrc").and_then(|v| v.as_str()).unwrap_or("?"),
    );
    // the row the app's mapper reads must survive the real shape
    let mapped = crate::apple::wire::parse_search_results(&answer.to_string());
    assert!(!mapped.tracks.is_empty(), "the wire mapper read no songs from a real answer");
    let track = &mapped.tracks[0];
    assert!(!track.name.is_empty() && !track.artists.is_empty() && track.duration_ms > 0);
    assert!(track.album.as_ref().and_then(|a| a.image_url.as_deref()).is_some_and(|u| !u.contains("{w}")));
}
