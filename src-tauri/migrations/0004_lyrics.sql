-- Lyrics cache. One row per Spotify track id.
--
-- We store the raw LRC (synced) + plain text exactly as the provider returns
-- them and parse on read, so re-parsing logic can evolve without a migration.
-- `found = 0` is a negative cache (provider had nothing) so we don't hammer the
-- network on every replay; `fetched_at` lets us expire negatives and retry.
CREATE TABLE IF NOT EXISTS lyrics (
    track_id     TEXT PRIMARY KEY,
    synced_lrc   TEXT,                       -- raw LRC ([mm:ss.xx] lines) or NULL
    plain        TEXT,                       -- plain lyrics or NULL
    source       TEXT    NOT NULL,           -- 'lrclib' | 'none'
    instrumental INTEGER NOT NULL DEFAULT 0, -- provider flagged it instrumental
    found        INTEGER NOT NULL DEFAULT 0, -- 1 = real lyrics cached
    fetched_at   INTEGER NOT NULL            -- unix seconds
);
