import { invoke } from "@tauri-apps/api/core";
import type {
  ArtistDetail,
  AlbumDetail,
  TrackDetail,
  PlaylistDetail,
  SearchResults,
  TrackItem,
} from "../types/spotify";

export const search      = (query: string, types = "track,artist,album"): Promise<SearchResults> =>
  invoke("search", { query, types });

export const getArtist   = (id: string): Promise<ArtistDetail>   => invoke("get_artist",   { id });
export const getAlbum    = (id: string): Promise<AlbumDetail>    => invoke("get_album",    { id });
export const getTrack    = (id: string): Promise<TrackDetail>    => invoke("get_track",    { id });
export const getPlaylist = (id: string): Promise<PlaylistDetail> => invoke("get_playlist", { id });
export const getCachedPlaylist = (id: string): Promise<PlaylistDetail | null> => invoke("get_cached_playlist", { id });

export const getRecommendations = (
  seedArtistIds?: string[],
  limit?: number,
  excludeTrackIds?: string[],
): Promise<TrackItem[]> =>
  invoke("get_recommendations", {
    seedArtistIds:   seedArtistIds   ?? null,
    excludeTrackIds: excludeTrackIds ?? null,
    limit:           limit           ?? null,
  });
