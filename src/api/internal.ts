import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

/* wrappers for the internal data layer (spclient / pathfinder, see
src-tauri/src/internal). everything here is best-effort on the rust side: a
moved endpoint degrades one feature, it never takes a page down. */

// bare track ids or uris -> full rows (order kept, unknown ids dropped)
export const getTracksMetadata = (ids: string[]): Promise<TrackItem[]> =>
  invoke("get_tracks_metadata", { ids });

export interface Canvas {
  url: string;
  kind: "video" | "image" | "gif";
  artist_name: string | null;
  artist_avatar: string | null;
}

// the looping visual an artist attached to a track, or null (most tracks)
export const getCanvas = (trackId: string): Promise<Canvas | null> =>
  invoke("get_canvas", { trackId });

export interface CreditPerson {
  name: string;
  artist_id: string | null;
  image_url: string | null;
  roles: string[];
}

export interface TrackCredits {
  track_name: string | null;
  sections: { title: string; people: CreditPerson[] }[];
  sources: string[];
}

export const getTrackCredits = (trackId: string): Promise<TrackCredits> =>
  invoke("get_track_credits", { trackId });

export type RootItem =
  | { kind: "playlist"; id: string; name: string | null; image_url: string | null; length: number | null }
  | { kind: "folder"; id: string; name: string; children: RootItem[] };

// your playlists as spotify arranges them - folders, nesting, order
export const getPlaylistFolders = (): Promise<RootItem[]> => invoke("get_playlist_folders");

export interface RadioResult {
  title: string | null;
  // backing playlist of a track radio, openable as a page
  playlist_id: string | null;
  tracks: TrackItem[];
}

// spotify's radio for a track ("inspired by" mix, apollo station fallback)
export const getTrackRadio = (trackId: string): Promise<RadioResult> =>
  invoke("get_track_radio", { trackId });

// a station from any artist / album / playlist / track uri
export const getStation = (seed: string): Promise<RadioResult> =>
  invoke("get_station", { seed });

// what spotify would autoplay next after this context
export const getAutoplayTracks = (contextUri: string, recentTrackIds: string[]): Promise<TrackItem[]> =>
  invoke("get_autoplay_tracks", { contextUri, recentTrackIds });

export interface ArtistExtras {
  biography: string | null;
  gallery: string[];
  active_years: string | null;
  related: import("../types/spotify").ArtistItem[];
  appears_on: import("../types/spotify").AlbumItem[];
  compilations: import("../types/spotify").AlbumItem[];
}

// bio, related artists, gallery, appears-on (ARTIST_V4)
export const getArtistExtras = (id: string): Promise<ArtistExtras> =>
  invoke("get_artist_extras", { id });
