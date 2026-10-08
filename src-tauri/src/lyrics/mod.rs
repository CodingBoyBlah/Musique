//! lyrics: correctly-synced for every track, word-by-word wherever we can prove it.
//!
//! The thing that makes lyrics feel broken is almost never a provider's timing
//! quality - it's **version mismatch**. Nearly every lyrics provider matches by
//! fuzzy search (name + artist + duration) and hands back the document for a
//! *different master*: the radio edit, the remaster, the live cut. That
//! document's timing is internally consistent and completely wrong for the audio
//! actually playing.
//!
//! There is exactly one source keyed to the exact track we're playing: Spotify's
//! own `color-lyrics` endpoint, reached through the librespot session we already
//! hold open for playback. It's line-level only, but it has zero version
//! ambiguity. So it isn't a fallback - it's the **sync reference** that every
//! richer candidate has to agree with before we'll trust it.
//!
//! ```text
//! HOT PATH (1 request, always):
//!   spclient.get_lyrics(track_id)
//!       -> render LINE-LEVEL, correctly synced, instantly
//!       -> keep as the SYNC REFERENCE
//!
//! BACKGROUND (non-blocking, bounded, early-exit):
//!   word-level candidates, staged by expected yield:
//!     AMLL -> Musixmatch / NetEase -> QQ -> Kugou
//!         |
//!         +- aligns to the reference -> shift by the derived offset
//!         |                          -> silently upgrade to WORD-BY-WORD
//!         +- none align              -> stay on correctly-synced line-level
//! ```
//!
//! The governing rule, everywhere in here: **a correctly-synced line-level
//! result beats a word-level result that drifts.** Word timing is a bonus; sync
//! is the product.

pub mod align;
pub mod cache;
pub mod parse;
pub mod providers;
pub mod ttml;
pub mod types;
pub mod voices;

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Semaphore};

use crate::errors::AppError;
// re-exported so the commands layer and the frontend contract have one import
// path; not every name is used inside this module itself
#[allow(unused_imports)]
pub use types::{Alternate, Candidate, LineRole, LyricLine, LyricWord, Lyrics, TrackRef};

/// event the frontend listens on for a silent word-by-word upgrade. the panel
/// swaps the payload into its query cache in place - no spinner, no reflow
pub const UPGRADE_EVENT: &str = "lyrics:upgraded";

/// lyrics get their OWN permit pool rather than sharing the 8-permit API
/// semaphore the rest of the app uses. background lyric fetching must never
/// queue behind (or ahead of) playback and metadata calls - those are what the
/// user is actually waiting on
fn sem() -> &'static Semaphore {
    static SEM: OnceLock<Semaphore> = OnceLock::new();
    SEM.get_or_init(|| Semaphore::new(3))
}

/// per-track single-flight. panel-open and queue prefetch race each other
/// constantly; without this the same track fetches twice and we pay every
/// provider round trip twice
fn inflight() -> &'static Inflight {
    static INFLIGHT: OnceLock<Inflight> = OnceLock::new();
    INFLIGHT.get_or_init(Inflight::default)
}

#[derive(Default)]
struct Inflight {
    tracks: StdMutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl Inflight {
    fn track_lock(&self, track_id: &str) -> TrackLock<'_> {
        let mut tracks = self.tracks.lock().unwrap();
        let mutex = tracks.entry(track_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        TrackLock { registry: self, track_id: track_id.to_string(), mutex }
    }
}

/// Keep the registry entry only while a fetch owns or awaits its mutex.
/// Synchronous cleanup also runs when an async fetch is cancelled.
struct TrackLock<'a> {
    registry: &'a Inflight,
    track_id: String,
    mutex: Arc<Mutex<()>>,
}

impl Drop for TrackLock<'_> {
    fn drop(&mut self) {
        let mut tracks = self.registry.tracks.lock().unwrap();
        // The map and this handle are the last two owners. New handles can
        // only be created under this same registry lock, so removing here
        // cannot give a queued fetch a different mutex for the same track.
        if Arc::strong_count(&self.mutex) == 2 {
            tracks.remove(&self.track_id);
        }
    }
}

/// tracks whose background upgrade is already running, so a second panel open
/// doesn't kick off a duplicate race
fn upgrading_set() -> &'static Mutex<std::collections::HashSet<String>> {
    static SET: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    SET.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

// building the answer

/// turn a chosen candidate into the thing the frontend renders
fn finalize(track_id: &str, cand: &Candidate, offset_ms: i64, upgrading: bool) -> Lyrics {
    let word_level = cand.word_level();
    let has_translation = cand.lines.iter().any(|l| l.translation.is_some());
    let has_roman = cand.lines.iter().any(|l| l.roman.is_some());
    let found = !cand.lines.is_empty() || cand.plain.is_some() || cand.instrumental;

    Lyrics {
        track_id: track_id.to_string(),
        synced: !cand.lines.is_empty(),
        word_level,
        lines: cand.lines.clone(),
        plain: cand.plain.clone(),
        instrumental: cand.instrumental,
        source: cand.source.to_string(),
        found,
        offset_ms,
        alternates: Vec::new(), // filled in by the caller from the cache
        upgrading,
        has_translation,
        has_roman,
    }
}

/// a word-level result is only an upgrade over what we're already showing if it
/// actually adds word timings (or provider translation/romanisation we lacked)
fn is_upgrade(new: &Lyrics, old: &Lyrics) -> bool {
    if new.word_level && !old.word_level {
        return true;
    }
    if new.word_level == old.word_level {
        return (new.has_translation && !old.has_translation)
            || (new.has_roman && !old.has_roman);
    }
    false
}

// provider fan-out

/// run one provider under the lyrics semaphore, honouring its per-source
/// negative cache. a source that missed recently is skipped entirely - that's
/// the point of caching per (track, source) rather than per track: one
/// provider's miss must never suppress the others
macro_rules! try_provider {
    ($pool:expr, $track:expr, $source:literal, $fut:expr) => {{
        if cache::source_missed($pool, &$track.id, $source).await {
            None
        } else {
            let _permit = sem().acquire().await.ok();
            let mut got = $fut.await;
            // fold backing vocals onto their lead BEFORE anything downstream sees
            // the candidate: align.rs then compares lead-to-lead, and the
            // renderer gets rows whose ends are set by lead lines only
            if let Some(c) = got.as_mut() {
                voices::structure(c);
            }
            if got.is_none() {
                let _ = cache::mark_source_miss($pool, &$track.id, $source).await;
            }
            got
        }
    }};
}

/// does the track look CJK? decides whether NetEase (huge CJK catalogue, ships
/// translation + romanisation for free) or Musixmatch (largest western
/// catalogue) is worth trying first. purely an ordering hint - both still run
fn looks_cjk(track: &TrackRef) -> bool {
    track.name.chars().chain(track.artist.chars()).any(|c| {
        matches!(c as u32,
            0x3040..=0x30ff |   // hiragana + katakana
            0x4e00..=0x9fff |   // cjk unified
            0xac00..=0xd7af |   // hangul
            0x3400..=0x4dbf)    // cjk ext a
    })
}

/// stage 1: the Apple-Music-format word sources, keyed to the exact recording.
///
/// These are the best documents in the system and they're also the cheapest to
/// verify, so they go first and usually end the race.
///
/// `applemusic` looks the track up by **ISRC** - the identifier for the exact
/// *recording*, not the song - and hands back Apple's own TTML: real syllable
/// timings, background vocals marked `x-bg`, duet voices separated by agent.
/// There is no search and no version ambiguity in that path at all, which is
/// precisely what makes word-by-word trustworthy rather than decorative.
///
/// `amll` is the same format keyed by Spotify track id, hand-corrected to
/// ~100ms - the community answer for whatever Apple doesn't cover.
async fn stage_exact(pool: &SqlitePool, track: &TrackRef) -> Vec<Candidate> {
    let (apple, amll) = tokio::join!(
        async { try_provider!(pool, track, "applemusic", providers::applemusic::fetch(track)) },
        async { try_provider!(pool, track, "amll", providers::amll::fetch(track)) },
    );
    apple.into_iter().chain(amll).collect()
}

/// stage 2: Apple Music TTML again, but found by fuzzy search rather than by
/// ISRC - the fallback for recordings the exact lookup doesn't index. Same rich
/// format, so it still carries word timings and background vocals; it just has
/// to prove itself against the sync reference first.
async fn stage_betterlyrics(pool: &SqlitePool, track: &TrackRef) -> Vec<Candidate> {
    try_provider!(pool, track, "betterlyrics", providers::betterlyrics::fetch(track))
        .into_iter()
        .collect()
}

/// stage 3: the two big catalogues, raced together
async fn stage_major(pool: &SqlitePool, track: &TrackRef) -> Vec<Candidate> {
    let (mxm, ne) = tokio::join!(
        async { try_provider!(pool, track, "musixmatch", providers::musixmatch::fetch(track)) },
        async { try_provider!(pool, track, "netease", providers::netease::fetch(track)) },
    );
    // put the likelier-correct one first so ties break sensibly downstream
    let mut out: Vec<Candidate> = Vec::new();
    if looks_cjk(track) {
        out.extend(ne);
        out.extend(mxm);
    } else {
        out.extend(mxm);
        out.extend(ne);
    }
    out
}

/// stage 4: the encrypted-body chinese sources. lower hit rate for most
/// libraries and the most expensive to decode, so they only run when the
/// cheaper stages produced nothing trustworthy
async fn stage_cjk(pool: &SqlitePool, track: &TrackRef) -> Vec<Candidate> {
    let (qq, kg) = tokio::join!(
        async { try_provider!(pool, track, "qq", providers::qq::fetch(track)) },
        async { try_provider!(pool, track, "kugou", providers::kugou::fetch(track)) },
    );
    qq.into_iter().chain(kg).collect()
}

/// the line-level safety net, only worth reaching for when nothing else landed
async fn stage_lrclib(pool: &SqlitePool, track: &TrackRef) -> Vec<Candidate> {
    try_provider!(pool, track, "lrclib", providers::lrclib::fetch(track))
        .into_iter()
        .collect()
}

// entry point

/// Fetch (or read from cache) the lyrics for a track.
///
/// Returns as soon as there's something correctly synced to show - normally
/// after a single request to Spotify. Any word-by-word upgrade happens in a
/// background task and arrives later over [`UPGRADE_EVENT`].
pub async fn get_or_fetch(
    app: &AppHandle,
    pool: &SqlitePool,
    track: TrackRef,
    force: bool,
) -> Result<Lyrics, AppError> {
    if !force {
        if let Some(mut hit) = cache::read(pool, &track.id).await {
            hit.alternates = cache::list_alts(pool, &track.id).await;
            // only resume an upgrade that never settled (e.g. the app closed
            // mid-race). a finished race writes upgrading=false, and so does a
            // manual source pick - re-racing those would repeat every provider
            // round trip on each open and could overwrite the user's choice
            if !hit.word_level && hit.found && hit.upgrading {
                spawn_upgrade(app, pool, track.clone(), hit.clone());
                hit.upgrading = true;
            }
            return Ok(hit);
        }
    }

    if track.name.trim().is_empty() || track.artist.trim().is_empty() {
        return Ok(Lyrics::none(&track.id));
    }

    // single-flight: whoever gets here second waits, then reads the cache the
    // winner just wrote instead of repeating every provider round trip
    let lock = inflight().track_lock(&track.id);
    let _guard = lock.mutex.lock().await;
    if !force {
        if let Some(mut hit) = cache::read(pool, &track.id).await {
            hit.alternates = cache::list_alts(pool, &track.id).await;
            return Ok(hit);
        }
    }

    // HOT PATH - one request, the exact track, no ambiguity
    let mut reference = providers::spotify::fetch(app, &track).await;
    // the reference defines the rows the user actually reads, so it gets the
    // same voice structuring as every other source - Spotify lists backing
    // vocals as ordinary lines too
    if let Some(r) = reference.as_mut() {
        voices::structure(r);
    }

    if let Some(reference) = reference {
        let mut out = finalize(&track.id, &reference, 0, true);
        out.alternates = cache::list_alts(pool, &track.id).await;
        cache::write(pool, &out).await.ok();
        cache::write_alt(pool, &track.id, &out).await.ok();

        // everything richer is earned in the background, against this reference
        spawn_upgrade(app, pool, track.clone(), out.clone());
        return Ok(out);
    }

    // NO REFERENCE - Spotify has nothing for this track. We can't validate
    // against the exact recording any more, so fall back to cross-source
    // consensus: two independent providers agreeing on the timing is decent
    // evidence they're both describing the same master.
    let mut candidates = stage_exact(pool, &track).await;
    if candidates.is_empty() {
        candidates.extend(stage_betterlyrics(pool, &track).await);
    }
    candidates.extend(stage_major(pool, &track).await);
    if candidates.len() < 2 {
        candidates.extend(stage_cjk(pool, &track).await);
    }
    if candidates.is_empty() {
        candidates.extend(stage_lrclib(pool, &track).await);
    }

    // stash every source we saw, so the ui's switcher works offline later
    for c in &candidates {
        let alt = finalize(&track.id, c, 0, false);
        cache::write_alt(pool, &track.id, &alt).await.ok();
    }

    let Some((chosen, alignment)) = align::choose(None, candidates) else {
        cache::write_miss(pool, &track.id).await.ok();
        return Ok(Lyrics::none(&track.id));
    };

    let mut out = finalize(&track.id, &chosen, alignment.offset_ms, false);
    out.alternates = cache::list_alts(pool, &track.id).await;
    cache::write(pool, &out).await.ok();
    Ok(out)
}

/// Kick off the background word-by-word hunt. Never blocks the caller, and
/// every stage is gated on the alignment check so a drifting candidate can't
/// replace correctly-synced lines.
fn spawn_upgrade(app: &AppHandle, pool: &SqlitePool, track: TrackRef, current: Lyrics) {
    let app = app.clone();
    let pool = pool.clone();

    tauri::async_runtime::spawn(async move {
        // one upgrade race per track per session
        {
            let mut set = upgrading_set().lock().await;
            if !set.insert(track.id.clone()) {
                return;
            }
        }
        let result = run_upgrade(&app, &pool, &track, &current).await;
        upgrading_set().lock().await.remove(&track.id);

        if let Some(upgraded) = result {
            let _ = app.emit(UPGRADE_EVENT, &upgraded);
        }
    });
}

async fn run_upgrade(
    app: &AppHandle,
    pool: &SqlitePool,
    track: &TrackRef,
    current: &Lyrics,
) -> Option<Lyrics> {
    // rebuild the reference candidate from what we're currently showing, so the
    // alignment check compares against the exact timings on screen
    let reference = Candidate {
        source:       "spotify",
        lines:        current.lines.clone(),
        plain:        current.plain.clone(),
        instrumental: current.instrumental,
        exact:        true, // it came from the by-track-id fetch
    };
    let reference = (current.source == "spotify" && !reference.lines.is_empty())
        .then_some(reference);

    let mut seen: Vec<Candidate> = Vec::new();

    // staged by expected yield, cheapest and most trustworthy first. we stop
    // the moment something passes the check - that's the early exit that keeps
    // this from costing six round trips per track
    for stage in 0..4 {
        let batch = match stage {
            0 => stage_exact(pool, track).await,
            1 => stage_betterlyrics(pool, track).await,
            2 => stage_major(pool, track).await,
            _ => stage_cjk(pool, track).await,
        };
        if batch.is_empty() {
            continue;
        }

        for c in &batch {
            let alt = finalize(&track.id, c, 0, false);
            cache::write_alt(pool, &track.id, &alt).await.ok();
        }
        seen.extend(batch);

        // only word-level candidates can upgrade us; a line-level one is at
        // best a sideways move and at worst a worse-synced one
        let word_level: Vec<Candidate> = seen.iter().filter(|c| c.word_level()).cloned().collect();
        if word_level.is_empty() {
            continue;
        }

        if let Some((chosen, alignment)) = align::choose(reference.as_ref(), word_level) {
            if !alignment.aligned {
                continue;
            }
            let mut upgraded = finalize(&track.id, &chosen, alignment.offset_ms, false);
            if !is_upgrade(&upgraded, current) {
                continue;
            }
            upgraded.alternates = cache::list_alts(pool, &track.id).await;
            cache::write(pool, &upgraded).await.ok();

            eprintln!(
                "[lyrics] {} upgraded to word-level via {} (offset {}ms, residual {}ms)",
                track.id, upgraded.source, alignment.offset_ms, alignment.residual_ms
            );
            return Some(upgraded);
        }
    }

    // nothing aligned. we keep the correctly-synced line-level lyrics - that is
    // the right outcome, not a failure. still clear the flag on the frontend so
    // it stops expecting an upgrade.
    let _ = app;
    if current.upgrading {
        let mut settled = current.clone();
        settled.upgrading = false;
        settled.alternates = cache::list_alts(pool, &track.id).await;
        cache::write(pool, &settled).await.ok();
        return Some(settled);
    }
    None
}

/// Switch the displayed lyrics to another source we already hold cached.
/// Offline-instant: this never touches the network.
pub async fn switch_source(
    pool: &SqlitePool,
    track_id: &str,
    source: &str,
) -> Result<Lyrics, AppError> {
    let Some(mut alt) = cache::read_alt(pool, track_id, source).await else {
        return Err(AppError::NotFound(format!("no cached {source} lyrics for {track_id}")));
    };
    alt.alternates = cache::list_alts(pool, track_id).await;
    alt.upgrading = false;
    // make the choice sticky, so reopening the panel keeps the user's pick
    cache::write(pool, &alt).await.ok();
    Ok(alt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_fetches_do_not_retain_track_locks() {
        let registry = Inflight::default();
        for i in 0..10_000 {
            drop(registry.track_lock(&format!("track-{i}")));
        }
        assert!(registry.tracks.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn queued_fetch_preserves_single_flight_after_first_fetch_finishes() {
        let registry = Inflight::default();
        let first = registry.track_lock("track");
        let first_guard = first.mutex.lock().await;
        let queued = registry.track_lock("track");
        assert!(queued.mutex.try_lock().is_err());

        drop(first_guard);
        drop(first);
        let queued_guard = queued.mutex.lock().await;
        let arriving = registry.track_lock("track");
        assert!(Arc::ptr_eq(&queued.mutex, &arriving.mutex));
        assert!(arriving.mutex.try_lock().is_err());
        drop(arriving);
        drop(queued_guard);
        drop(queued);
        assert!(registry.tracks.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn cancelling_a_waiting_fetch_releases_only_its_handle() {
        let registry = Arc::new(Inflight::default());
        let active = registry.track_lock("track");
        let active_guard = active.mutex.lock().await;
        let waiting_registry = registry.clone();
        let (ready, started) = tokio::sync::oneshot::channel();
        let waiter = tokio::spawn(async move {
            let waiting = waiting_registry.track_lock("track");
            ready.send(()).unwrap();
            let _guard = waiting.mutex.lock().await;
        });
        started.await.unwrap();
        waiter.abort();
        assert!(waiter.await.unwrap_err().is_cancelled());
        assert_eq!(registry.tracks.lock().unwrap().len(), 1);
        assert_eq!(Arc::strong_count(&active.mutex), 2);

        drop(active_guard);
        drop(active);
        assert!(registry.tracks.lock().unwrap().is_empty());
    }

    fn cand(source: &'static str, lines: Vec<LyricLine>) -> Candidate {
        Candidate::new(source, lines)
    }

    #[test]
    fn cjk_detection_drives_provider_order() {
        let mut t = TrackRef { name: "Lemon".into(), artist: "米津玄師".into(), ..Default::default() };
        assert!(looks_cjk(&t));
        t.artist = "Kenshi Yonezu".into();
        assert!(!looks_cjk(&t));
    }

    #[test]
    fn word_level_counts_as_an_upgrade() {
        let mut old = finalize("t", &cand("spotify", vec![LyricLine::line(0, "hi")]), 0, true);
        let new = finalize(
            "t",
            &cand("amll", vec![LyricLine::worded(0, None, vec![LyricWord { time_ms: 0, end_ms: 100, text: "hi".into() }])]),
            0,
            false,
        );
        assert!(is_upgrade(&new, &old));
        // and the same thing twice is not an upgrade
        old = new.clone();
        assert!(!is_upgrade(&new, &old));
    }

    #[test]
    fn translation_alone_upgrades_a_word_level_result() {
        let w = || vec![LyricLine::worded(0, None, vec![LyricWord { time_ms: 0, end_ms: 100, text: "hi".into() }])];
        let old = finalize("t", &cand("qq", w()), 0, false);

        let mut lines = w();
        lines[0].translation = Some("bonjour".into());
        let new = finalize("t", &cand("netease", lines), 0, false);

        assert!(is_upgrade(&new, &old));
    }
}
