-- Spotify track -> YouTube Music video mappings for free-account playback.
--
-- Cached for three reasons, in order of importance:
--   1. Stability. A search re-run months later can rank differently, so a
--      track that played correctly once could silently start resolving to a
--      different recording. Pinning the mapping removes that entire class of
--      bug.
--   2. Auditability. `score` and `reason` record WHY a match was accepted, so
--      a bad match can be diagnosed without re-running the search.
--   3. Latency. Skips a search round trip on replay.
--
-- Negative results are cached too (video_id NULL). "No acceptable match" is a
-- real, correct outcome and re-searching for it on every play is wasted work -
-- but it is given a TTL by `checked_at` so a track that later appears on
-- YouTube Music is eventually picked up.
CREATE TABLE IF NOT EXISTS yt_track_match (
    -- Spotify track id.
    track_id   TEXT PRIMARY KEY,
    -- YouTube video id, or NULL for a cached "no acceptable match".
    video_id   TEXT,
    -- Match score at the time of acceptance (NULL for negative results).
    score      REAL,
    -- Human-readable justification from the matcher.
    reason     TEXT,
    -- Duration of the matched YouTube track, for drift detection.
    duration_ms INTEGER,
    -- Set when a user overrides the automatic match. Pinned rows are never
    -- re-resolved or evicted by TTL.
    pinned     INTEGER NOT NULL DEFAULT 0,
    -- Unix seconds of the last resolution attempt.
    checked_at INTEGER NOT NULL
);

-- Negative-result sweeps and TTL expiry scan on this.
CREATE INDEX IF NOT EXISTS idx_yt_track_match_checked
    ON yt_track_match (checked_at)
    WHERE video_id IS NULL;
