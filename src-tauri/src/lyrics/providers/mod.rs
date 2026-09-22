//! lyric providers.
//!
//! every provider is best-effort and infallible from the caller's point of
//! view: it returns `Some(Candidate)` when it found usable timed lyrics for
//! *this* track and `None` for anything else (network error, no match, wrong
//! version, empty body). a provider must never propagate an error, because one
//! dead source must never block the union of all the others.
//!
//! all of them share the pooled cookie client from `crate::http` so nothing
//! pays a fresh TLS handshake, and all of them verify the match by duration
//! (and ISRC where the source exposes it) before returning - matching the wrong
//! master is the single biggest cause of "unsynced" lyrics.

pub mod amll;
pub mod applemusic;
pub mod betterlyrics;
pub mod kugou;
pub mod lrclib;
pub mod musixmatch;
pub mod netease;
pub mod qq;
pub mod spotify;

/// duration tolerance for accepting a provider's match as the same recording.
/// the old netease path used 6s, which happily returned a different master;
/// 1.5s is tight enough to pin the right one while surviving providers that
/// round to whole seconds
pub const DUR_TOL_MS: i64 = 1500;

/// a provider's search hit is only the same recording if its duration is within
/// `DUR_TOL_MS`. `cand_ms` of 0 means the provider didn't say, which we accept
/// only when nothing better matched
pub fn duration_ok(cand_ms: i64, want_ms: i64) -> bool {
    cand_ms > 0 && (cand_ms - want_ms).abs() <= DUR_TOL_MS
}

/// loose title/artist agreement, used as a second gate on search results so a
/// duration coincidence alone can't pick a totally different song
pub fn text_matches(cand: &str, want: &str) -> bool {
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    let (a, b) = (norm(cand), norm(want));
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a.contains(&b) || b.contains(&a)
}

/// browser-ish UA. several of these providers 403 a default reqwest agent
pub const UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
