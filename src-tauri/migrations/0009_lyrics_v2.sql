-- Lyrics v2: cache the PARSED result instead of the provider's raw body.
--
-- v1 kept each provider's body verbatim and re-parsed it on every single read,
-- so every panel open and every prefetch paid for a full LRC/YRC/richsync parse
-- again for lyrics we had already parsed. `payload` holds the serialized
-- `Lyrics` struct, so a cache hit is now one JSON deserialize and nothing else.
--
-- `schema_version` is the escape hatch that makes that safe: the parsers keep
-- improving, and a cached payload is frozen at whatever the parser produced the
-- day it was written. Bumping SCHEMA_VERSION in lyrics/cache.rs makes every
-- older payload read as a miss, so a parser fix reaches users without a
-- migration. It defaults to 0, which is deliberately not the current version -
-- every row written by v1 is therefore stale on sight.
--
-- The v1 columns stay exactly as they were so anything still reading the old
-- shape keeps working. We no longer fill `synced_lrc` (the raw body is the thing
-- we stopped storing); `plain`, `source`, `instrumental` and `found` are still
-- written so the legacy shape stays truthful.
--
-- SQLite takes one column per ALTER and has no IF NOT EXISTS for ADD COLUMN,
-- which is fine because sqlx runs this file exactly once and records it in
-- _sqlx_migrations.
ALTER TABLE lyrics ADD COLUMN payload TEXT;
ALTER TABLE lyrics ADD COLUMN offset_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lyrics ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lyrics ADD COLUMN word_level INTEGER NOT NULL DEFAULT 0;

-- Every OTHER source we hold for the same track, one row each, parsed and ready.
-- This is what powers the UI's source switcher: flipping from the chosen source
-- to musixmatch or netease has to be instant and has to work on a plane, so the
-- losers of the race are kept rather than thrown away. Keyed by (track, source)
-- because a provider only ever has one answer per track.
CREATE TABLE IF NOT EXISTS lyrics_alt (
    track_id   TEXT    NOT NULL,
    source     TEXT    NOT NULL,           -- amll | spotify | musixmatch | netease | qq | kugou | lrclib
    payload    TEXT    NOT NULL,           -- serialized Lyrics, same shape as lyrics.payload
    word_level INTEGER NOT NULL DEFAULT 0, -- carries real per-word timings
    synced     INTEGER NOT NULL DEFAULT 0, -- has timed lines at all
    fetched_at INTEGER NOT NULL,           -- unix seconds
    PRIMARY KEY (track_id, source)
);

CREATE INDEX IF NOT EXISTS idx_lyrics_alt_track ON lyrics_alt (track_id);

-- Per-(track, source) negative cache. The whole-track `found = 0` flag in
-- `lyrics` is too blunt for a race across six providers: it can't tell "nobody
-- has this song" from "netease timed out". Recording the miss against the single
-- provider that missed means one dead provider can never suppress or re-trigger
-- the other five, and a flaky source gets retried on a much shorter leash than a
-- song that genuinely has no lyrics anywhere (TTLs live in lyrics/cache.rs).
CREATE TABLE IF NOT EXISTS lyrics_miss (
    track_id   TEXT    NOT NULL,
    source     TEXT    NOT NULL,
    fetched_at INTEGER NOT NULL, -- unix seconds
    PRIMARY KEY (track_id, source)
);

CREATE INDEX IF NOT EXISTS idx_lyrics_miss_track ON lyrics_miss (track_id);
