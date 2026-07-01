export interface ArtistItem {
  id:          string;
  name:        string;
  image_url:   string | null;
  popularity?: number | null;
}

export interface AlbumItem {
  id:           string;
  name:         string;
  album_type:   string;
  image_url:    string | null;
  release_date: string | null;
  artists:      ArtistItem[];
  popularity?:  number | null;
}

export interface TrackItem {
  id:          string;
  name:        string;
  duration_ms: number;
  explicit:    boolean;
  artists:     ArtistItem[];
  album:       AlbumItem | null;
  popularity?: number | null;
}

export interface PlaylistCard {
  id:          string;
  name:        string;
  description: string | null;
  image_url:   string | null;
  owner_name:  string | null;
}

export interface SearchResults {
  tracks:    TrackItem[];
  artists:   ArtistItem[];
  albums:    AlbumItem[];
  playlists: PlaylistCard[];
}

export interface Profile {
  id:                      string | null;
  display_name:            string | null;
  email:                   string | null;
  country:                 string | null;
  product:                 string | null;
  followers:               number;
  image_url:               string | null;
  spotify_url:             string | null;
  explicit_filter_enabled: boolean;
  explicit_filter_locked:  boolean;
}

export interface ArtistDetail {
  id:         string;
  name:       string;
  image_url:  string | null;
  genres:     string[];
  popularity: number | null;
  albums:     AlbumItem[];
  singles:    AlbumItem[];
  top_tracks: TrackItem[];
}

export interface AlbumDetail {
  id:           string;
  name:         string;
  album_type:   string;
  image_url:    string | null;
  release_date: string | null;
  total_tracks: number;
  popularity:   number | null;
  artists:      ArtistItem[];
  tracks:       TrackItem[];
}

export interface TrackDetail {
  id:          string;
  name:        string;
  duration_ms: number;
  explicit:    boolean;
  popularity:  number | null;
  preview_url: string | null;
  artists:     ArtistItem[];
  album:       AlbumItem | null;
}

export interface PlaylistDetail {
  id:           string;
  name:         string;
  description:  string | null;
  image_url:    string | null;
  owner_name:   string | null;
  total_tracks: number;
  tracks:       TrackItem[];
}
