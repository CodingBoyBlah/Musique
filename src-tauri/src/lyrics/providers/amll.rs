//! the amll ttml database (<https://github.com/amll-dev/amll-ttml-db>).
//!
//! a community database of hand-corrected, word-level apple-music-style TTML,
//! indexed by spotify track id among others. that indexing is the whole point:
//! every other provider has to search by title + artist and then prove the hit
//! is the same recording, and every one of them occasionally picks a different
//! master. here the spotify id *is* the key, so there is no search, no match
//! heuristic and no version ambiguity - one static GET either hits the exact
//! recording or 404s. when it hits it's the best candidate in the system,
//! because a human aligned it.
//!
//! the TTML parser is shared with the other apple-music-format providers and
//! lives in `lyrics/ttml.rs`.
//! dependency and never panics on a malformed submission.

use super::UA;
use crate::lyrics::parse::scrub_spaces;
use crate::lyrics::ttml::parse_ttml;
use crate::lyrics::types::{Candidate, TrackRef};

/// verified live: `.../main/spotify-lyrics/<spotifyTrackId>.ttml` returns 200
/// for an indexed track and 404 otherwise. the repo moved from `Steve-xmh` to
/// the `amll-dev` org; the old owner path still redirects, but point at the
/// current one so we're not relying on github keeping the rename alias
const BASE: &str = "https://raw.githubusercontent.com/amll-dev/amll-ttml-db/main/spotify-lyrics";

pub async fn fetch(track: &TrackRef) -> Option<Candidate> {
    if track.id.is_empty() || !track.id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None; // the id goes straight into a path, so keep it base62
    }

    let url = format!("{BASE}/{}.ttml", track.id);
    let resp = crate::http::cookie_client()
        .get(&url)
        .header("User-Agent", UA)
        .send()
        .await
        .ok()?;

    // 404 is the common answer - the db only covers a few hundred thousand
    // tracks - and it is not an error, just "no entry"
    if !resp.status().is_success() {
        return None;
    }

    let body = resp.text().await.ok()?;
    let lines = parse_ttml(&body);
    if lines.is_empty() {
        return None;
    }
    let mut cand = Candidate::new("amll", lines);
    // keyed by the spotify track id, so it is this recording by construction -
    // its hand-corrected timings must not be shifted onto the reference
    cand.exact = true;
    Some(cand)
}
