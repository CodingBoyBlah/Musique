-- Replace the scaffold table with the full playback_history table
DROP TABLE IF EXISTS play_history;

CREATE TABLE IF NOT EXISTS users (
    id           TEXT    PRIMARY KEY NOT NULL,
    display_name TEXT,
    email        TEXT,
    product      TEXT,
    image_url    TEXT,
    country      TEXT,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS artists (
    id         TEXT    PRIMARY KEY NOT NULL,
    name       TEXT    NOT NULL,
    image_url  TEXT,
    genres     TEXT,       -- JSON array, e.g. '["pop","rock"]'
    popularity INTEGER,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS albums (
    id           TEXT    PRIMARY KEY NOT NULL,
    name         TEXT    NOT NULL,
    album_type   TEXT    NOT NULL DEFAULT 'album',  -- album | single | compilation
    image_url    TEXT,
    release_date TEXT,
    total_tracks INTEGER NOT NULL DEFAULT 0,
    genres       TEXT,       -- JSON array
    popularity   INTEGER,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS album_artists (
    album_id  TEXT    NOT NULL REFERENCES albums(id)  ON DELETE CASCADE,
    artist_id TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    position  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (album_id, artist_id)
);

CREATE TABLE IF NOT EXISTS tracks (
    id           TEXT    PRIMARY KEY NOT NULL,
    name         TEXT    NOT NULL,
    album_id     TEXT    REFERENCES albums(id) ON DELETE SET NULL,
    duration_ms  INTEGER NOT NULL DEFAULT 0,
    track_number INTEGER NOT NULL DEFAULT 0,
    disc_number  INTEGER NOT NULL DEFAULT 1,
    explicit     INTEGER NOT NULL DEFAULT 0,
    popularity   INTEGER,
    preview_url  TEXT,
    is_local     INTEGER NOT NULL DEFAULT 0,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS track_artists (
    track_id  TEXT    NOT NULL REFERENCES tracks(id)  ON DELETE CASCADE,
    artist_id TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    position  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (track_id, artist_id)
);

CREATE TABLE IF NOT EXISTS playlists (
    id           TEXT    PRIMARY KEY NOT NULL,
    name         TEXT    NOT NULL,
    description  TEXT,
    owner_id     TEXT,
    image_url    TEXT,
    total_tracks INTEGER NOT NULL DEFAULT 0,
    is_public    INTEGER NOT NULL DEFAULT 0,
    is_local     INTEGER NOT NULL DEFAULT 0,
    snapshot_id  TEXT,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT    NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    TEXT    NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,
    position    INTEGER NOT NULL DEFAULT 0,
    added_at    INTEGER NOT NULL,
    added_by    TEXT,
    PRIMARY KEY (playlist_id, track_id)
);

CREATE TABLE IF NOT EXISTS saved_tracks (
    track_id TEXT    NOT NULL PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS search_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    query       TEXT    NOT NULL,
    result_type TEXT,       -- track | artist | album | playlist | NULL
    result_id   TEXT,
    searched_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playback_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    track_id     TEXT    NOT NULL,
    context_type TEXT,       -- album | playlist | artist | NULL
    context_id   TEXT,
    played_at    INTEGER NOT NULL,
    duration_ms  INTEGER     -- ms actually played, NULL if unknown
);

CREATE INDEX IF NOT EXISTS idx_playback_history_played_at ON playback_history (played_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_searched_at ON search_history (searched_at DESC);
CREATE INDEX IF NOT EXISTS idx_tracks_album_id            ON tracks (album_id);
CREATE INDEX IF NOT EXISTS idx_saved_tracks_added_at      ON saved_tracks (added_at DESC);
