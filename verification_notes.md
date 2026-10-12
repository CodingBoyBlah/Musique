# Read-only verification

No build, install, application launch, native test, or persistent database operation was run. The commands below used in-memory compiler/SQL/data analysis and read-only file inspection. Only audit Markdown documents were written.

## Final file and Git reconciliation

Completed 2026-10-10. All 540 original inventory entries are checked. Every original file's byte count and SHA-256 match its baseline, including original Markdown, pre-existing user changes and sensitive signing-file bytes. No secret contents were printed. No original path is missing.

HEAD remains 2d55bb477c40a7b27affc35cac64062baf1c7e74. The same eight pre-existing modified source files and untracked src/components/ui/SignInPrompt.css remain. No tracked-file change was added by the audit.

A fresh workspace walk using the same installed/generated/Git exclusions found exactly nineteen new files, all audit Markdown documents. No unexpected new file or missing audit document was found. Installed/generated boundaries were not hashed as original project source; this is not a claim to reconcile all pre-existing build/dependency artifacts.

All nineteen audit documents passed local-link existence and fenced-block closure checks: 24 local document links and twelve Mermaid diagrams. Diagram fences were checked structurally; no graphical Mermaid renderer was launched. The plan has no unchecked items, and the review checklist has 540 checked entries and zero pending entries. A manual event-data-flow check supplements nine literal emits with four constant/table-driven names.

## TypeScript tsconfig.json

TypeScript 5.8.3; 193 root files; 0 diagnostics. `noEmit=true`, `incremental=false`, `composite=false`; referenced project builds were not invoked.

## TypeScript tsconfig.node.json

TypeScript 5.8.3; 1 root files; 2 diagnostics. `noEmit=true`, `incremental=false`, `composite=false`; referenced project builds were not invoked.

- vite.config.ts:4:18 TS2307: Cannot find module 'path' or its corresponding type declarations.
- vite.config.ts:14:25 TS2304: Cannot find name '__dirname'.

## Native migrations and schema in memory

SQLite 3.37.2; all thirteen native migrations applied in order to `:memory:` with foreign keys enabled and temporary storage in memory. This checks SQLite syntax/schema constraints, not SQLx runtime or packaged native SQLite compatibility. No database file was created.

Applied: 0000_init.sql, 0001_catalog.sql, 0002_library.sql, 0003_saved_albums.sql, 0004_lyrics.sql, 0005_discovery.sql, 0006_updates.sql, 0007_perf_indexes.sql, 0008_taste.sql, 0009_lyrics_v2.sql, 0010_track_isrc.sql, 0011_yt_match.sql, 0012_internal_cache.sql.

| Type | Name/table | Actual final SQL |
| --- | --- | --- |
| index | idx_album_artists_artist / album_artists | `CREATE INDEX idx_album_artists_artist ON album_artists (artist_id)` |
| index | idx_extension_cache_fetched / extension_cache | `CREATE INDEX idx_extension_cache_fetched ON extension_cache (fetched_at)` |
| index | idx_followed_artists_followed_at / followed_artists | `CREATE INDEX idx_followed_artists_followed_at ON followed_artists (followed_at DESC)` |
| index | idx_listen_events_at / listen_events | `CREATE INDEX idx_listen_events_at    ON listen_events (occurred_at DESC)` |
| index | idx_listen_events_track / listen_events | `CREATE INDEX idx_listen_events_track ON listen_events (track_id, occurred_at DESC)` |
| index | idx_lyrics_alt_track / lyrics_alt | `CREATE INDEX idx_lyrics_alt_track ON lyrics_alt (track_id)` |
| index | idx_lyrics_miss_track / lyrics_miss | `CREATE INDEX idx_lyrics_miss_track ON lyrics_miss (track_id)` |
| index | idx_new_releases_pos / new_releases | `CREATE INDEX idx_new_releases_pos   ON new_releases (position)` |
| index | idx_playback_history_played_at / playback_history | `CREATE INDEX idx_playback_history_played_at ON playback_history (played_at DESC)` |
| index | idx_playlist_tracks_pos / playlist_tracks | `CREATE INDEX idx_playlist_tracks_pos ON playlist_tracks (playlist_id, position)` |
| index | idx_playlists_snapshot / playlists | `CREATE INDEX idx_playlists_snapshot ON playlists (snapshot_id)` |
| index | idx_recently_played_at / recently_played | `CREATE INDEX idx_recently_played_at ON recently_played (played_at DESC)` |
| index | idx_saved_albums_added_at / saved_albums | `CREATE INDEX idx_saved_albums_added_at ON saved_albums (added_at DESC)` |
| index | idx_saved_tracks_added / saved_tracks | `CREATE INDEX idx_saved_tracks_added ON saved_tracks (added_at DESC)` |
| index | idx_saved_tracks_added_at / saved_tracks | `CREATE INDEX idx_saved_tracks_added_at      ON saved_tracks (added_at DESC)` |
| index | idx_search_history_searched_at / search_history | `CREATE INDEX idx_search_history_searched_at ON search_history (searched_at DESC)` |
| index | idx_top_artists_range / top_artists | `CREATE INDEX idx_top_artists_range  ON top_artists (time_range, position)` |
| index | idx_top_tracks_range / top_tracks | `CREATE INDEX idx_top_tracks_range   ON top_tracks  (time_range, position)` |
| index | idx_track_artists_artist / track_artists | `CREATE INDEX idx_track_artists_artist ON track_artists (artist_id)` |
| index | idx_track_stats_last_played / track_stats | `CREATE INDEX idx_track_stats_last_played  ON track_stats (last_played_at DESC)` |
| index | idx_track_stats_last_skipped / track_stats | `CREATE INDEX idx_track_stats_last_skipped ON track_stats (last_skipped_at DESC)` |
| index | idx_tracks_album_id / tracks | `CREATE INDEX idx_tracks_album_id            ON tracks (album_id)` |
| index | idx_yt_track_match_checked / yt_track_match | `CREATE INDEX idx_yt_track_match_checked     ON yt_track_match (checked_at)     WHERE video_id IS NULL` |
| table | album_artists / album_artists | `CREATE TABLE album_artists (     album_id  TEXT    NOT NULL REFERENCES albums(id)  ON DELETE CASCADE,     artist_id TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,     position  INTEGER NOT NULL DEFAULT 0,     PRIMARY KEY (album_id, artist_id) )` |
| table | albums / albums | `CREATE TABLE albums (     id           TEXT    PRIMARY KEY NOT NULL,     name         TEXT    NOT NULL,     album_type   TEXT    NOT NULL DEFAULT 'album',  -- album \| single \| compilation     image_url    TEXT,     release_date TEXT,     total_tracks INTEGER NOT NULL DEFAULT 0,     genres       TEXT,       -- JSON array     popularity   INTEGER,     updated_at   INTEGER NOT NULL )` |
| table | artists / artists | `CREATE TABLE artists (     id         TEXT    PRIMARY KEY NOT NULL,     name       TEXT    NOT NULL,     image_url  TEXT,     genres     TEXT,       -- JSON array, e.g. '["pop","rock"]'     popularity INTEGER,     updated_at INTEGER NOT NULL )` |
| table | extension_cache / extension_cache | `CREATE TABLE extension_cache (     entity_uri TEXT    NOT NULL,     kind       TEXT    NOT NULL,     payload    BLOB    NOT NULL,     fetched_at INTEGER NOT NULL,     PRIMARY KEY (entity_uri, kind) )` |
| table | followed_artists / followed_artists | `CREATE TABLE followed_artists (     artist_id   TEXT    NOT NULL PRIMARY KEY REFERENCES artists(id) ON DELETE CASCADE,     followed_at INTEGER NOT NULL )` |
| table | listen_events / listen_events | `CREATE TABLE listen_events (     id                INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,     track_id          TEXT    NOT NULL,     event_type        TEXT    NOT NULL,          -- play \| complete \| skip     ms_played         INTEGER NOT NULL DEFAULT 0,     track_duration_ms INTEGER NOT NULL DEFAULT 0,     context_type      TEXT,                       -- album \| playlist \| radio \| ...     context_id        TEXT,     occurred_at       INTEGER NOT NULL )` |
| table | lyrics / lyrics | `CREATE TABLE lyrics (     track_id     TEXT PRIMARY KEY,     synced_lrc   TEXT,                       -- raw LRC ([mm:ss.xx] lines) or NULL     plain        TEXT,                       -- plain lyrics or NULL     source       TEXT    NOT NULL,           -- 'lrclib' \| 'none'     instrumental INTEGER NOT NULL DEFAULT 0, -- provider flagged it instrumental     found        INTEGER NOT NULL DEFAULT 0, -- 1 = real lyrics cached     fetched_at   INTEGER NOT NULL            -- unix seconds , payload TEXT, offset_ms INTEGER NOT NULL DEFAULT 0, schema_version INTEGER NOT NULL DEFAULT 0, word_level INTEGER NOT NULL DEFAULT 0)` |
| table | lyrics_alt / lyrics_alt | `CREATE TABLE lyrics_alt (     track_id   TEXT    NOT NULL,     source     TEXT    NOT NULL,           -- amll \| spotify \| musixmatch \| netease \| qq \| kugou \| lrclib     payload    TEXT    NOT NULL,           -- serialized Lyrics, same shape as lyrics.payload     word_level INTEGER NOT NULL DEFAULT 0, -- carries real per-word timings     synced     INTEGER NOT NULL DEFAULT 0, -- has timed lines at all     fetched_at INTEGER NOT NULL,           -- unix seconds     PRIMARY KEY (track_id, source) )` |
| table | lyrics_miss / lyrics_miss | `CREATE TABLE lyrics_miss (     track_id   TEXT    NOT NULL,     source     TEXT    NOT NULL,     fetched_at INTEGER NOT NULL, -- unix seconds     PRIMARY KEY (track_id, source) )` |
| table | new_releases / new_releases | `CREATE TABLE new_releases (     album_id TEXT    NOT NULL PRIMARY KEY REFERENCES albums(id) ON DELETE CASCADE,     position INTEGER NOT NULL )` |
| table | playback_history / playback_history | `CREATE TABLE playback_history (     id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,     track_id     TEXT    NOT NULL,     context_type TEXT,       -- album \| playlist \| artist \| NULL     context_id   TEXT,     played_at    INTEGER NOT NULL,     duration_ms  INTEGER     -- ms actually played, NULL if unknown )` |
| table | playlist_tracks / playlist_tracks | `CREATE TABLE playlist_tracks (     playlist_id TEXT    NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,     track_id    TEXT    NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,     position    INTEGER NOT NULL DEFAULT 0,     added_at    INTEGER NOT NULL,     added_by    TEXT,     PRIMARY KEY (playlist_id, track_id) )` |
| table | playlists / playlists | `CREATE TABLE playlists (     id           TEXT    PRIMARY KEY NOT NULL,     name         TEXT    NOT NULL,     description  TEXT,     owner_id     TEXT,     image_url    TEXT,     total_tracks INTEGER NOT NULL DEFAULT 0,     is_public    INTEGER NOT NULL DEFAULT 0,     is_local     INTEGER NOT NULL DEFAULT 0,     snapshot_id  TEXT,     updated_at   INTEGER NOT NULL )` |
| table | recently_played / recently_played | `CREATE TABLE recently_played (     track_id  TEXT    NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,     played_at INTEGER NOT NULL,     PRIMARY KEY (track_id, played_at) )` |
| table | saved_albums / saved_albums | `CREATE TABLE saved_albums (     album_id TEXT    NOT NULL PRIMARY KEY REFERENCES albums(id) ON DELETE CASCADE,     added_at INTEGER NOT NULL )` |
| table | saved_tracks / saved_tracks | `CREATE TABLE saved_tracks (     track_id TEXT    NOT NULL PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,     added_at INTEGER NOT NULL )` |
| table | search_history / search_history | `CREATE TABLE search_history (     id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,     query       TEXT    NOT NULL,     result_type TEXT,       -- track \| artist \| album \| playlist \| NULL     result_id   TEXT,     searched_at INTEGER NOT NULL )` |
| table | settings / settings | `CREATE TABLE settings (     key        TEXT PRIMARY KEY NOT NULL,     value      TEXT NOT NULL,     updated_at INTEGER NOT NULL )` |
| table | sqlite_sequence / sqlite_sequence | `CREATE TABLE sqlite_sequence(name,seq)` |
| table | top_artists / top_artists | `CREATE TABLE top_artists (     time_range TEXT    NOT NULL,     artist_id  TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,     position   INTEGER NOT NULL,     PRIMARY KEY (time_range, artist_id) )` |
| table | top_tracks / top_tracks | `CREATE TABLE top_tracks (     time_range TEXT    NOT NULL,            -- short_term \| medium_term \| long_term     track_id   TEXT    NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,     position   INTEGER NOT NULL,     PRIMARY KEY (time_range, track_id) )` |
| table | track_artists / track_artists | `CREATE TABLE track_artists (     track_id  TEXT    NOT NULL REFERENCES tracks(id)  ON DELETE CASCADE,     artist_id TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,     position  INTEGER NOT NULL DEFAULT 0,     PRIMARY KEY (track_id, artist_id) )` |
| table | track_stats / track_stats | `CREATE TABLE track_stats (     track_id          TEXT    PRIMARY KEY NOT NULL,     play_count        INTEGER NOT NULL DEFAULT 0,     complete_count    INTEGER NOT NULL DEFAULT 0,     skip_count        INTEGER NOT NULL DEFAULT 0,     early_skip_count  INTEGER NOT NULL DEFAULT 0,     last_played_at    INTEGER,     last_skipped_at   INTEGER,     last_completed_at INTEGER,     updated_at        INTEGER NOT NULL )` |
| table | tracks / tracks | `CREATE TABLE tracks (     id           TEXT    PRIMARY KEY NOT NULL,     name         TEXT    NOT NULL,     album_id     TEXT    REFERENCES albums(id) ON DELETE SET NULL,     duration_ms  INTEGER NOT NULL DEFAULT 0,     track_number INTEGER NOT NULL DEFAULT 0,     disc_number  INTEGER NOT NULL DEFAULT 1,     explicit     INTEGER NOT NULL DEFAULT 0,     popularity   INTEGER,     preview_url  TEXT,     is_local     INTEGER NOT NULL DEFAULT 0,     updated_at   INTEGER NOT NULL , isrc TEXT)` |
| table | users / users | `CREATE TABLE users (     id           TEXT    PRIMARY KEY NOT NULL,     display_name TEXT,     email        TEXT,     product      TEXT,     image_url    TEXT,     country      TEXT,     updated_at   INTEGER NOT NULL )` |
| table | yt_track_match / yt_track_match | `CREATE TABLE yt_track_match (     -- Spotify track id.     track_id   TEXT PRIMARY KEY,     -- YouTube video id, or NULL for a cached "no acceptable match".     video_id   TEXT,     -- Match score at the time of acceptance (NULL for negative results).     score      REAL,     -- Human-readable justification from the matcher.     reason     TEXT,     -- Duration of the matched YouTube track, for drift detection.     duration_ms INTEGER,     -- Set when a user overrides the automatic match. Pinned rows are never     -- re-resolved or evicted by TTL.     pinned     INTEGER NOT NULL DEFAULT 0,     -- Unix seconds of the last resolution attempt.     checked_at INTEGER NOT NULL )` |

## PNPM edge reconciliation

All 285 resolved dependency/optional-dependency snapshot edges point to existing snapshot identities. Unresolved edges: 0. Lock importer specifiers match package.json. This does not check the installation against disk or current advisories.

## Cache occurrence and query-plan verification

Using the actual migrated schema, inserting the same track at positions 0 and 1 with the repository `INSERT OR IGNORE` shape leaves [('repeated-track', 0)]. This independently confirms that the composite primary key loses repeated playlist occurrences. It does not invoke the Rust repository or Spotify.

The following are actual query shapes with fixture/empty data, not timing benchmarks. Native bundled SQLite versions/statistics may choose different plans.

### Liked songs actual join/order shape

`SELECT t.id,t.name,t.duration_ms,t.explicit,t.popularity,t.preview_url,t.album_id,al.name,al.album_type,al.image_url,al.release_date FROM saved_tracks st JOIN tracks t ON t.id=st.track_id LEFT JOIN albums al ON al.id=t.album_id ORDER BY st.added_at DESC LIMIT ? OFFSET ?`

- SCAN st USING INDEX idx_saved_tracks_added
- SEARCH t USING INDEX sqlite_autoindex_tracks_1 (id=?)
- SEARCH al USING INDEX sqlite_autoindex_albums_1 (id=?)

### Cached playlist order

`SELECT t.id FROM playlist_tracks pt JOIN tracks t ON t.id=pt.track_id WHERE pt.playlist_id=? ORDER BY pt.position`

- SEARCH pt USING INDEX idx_playlist_tracks_pos (playlist_id=?)
- SEARCH t USING COVERING INDEX sqlite_autoindex_tracks_1 (id=?)

### Album track order

`SELECT * FROM tracks WHERE album_id=? ORDER BY disc_number,track_number`

- SEARCH tracks USING INDEX idx_tracks_album_id (album_id=?)
- USE TEMP B-TREE FOR ORDER BY

### Offline recommendation random order

`SELECT t.id FROM tracks t LEFT JOIN track_stats ts ON ts.track_id=t.id WHERE ts.last_played_at IS NULL OR ts.last_played_at < ? ORDER BY RANDOM() LIMIT ?`

- SCAN t USING COVERING INDEX sqlite_autoindex_tracks_1
- SEARCH ts USING INDEX sqlite_autoindex_track_stats_1 (track_id=?)
- USE TEMP B-TREE FOR ORDER BY

