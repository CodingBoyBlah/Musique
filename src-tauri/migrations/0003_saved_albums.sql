CREATE TABLE IF NOT EXISTS saved_albums (
    album_id TEXT    NOT NULL PRIMARY KEY REFERENCES albums(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_saved_albums_added_at ON saved_albums (added_at DESC);
