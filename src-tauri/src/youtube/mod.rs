// YouTube Music audio backend for free Spotify accounts.
//
// Spotify remains the source of truth for everything the user sees - metadata,
// artwork, lyrics, library, discovery, playback history. The only thing sourced
// from YouTube is the decoded audio, because librespot streaming requires
// Premium. Nothing in this module writes to the Spotify-facing data model.
//
// The pipeline:
//
//   Spotify track  --search.rs-->  candidate songs
//                  --matching.rs-> one video id, or a refusal
//                  --player.rs---> a direct CDN audio URL
//                  --stream.rs---> bytes, fetched in bounded ranges
//
// Two design decisions are load-bearing and should not be relaxed casually:
//
//  1. **Matching refuses rather than guesses.** `resolve` returns an error when
//     no candidate clears the gates. Playing a plausible-but-wrong recording is
//     worse than playing nothing, because it is undetectable without listening.
//
//  2. **Media is fetched in bounded ranges, never as one open-ended GET.**
//     See stream.rs - an unbounded GET is throttled to ~0.25x realtime.
//
// Legal posture: this moves ToS exposure from Spotify to Google, whose terms
// prohibit stream extraction. It does not eliminate it. Same posture as
// NewPipe / yt-dlp / InnerTune / Metrolist.

pub mod client;
pub mod format;
pub mod innertube;
pub mod matching;
pub mod player;
pub mod search;
pub mod stream;
pub mod visitor;

use sqlx::SqlitePool;

use crate::{db::repos::yt_match, errors::AppError};

pub use matching::TrackQuery;
pub use player::ExtractedStream;

/// How many search results to consider. YouTube's songs filter returns ~20;
/// correct matches land in the first handful, and a long tail only adds chances
/// for a near-miss to sneak past the gates.
const MAX_CANDIDATES: usize = 10;

/// Resolve a Spotify track to a YouTube video id, using and populating the
/// cache.
///
/// Returns `Err(NotFound)` when nothing acceptable exists - a normal outcome
/// that callers must surface to the user rather than retry blindly.
pub async fn resolve(
    pool: &SqlitePool,
    track_id: &str,
    query: &TrackQuery,
) -> Result<String, AppError> {
    if let Some(cached) = yt_match::get(pool, track_id).await? {
        return match cached.video_id {
            Some(v) => Ok(v),
            None => Err(AppError::NotFound(format!(
                "no YouTube Music match for \"{}\" by {} ({})",
                query.title,
                query.artists.first().map(String::as_str).unwrap_or("unknown artist"),
                cached.reason.as_deref().unwrap_or("no acceptable candidate"),
            ))),
        };
    }

    // Query shape matters: "title artist" is what YouTube Music's own search
    // box is tuned for. Including the album tends to *hurt* - it pulls in
    // compilations and reissues that then have to be gated out.
    let terms = format!(
        "{} {}",
        query.title,
        query.artists.first().map(String::as_str).unwrap_or_default()
    );
    let candidates = search::songs(terms.trim()).await?;
    let considered = &candidates[..candidates.len().min(MAX_CANDIDATES)];

    match matching::best_match(query, considered) {
        Some(m) => {
            eprintln!(
                "[youtube] {track_id} -> {} (score {:.2}; {})",
                m.video_id, m.score, m.reason
            );
            yt_match::put(pool, track_id, &m.video_id, m.score, &m.reason, m.duration_ms).await?;
            Ok(m.video_id)
        }
        None => {
            let reason = format!(
                "{} candidate(s) searched, none cleared matching",
                considered.len()
            );
            yt_match::put_negative(pool, track_id, &reason).await?;
            Err(AppError::NotFound(format!(
                "no YouTube Music match for \"{}\" by {} ({reason})",
                query.title,
                query.artists.first().map(String::as_str).unwrap_or("unknown artist"),
            )))
        }
    }
}

/// Build a match query from the locally cached Spotify catalog.
///
/// Reads from the local tables rather than the Web API because the catalog is
/// already populated for anything the user can press play on, and a network
/// round trip here would sit directly in the play path. `explicit` and
/// `duration_ms` in particular are matcher gates, so they must come from
/// Spotify's own data - never from anything YouTube told us.
pub async fn query_for_track(pool: &SqlitePool, track_id: &str) -> Result<TrackQuery, AppError> {
    let row: Option<(String, i64, bool, Option<String>)> = sqlx::query_as(
        "SELECT t.name, t.duration_ms, t.explicit, al.name
           FROM tracks t
           LEFT JOIN albums al ON al.id = t.album_id
          WHERE t.id = ?",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await?;

    let (title, duration_ms, explicit, album) = row.ok_or_else(|| {
        AppError::NotFound(format!("track {track_id} is not in the local catalog"))
    })?;

    // Ordered by `position` so the primary artist stays first - the matcher's
    // artist gate is specifically about the lead artist.
    let artists: Vec<String> = sqlx::query_scalar(
        "SELECT a.name
           FROM track_artists ta
           JOIN artists a ON a.id = ta.artist_id
          WHERE ta.track_id = ?
          ORDER BY ta.position",
    )
    .bind(track_id)
    .fetch_all(pool)
    .await?;

    if artists.is_empty() {
        // Without an artist the matcher has no way to tell two identically
        // titled songs apart, so refuse rather than match on title alone.
        return Err(AppError::NotFound(format!(
            "track {track_id} has no artists cached; cannot match safely"
        )));
    }

    Ok(TrackQuery {
        title,
        artists,
        album,
        duration_ms: duration_ms.max(0) as u64,
        explicit,
    })
}

/// Resolve a Spotify track and extract a playable stream for it.
pub async fn resolve_stream(
    pool: &SqlitePool,
    track_id: &str,
    query: &TrackQuery,
) -> Result<ExtractedStream, AppError> {
    let video_id = resolve(pool, track_id, query).await?;
    player::extract(&video_id).await
}

/// Fetch the audio bytes for an already-extracted stream.
pub async fn fetch_audio(stream: &ExtractedStream) -> Result<Vec<u8>, AppError> {
    stream::download(stream).await
}

// ---------------------------------------------------------------------------
// prepared-track cache
// ---------------------------------------------------------------------------

/// A track resolved, extracted and downloaded, ready to hand to the decoder.
pub struct PreparedTrack {
    pub stream: ExtractedStream,
    /// `Arc<[u8]>` so handing it to a decoder is a pointer copy, not a 3-4 MB
    /// memcpy, and so the cache can keep its own reference.
    pub audio:  std::sync::Arc<[u8]>,
}

/// How many prepared tracks are held at once.
///
/// Two: the one playing and the one queued next. At ~3-4 MB of encoded audio
/// each that is under 10 MB, which is noise next to the WebView - but it is
/// still real memory, so this does not grow into a general-purpose cache.
const PREPARED_CAPACITY: usize = 2;

static PREPARED: tokio::sync::RwLock<Vec<(String, std::sync::Arc<PreparedTrack>)>> =
    tokio::sync::RwLock::const_new(Vec::new());

/// A previously prepared track, if it is still cached.
pub async fn take_prepared(track_id: &str) -> Option<std::sync::Arc<PreparedTrack>> {
    PREPARED
        .read()
        .await
        .iter()
        .find(|(id, _)| id == track_id)
        .map(|(_, t)| std::sync::Arc::clone(t))
}

/// Per-track lock so a preload and a play of the same track share one download
/// instead of racing two. Entries are removed once nobody holds them.
static INFLIGHT: std::sync::Mutex<
    Vec<(String, std::sync::Arc<tokio::sync::Mutex<()>>)>,
> = std::sync::Mutex::new(Vec::new());

fn inflight_lock(track_id: &str) -> std::sync::Arc<tokio::sync::Mutex<()>> {
    let mut map = INFLIGHT.lock().unwrap();
    if let Some((_, l)) = map.iter().find(|(id, _)| id == track_id) {
        return std::sync::Arc::clone(l);
    }
    let l = std::sync::Arc::new(tokio::sync::Mutex::new(()));
    map.push((track_id.to_string(), std::sync::Arc::clone(&l)));
    l
}

fn release_inflight(track_id: &str, lock: std::sync::Arc<tokio::sync::Mutex<()>>) {
    let mut map = INFLIGHT.lock().unwrap();
    // map + this handle; anyone else still waiting keeps the entry alive
    if std::sync::Arc::strong_count(&lock) <= 2 {
        map.retain(|(id, _)| id != track_id);
    }
}

/// Resolve, extract and download a track, caching the result.
///
/// This is the whole cost of starting playback - measured at roughly 1 second,
/// dominated by the download, which is bandwidth-bound and so cannot be made
/// meaningfully faster. Running it ahead of time for the next track is what
/// turns a track change from a one-second gap into an instant one.
///
/// Returns the existing entry if the track is already prepared, so repeated
/// preload calls for the same track are free.
pub async fn prepare(
    pool: &SqlitePool,
    track_id: &str,
    query: &TrackQuery,
) -> Result<std::sync::Arc<PreparedTrack>, AppError> {
    if let Some(hit) = take_prepared(track_id).await {
        return Ok(hit);
    }

    let lock = inflight_lock(track_id);
    let result = {
        let _guard = lock.lock().await;
        prepare_locked(pool, track_id, query).await
    };
    release_inflight(track_id, lock);
    result
}

async fn prepare_locked(
    pool: &SqlitePool,
    track_id: &str,
    query: &TrackQuery,
) -> Result<std::sync::Arc<PreparedTrack>, AppError> {
    // Whoever held the lock before us may have just finished this track.
    if let Some(hit) = take_prepared(track_id).await {
        return Ok(hit);
    }

    let stream = resolve_stream(pool, track_id, query).await?;
    let audio: std::sync::Arc<[u8]> = fetch_audio(&stream).await?.into();
    let prepared = std::sync::Arc::new(PreparedTrack { stream, audio });

    let mut cache = PREPARED.write().await;
    cache.retain(|(id, _)| id != track_id);
    cache.push((track_id.to_string(), std::sync::Arc::clone(&prepared)));
    while cache.len() > PREPARED_CAPACITY {
        cache.remove(0);
    }
    Ok(prepared)
}

/// Drop one track's prepared audio, e.g. after its match was pinned or
/// forgotten. Waits out an in-flight prepare of the same track first, so a
/// download that started before the change can't re-insert the old upload.
pub async fn evict_prepared(track_id: &str) {
    let lock = inflight_lock(track_id);
    {
        let _guard = lock.lock().await;
        PREPARED.write().await.retain(|(id, _)| id != track_id);
    }
    release_inflight(track_id, lock);
}

/// Drop every prepared track. Used when leaving the YouTube backend so its
/// buffers are not held for a session that is no longer using them.
pub async fn clear_prepared() {
    PREPARED.write().await.clear();
}

#[cfg(test)]
mod live_tests;

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_track(
        pool: &SqlitePool,
        id: &str,
        name: &str,
        duration_ms: i64,
        explicit: bool,
    ) {
        sqlx::query(
            "INSERT INTO tracks (id, name, album_id, duration_ms, track_number,
                                 disc_number, explicit, is_local, updated_at)
             VALUES (?, ?, NULL, ?, 1, 1, ?, 0, 0)",
        )
        .bind(id)
        .bind(name)
        .bind(duration_ms)
        .bind(explicit)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn add_artist(pool: &SqlitePool, track_id: &str, artist_id: &str, name: &str, pos: i64) {
        sqlx::query("INSERT OR IGNORE INTO artists (id, name, updated_at) VALUES (?, ?, 0)")
            .bind(artist_id)
            .bind(name)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO track_artists (track_id, artist_id, position) VALUES (?, ?, ?)",
        )
        .bind(track_id)
        .bind(artist_id)
        .bind(pos)
        .execute(pool)
        .await
        .unwrap();
    }

    /// Pins the contract `commands::playback::match_query` depends on.
    ///
    /// The Spotify API fallback there triggers on `AppError::NotFound` and
    /// nothing else. If this ever starts returning a different variant, every
    /// track that isn't in the local catalog silently stops playing on the
    /// YouTube backend again - which is exactly the bug this guards.
    #[sqlx::test]
    async fn missing_track_is_not_found(pool: SqlitePool) {
        let err = query_for_track(&pool, "4yownbPq8m0iEqVsjfle93")
            .await
            .expect_err("a track absent from the catalog must not resolve");
        assert!(
            matches!(err, AppError::NotFound(_)),
            "fallback only fires on NotFound, got {err:?}"
        );
    }

    /// Same contract for the other refusal path.
    #[sqlx::test]
    async fn track_without_artists_is_not_found(pool: SqlitePool) {
        insert_track(&pool, "t1", "Some Song", 200_000, false).await;
        let err = query_for_track(&pool, "t1")
            .await
            .expect_err("a track with no artists can't be matched safely");
        assert!(matches!(err, AppError::NotFound(_)), "got {err:?}");
    }

    #[sqlx::test]
    async fn builds_query_from_catalog(pool: SqlitePool) {
        insert_track(&pool, "t1", "Blinding Lights", 200_040, true).await;
        add_artist(&pool, "t1", "a1", "The Weeknd", 0).await;

        let q = query_for_track(&pool, "t1").await.unwrap();
        assert_eq!(q.title, "Blinding Lights");
        assert_eq!(q.artists, vec!["The Weeknd"]);
        assert_eq!(q.duration_ms, 200_040);
        assert!(q.explicit);
    }

    /// The matcher's artist gate is specifically about the *lead* artist, so
    /// the ordering from `track_artists.position` has to survive the read.
    #[sqlx::test]
    async fn artists_keep_their_order(pool: SqlitePool) {
        insert_track(&pool, "t1", "Starboy", 230_000, false).await;
        add_artist(&pool, "t1", "a2", "Daft Punk", 1).await;
        add_artist(&pool, "t1", "a1", "The Weeknd", 0).await;

        let q = query_for_track(&pool, "t1").await.unwrap();
        assert_eq!(q.artists, vec!["The Weeknd", "Daft Punk"]);
    }
}
