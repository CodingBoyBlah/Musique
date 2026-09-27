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
