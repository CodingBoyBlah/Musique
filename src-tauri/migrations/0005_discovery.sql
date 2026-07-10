-- Discovery surfaces, cached for instant + offline reads.
-- Each is a flat ordered list; the catalog rows (tracks/artists/albums) they
-- point at are upserted by the sync layer before these rows are written.

-- Top tracks / artists, keyed by Spotify time_range (short|medium|long term).
CREATE TABLE IF NOT EXISTS top_tracks (
    time_range TEXT    NOT NULL,            -- short_term | medium_term | long_term
    track_id   TEXT    NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    PRIMARY KEY (time_range, track_id)
);

CREATE TABLE IF NOT EXISTS top_artists (
    time_range TEXT    NOT NULL,
    artist_id  TEXT    NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    PRIMARY KEY (time_range, artist_id)
);

-- Recently played - a rolling log; (track_id, played_at) is unique per play.
CREATE TABLE IF NOT EXISTS recently_played (
    track_id  TEXT    NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    played_at INTEGER NOT NULL,
    PRIMARY KEY (track_id, played_at)
);

-- New releases - an ordered album list straight from /browse/new-releases.
CREATE TABLE IF NOT EXISTS new_releases (
    album_id TEXT    NOT NULL PRIMARY KEY REFERENCES albums(id) ON DELETE CASCADE,
    position INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_top_tracks_range   ON top_tracks  (time_range, position);
CREATE INDEX IF NOT EXISTS idx_top_artists_range  ON top_artists (time_range, position);
CREATE INDEX IF NOT EXISTS idx_recently_played_at ON recently_played (played_at DESC);
CREATE INDEX IF NOT EXISTS idx_new_releases_pos   ON new_releases (position);
