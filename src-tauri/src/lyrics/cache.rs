//! parsed lyrics cache.
//!
//! stores the finalized `Lyrics` struct as JSON to skip the expensive
//! re-parsing of raw LRC / YRC / richsync bodies on every read.

use serde::Deserialize;
use sqlx::SqlitePool;

use crate::errors::AppError;
use super::types::{Alternate, Lyrics};

/// parser revision: bumping this invalidates older cached payloads automatically
/// without needing a database migration
pub const SCHEMA_VERSION: i32 = 1;

/// whole-track miss TTL: 7 days. a track might have lyrics added later,
/// but we should avoid hammering providers repeatedly in the short term
const WHOLE_TRACK_MISS_TTL_SECS: i64 = 7 * 24 * 60 * 60;

/// per-source miss TTL: 24 hours. provider rate-limits, transient network
/// glitches, or catalog additions warrant a much sooner retry than whole-track misses
const SOURCE_MISS_TTL_SECS: i64 = 24 * 60 * 60;

fn now_sec() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(sqlx::FromRow)]
struct CacheRow {
    payload:        Option<String>,
    schema_version: i32,
    found:          bool,
    fetched_at:     i64,
}

#[derive(sqlx::FromRow)]
struct AltRow {
    source:     String,
    word_level: bool,
    synced:     bool,
    payload:    String,
}

/// minimal struct to count lines on the hot path without allocating full LyricLine structs
#[derive(Deserialize)]
struct AltLinesOnly {
    #[serde(default)]
    lines: Vec<serde::de::IgnoredAny>,
}

/// fetch cached lyrics for a track. returns None on row absence, missing payload,
/// schema version mismatch, deserialization failure, or an expired negative cache entry
pub async fn read(pool: &SqlitePool, track_id: &str) -> Option<Lyrics> {
    let row = sqlx::query_as::<_, CacheRow>(
        "SELECT payload, schema_version, found, fetched_at
         FROM lyrics
         WHERE track_id = ?",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await
    .ok()??;

    // parser changes invalidate older cached payloads automatically
    if row.schema_version != SCHEMA_VERSION {
        return None;
    }

    let age = now_sec().saturating_sub(row.fetched_at);
    if !row.found && age >= WHOLE_TRACK_MISS_TTL_SECS {
        return None;
    }

    let payload = row.payload?;
    let lyrics = serde_json::from_str::<Lyrics>(&payload).ok()?;

    if !lyrics.found && age >= WHOLE_TRACK_MISS_TTL_SECS {
        return None;
    }

    Some(lyrics)
}

/// persist lyrics payload to cache. keeps legacy v1 columns truthful
/// while populating v2 JSON payload and metadata columns (skips raw synced_lrc)
pub async fn write(pool: &SqlitePool, lyrics: &Lyrics) -> Result<(), AppError> {
    let payload = serde_json::to_string(lyrics)
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        "INSERT INTO lyrics
             (track_id, plain, source, instrumental, found, fetched_at,
              payload, offset_ms, schema_version, word_level)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(track_id) DO UPDATE SET
             plain          = excluded.plain,
             source         = excluded.source,
             instrumental   = excluded.instrumental,
             found          = excluded.found,
             fetched_at     = excluded.fetched_at,
             payload        = excluded.payload,
             offset_ms      = excluded.offset_ms,
             schema_version = excluded.schema_version,
             word_level     = excluded.word_level",
    )
    .bind(&lyrics.track_id)
    .bind(&lyrics.plain)
    .bind(&lyrics.source)
    .bind(lyrics.instrumental)
    .bind(lyrics.found)
    .bind(now_sec())
    .bind(payload)
    .bind(lyrics.offset_ms)
    .bind(SCHEMA_VERSION)
    .bind(lyrics.word_level)
    .execute(pool)
    .await?;

    Ok(())
}

/// record a whole-track miss so subsequent requests do not re-query providers
pub async fn write_miss(pool: &SqlitePool, track_id: &str) -> Result<(), AppError> {
    write(pool, &Lyrics::none(track_id)).await
}

/// read a specific cached alternate source for a track
pub async fn read_alt(pool: &SqlitePool, track_id: &str, source: &str) -> Option<Lyrics> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT payload
         FROM lyrics_alt
         WHERE track_id = ? AND source = ?",
    )
    .bind(track_id)
    .bind(source)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let (payload,) = row?;
    serde_json::from_str::<Lyrics>(&payload).ok()
}

/// store a parsed candidate as an alternate source for offline switching
pub async fn write_alt(pool: &SqlitePool, track_id: &str, lyrics: &Lyrics) -> Result<(), AppError> {
    let payload = serde_json::to_string(lyrics)
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        "INSERT INTO lyrics_alt
             (track_id, source, payload, word_level, synced, fetched_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(track_id, source) DO UPDATE SET
             payload    = excluded.payload,
             word_level = excluded.word_level,
             synced     = excluded.synced,
             fetched_at = excluded.fetched_at",
    )
    .bind(track_id)
    .bind(&lyrics.source)
    .bind(payload)
    .bind(lyrics.word_level)
    .bind(lyrics.synced)
    .bind(now_sec())
    .execute(pool)
    .await?;

    Ok(())
}

/// list all cached alternate sources for a track. reads source and flag columns directly,
/// extracting only the line count from payload without fully deserializing lyric lines
pub async fn list_alts(pool: &SqlitePool, track_id: &str) -> Vec<Alternate> {
    let rows: Vec<AltRow> = sqlx::query_as(
        "SELECT source, word_level, synced, payload
         FROM lyrics_alt
         WHERE track_id = ?
         ORDER BY source",
    )
    .bind(track_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| {
            let lines = serde_json::from_str::<AltLinesOnly>(&r.payload)
                .map(|p| p.lines.len())
                .unwrap_or(0);

            Alternate {
                source:     r.source,
                word_level: r.word_level,
                synced:     r.synced,
                lines,
            }
        })
        .collect()
}

/// check whether a specific provider has recorded a negative result within its TTL
pub async fn source_missed(pool: &SqlitePool, track_id: &str, source: &str) -> bool {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT fetched_at
         FROM lyrics_miss
         WHERE track_id = ? AND source = ?",
    )
    .bind(track_id)
    .bind(source)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some((fetched_at,)) = row else {
        return false;
    };

    now_sec().saturating_sub(fetched_at) < SOURCE_MISS_TTL_SECS
}

/// record a miss for a specific provider
pub async fn mark_source_miss(
    pool:     &SqlitePool,
    track_id: &str,
    source:   &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO lyrics_miss (track_id, source, fetched_at)
         VALUES (?, ?, ?)
         ON CONFLICT(track_id, source) DO UPDATE SET
             fetched_at = excluded.fetched_at",
    )
    .bind(track_id)
    .bind(source)
    .bind(now_sec())
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lyrics::types::{LyricLine, LyricWord};

    fn sample_lyrics(track_id: &str) -> Lyrics {
        Lyrics {
            track_id:        track_id.to_string(),
            lines:           vec![
                LyricLine::line(1000, "first line"),
                LyricLine::worded(
                    2000,
                    Some("second line".to_string()),
                    vec![
                        LyricWord { time_ms: 2000, end_ms: 2500, text: "second ".to_string() },
                        LyricWord { time_ms: 2500, end_ms: 3000, text: "line".to_string() },
                    ],
                ),
            ],
            plain:           Some("first line\nsecond line".to_string()),
            synced:          true,
            word_level:      true,
            instrumental:    false,
            source:          "spotify".to_string(),
            found:           true,
            offset_ms:       50,
            alternates:      Vec::new(),
            upgrading:       false,
            has_translation: false,
            has_roman:       false,
        }
    }

    #[sqlx::test]
    async fn payload_roundtrip(pool: SqlitePool) {
        let lyrics = sample_lyrics("t1");
        write(&pool, &lyrics).await.unwrap();

        let hit = read(&pool, "t1").await.unwrap();
        assert_eq!(hit.track_id, "t1");
        assert_eq!(hit.lines.len(), 2);
        assert_eq!(hit.lines[0].text, "first line");
        assert_eq!(hit.lines[1].words.len(), 2);
        assert_eq!(hit.plain.as_deref(), Some("first line\nsecond line"));
        assert!(hit.synced);
        assert!(hit.word_level);
        assert!(!hit.instrumental);
        assert_eq!(hit.source, "spotify");
        assert!(hit.found);
        assert_eq!(hit.offset_ms, 50);
    }

    #[sqlx::test]
    async fn schema_version_bump_reads_as_miss(pool: SqlitePool) {
        let lyrics = sample_lyrics("t1");
        write(&pool, &lyrics).await.unwrap();

        // bump schema_version in the db to simulate an older payload
        sqlx::query("UPDATE lyrics SET schema_version = 0 WHERE track_id = 't1'")
            .execute(&pool)
            .await
            .unwrap();

        assert!(read(&pool, "t1").await.is_none());
    }

    #[sqlx::test]
    async fn per_source_miss_ttl_behaviour(pool: SqlitePool) {
        assert!(!source_missed(&pool, "t1", "netease").await);

        mark_source_miss(&pool, "t1", "netease").await.unwrap();
        assert!(source_missed(&pool, "t1", "netease").await);

        assert!(!source_missed(&pool, "t1", "qq").await);
        assert!(!source_missed(&pool, "t2", "netease").await);

        let stale_time = now_sec() - (25 * 3600);
        sqlx::query("UPDATE lyrics_miss SET fetched_at = ? WHERE track_id = 't1' AND source = 'netease'")
            .bind(stale_time)
            .execute(&pool)
            .await
            .unwrap();

        assert!(!source_missed(&pool, "t1", "netease").await);
    }

    #[sqlx::test]
    async fn list_alts_returns_metadata(pool: SqlitePool) {
        let mut alt1 = sample_lyrics("t1");
        alt1.source = "musixmatch".to_string();
        alt1.word_level = false;
        alt1.synced = true;

        let mut alt2 = sample_lyrics("t1");
        alt2.source = "netease".to_string();
        alt2.word_level = true;
        alt2.synced = true;
        alt2.lines.push(LyricLine::line(4000, "third line"));

        write_alt(&pool, "t1", &alt1).await.unwrap();
        write_alt(&pool, "t1", &alt2).await.unwrap();

        let alts = list_alts(&pool, "t1").await;
        assert_eq!(alts.len(), 2);

        let mxm = alts.iter().find(|a| a.source == "musixmatch").unwrap();
        assert_eq!(mxm.lines, 2);
        assert!(!mxm.word_level);
        assert!(mxm.synced);

        let ne = alts.iter().find(|a| a.source == "netease").unwrap();
        assert_eq!(ne.lines, 3);
        assert!(ne.word_level);
        assert!(ne.synced);

        let read_ne = read_alt(&pool, "t1", "netease").await.unwrap();
        assert_eq!(read_ne.source, "netease");
        assert_eq!(read_ne.lines.len(), 3);
    }

    #[sqlx::test]
    async fn whole_track_miss_ttl_behaviour(pool: SqlitePool) {
        write_miss(&pool, "t1").await.unwrap();

        let hit = read(&pool, "t1").await.unwrap();
        assert!(!hit.found);
        assert_eq!(hit.source, "none");

        let stale_time = now_sec() - (8 * 86400);
        sqlx::query("UPDATE lyrics SET fetched_at = ? WHERE track_id = 't1'")
            .bind(stale_time)
            .execute(&pool)
            .await
            .unwrap();

        assert!(read(&pool, "t1").await.is_none());
    }
}
