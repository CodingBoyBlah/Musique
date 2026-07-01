CREATE TABLE IF NOT EXISTS followed_artists (
    artist_id   TEXT    NOT NULL PRIMARY KEY REFERENCES artists(id) ON DELETE CASCADE,
    followed_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_followed_artists_followed_at ON followed_artists (followed_at DESC);

-- Index playlist snapshots for conflict detection
CREATE INDEX IF NOT EXISTS idx_playlists_snapshot ON playlists (snapshot_id);
