export interface PlaylistSummary {
  id: string;
  name: string;
  description: string | null;
  image_url: string | null;
  total_tracks: number;
  snapshot_id: string | null;
}

export interface LibraryStatus {
  last_synced: number | null;
  is_syncing: boolean;
}

export interface SyncResult {
  liked_count: number;
  playlist_count: number;
  artist_count: number;
  album_count: number;
  top_track_count: number;
  top_artist_count: number;
  recent_count: number;
  new_release_count: number;
}

export type TimeRange = "short_term" | "medium_term" | "long_term";
