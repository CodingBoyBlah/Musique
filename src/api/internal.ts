import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

/* wrappers for the internal data layer (spclient / pathfinder, see
src-tauri/src/internal). everything here is best-effort on the rust side: a
moved endpoint degrades one feature, it never takes a page down. */

// bare track ids or uris -> full rows (order kept, unknown ids dropped)
export const getTracksMetadata = (ids: string[]): Promise<TrackItem[]> =>
  invoke("get_tracks_metadata", { ids });
