-- Performance indexes for accelerating library, playlist, and artist queries
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_pos ON playlist_tracks (playlist_id, position);
CREATE INDEX IF NOT EXISTS idx_track_artists_artist ON track_artists (artist_id);
CREATE INDEX IF NOT EXISTS idx_album_artists_artist ON album_artists (artist_id);
CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks (album_id);
CREATE INDEX IF NOT EXISTS idx_saved_tracks_added ON saved_tracks (added_at DESC);
