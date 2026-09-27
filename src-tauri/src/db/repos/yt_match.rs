// Cached Spotify track -> YouTube Music video mappings.
//
// See migrations/0011_yt_match.sql for why these are persisted at all. The
// short version: caching makes a match *stable*, which matters more than the
// latency win. Re-running a search months later can rank differently, so an
// uncached mapping could silently start resolving to a different recording of
// the same song between one play and the next.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::errors::AppError;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct YtMatch {
    pub track_id:    String,
    /// `None` is a cached "no acceptable match", not a cache miss.
    pub video_id:    Option<String>,
    pub score:       Option<f64>,
    pub reason:      Option<String>,
    pub duration_ms: Option<i64>,
    pub pinned:      bool,
    pub checked_at:  i64,
}

/// How long a cached "no acceptable match" stays authoritative.
///
/// Negative results deserve a TTL because they are frequently about YouTube's
/// catalogue rather than the track: a release can show up on YouTube Music
/// weeks after Spotify. Positive matches never expire - a correct mapping does
/// not go stale, and re-resolving one risks replacing it with a worse pick.
const NEGATIVE_TTL_SECS: i64 = 14 * 24 * 60 * 60;

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Look up a usable cached mapping.
///
/// Returns `None` both for a genuine miss and for an expired negative result -
/// in either case the caller should re-resolve. A live negative result is
/// returned as `Some(row)` with `video_id: None` so the caller can skip the
/// search entirely.
pub async fn get(pool: &SqlitePool, track_id: &str) -> Result<Option<YtMatch>, AppError> {
    let row: Option<YtMatch> = sqlx::query_as(
        "SELECT track_id, video_id, score, reason, duration_ms, pinned, checked_at
           FROM yt_track_match
          WHERE track_id = ?",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.filter(|r| {
        // A pinned row is a user decision and always wins, expiry included.
        r.pinned || r.video_id.is_some() || now_secs() - r.checked_at < NEGATIVE_TTL_SECS
    }))
}

/// Record a successful match.
///
/// Never overwrites a pinned row: a user override outranks anything the
/// matcher produces later.
pub async fn put(
    pool: &SqlitePool,
    track_id: &str,
    video_id: &str,
    score: f64,
    reason: &str,
    duration_ms: Option<u64>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO yt_track_match
             (track_id, video_id, score, reason, duration_ms, pinned, checked_at)
         VALUES (?, ?, ?, ?, ?, 0, ?)
         ON CONFLICT(track_id) DO UPDATE SET
             video_id    = excluded.video_id,
             score       = excluded.score,
             reason      = excluded.reason,
             duration_ms = excluded.duration_ms,
             checked_at  = excluded.checked_at
         WHERE yt_track_match.pinned = 0",
    )
    .bind(track_id)
    .bind(video_id)
    .bind(score)
    .bind(reason)
    .bind(duration_ms.map(|d| d as i64))
    .bind(now_secs())
    .execute(pool)
    .await?;
    Ok(())
}

/// Record that no candidate cleared the matcher's gates.
pub async fn put_negative(pool: &SqlitePool, track_id: &str, reason: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO yt_track_match
             (track_id, video_id, score, reason, duration_ms, pinned, checked_at)
         VALUES (?, NULL, NULL, ?, NULL, 0, ?)
         ON CONFLICT(track_id) DO UPDATE SET
             video_id    = NULL,
             score       = NULL,
             reason      = excluded.reason,
             duration_ms = NULL,
             checked_at  = excluded.checked_at
         WHERE yt_track_match.pinned = 0",
    )
    .bind(track_id)
    .bind(reason)
    .bind(now_secs())
    .execute(pool)
    .await?;
    Ok(())
}

/// Pin a user-chosen video id for a track, overriding automatic matching.
///
/// This is the escape hatch for the case the matcher cannot solve on its own:
/// a track that genuinely has no clean Art Track, or one where the user wants
/// a specific upload. Pinned rows are never re-resolved.
pub async fn pin(pool: &SqlitePool, track_id: &str, video_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO yt_track_match
             (track_id, video_id, score, reason, duration_ms, pinned, checked_at)
         VALUES (?, ?, NULL, 'pinned by user', NULL, 1, ?)
         ON CONFLICT(track_id) DO UPDATE SET
             video_id   = excluded.video_id,
             reason     = excluded.reason,
             pinned     = 1,
             checked_at = excluded.checked_at",
    )
    .bind(track_id)
    .bind(video_id)
    .bind(now_secs())
    .execute(pool)
    .await?;
    Ok(())
}

/// Drop a cached mapping so the next play re-resolves it. Clears pins too -
/// this is the "this match is wrong, try again" action.
pub async fn forget(pool: &SqlitePool, track_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM yt_track_match WHERE track_id = ?")
        .bind(track_id)
        .execute(pool)
        .await?;
    Ok(())
}
