-- Taste engine: ground-truth listening signals that make recommendations
-- self-improving. Everything here is derived from local playback (no Spotify
-- endpoint needed), so it works offline and survives the deprecation of
-- /recommendations, /audio-features and /related-artists.

-- Raw high-signal events. play = track started, complete = ran to the end,
-- skip = user moved off it early. ms_played lets us weigh an early skip
-- (hate it) against a near-full skip (just wanted something else).
CREATE TABLE IF NOT EXISTS listen_events (
    id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    track_id          TEXT    NOT NULL,
    event_type        TEXT    NOT NULL,          -- play | complete | skip
    ms_played         INTEGER NOT NULL DEFAULT 0,
    track_duration_ms INTEGER NOT NULL DEFAULT 0,
    context_type      TEXT,                       -- album | playlist | radio | ...
    context_id        TEXT,
    occurred_at       INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_listen_events_at    ON listen_events (occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_listen_events_track ON listen_events (track_id, occurred_at DESC);

-- Per-track aggregate, updated on every event so ranking never scans the log.
CREATE TABLE IF NOT EXISTS track_stats (
    track_id          TEXT    PRIMARY KEY NOT NULL,
    play_count        INTEGER NOT NULL DEFAULT 0,
    complete_count    INTEGER NOT NULL DEFAULT 0,
    skip_count        INTEGER NOT NULL DEFAULT 0,
    early_skip_count  INTEGER NOT NULL DEFAULT 0,
    last_played_at    INTEGER,
    last_skipped_at   INTEGER,
    last_completed_at INTEGER,
    updated_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_track_stats_last_played  ON track_stats (last_played_at DESC);
CREATE INDEX IF NOT EXISTS idx_track_stats_last_skipped ON track_stats (last_skipped_at DESC);
