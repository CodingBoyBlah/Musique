import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getLikedSongs,
  getLikedSongsCount,
  getMyPlaylists,
  getSavedAlbums,
  getFollowedArtists,
  getLibraryStatus,
  getSavedTrackIds,
  saveTrack,
  unsaveTrack,
  followArtist,
  unfollowArtist,
  isArtistFollowed,
  syncLibrary,
  getTopTracks,
  getTopArtists,
  getRecentlyPlayed,
  getNewReleases,
} from "../api/library";
import type { TimeRange } from "../types/library";

/* one place for all the library query keys. keeping them here means
useInvalidateLibrary / useSyncLibrary can nuke the whole slice with a single
invalidateQueries({ queryKey: ["library"] }) prefix match. */
export const LIBRARY_KEYS = {
  status:    ["library", "status"] as const,
  liked:     (limit: number, offset: number) => ["library", "liked", limit, offset] as const,
  likedCount: ["library", "liked", "count"] as const,
  playlists: ["library", "playlists"] as const,
  albums:    ["library", "albums"] as const,
  artists:   ["library", "artists"] as const,
  topTracks:  (range: TimeRange) => ["library", "top", "tracks", range] as const,
  topArtists: (range: TimeRange) => ["library", "top", "artists", range] as const,
  recent:      ["library", "recent"] as const,
  newReleases: ["library", "new-releases"] as const,
};

export function useLibraryStatus() {
  return useQuery({
    queryKey: LIBRARY_KEYS.status,
    queryFn:  getLibraryStatus,
  });
}

export function useLikedSongs(limit = 50, offset = 0) {
  return useQuery({
    queryKey: LIBRARY_KEYS.liked(limit, offset),
    queryFn:  () => getLikedSongs(limit, offset),
  });
}

export function useLikedSongsCount() {
  return useQuery({
    queryKey: LIBRARY_KEYS.likedCount,
    queryFn:  getLikedSongsCount,
  });
}

export function useMyPlaylists() {
  return useQuery({
    queryKey: LIBRARY_KEYS.playlists,
    queryFn:  getMyPlaylists,
  });
}

export function useSavedAlbums() {
  return useQuery({
    queryKey: LIBRARY_KEYS.albums,
    queryFn:  getSavedAlbums,
  });
}

export function useFollowedArtists() {
  return useQuery({
    queryKey: LIBRARY_KEYS.artists,
    queryFn:  getFollowedArtists,
  });
}

export function useTopTracks(range: TimeRange = "medium_term") {
  return useQuery({
    queryKey: LIBRARY_KEYS.topTracks(range),
    queryFn:  () => getTopTracks(range),
  });
}

export function useTopArtists(range: TimeRange = "medium_term") {
  return useQuery({
    queryKey: LIBRARY_KEYS.topArtists(range),
    queryFn:  () => getTopArtists(range),
  });
}

export function useRecentlyPlayed() {
  return useQuery({
    queryKey: LIBRARY_KEYS.recent,
    queryFn:  getRecentlyPlayed,
  });
}

export function useNewReleases() {
  return useQuery({
    queryKey: LIBRARY_KEYS.newReleases,
    queryFn:  getNewReleases,
  });
}

// kick off a full library sync, invalidate all library queries on success
export function useSyncLibrary() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: syncLibrary,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["library"] });
    },
  });
}

// stable callback that invalidates every library query (the `library:synced event uses this)
export function useInvalidateLibrary() {
  const qc = useQueryClient();
  return () => qc.invalidateQueries({ queryKey: ["library"] });
}

// liked-state for a set of track ids (local cache, fast + offline)
export function useSavedTrackIds(ids: string[]) {
  return useQuery({
    queryKey: ["library", "saved-ids", [...ids].sort().join(",")],
    queryFn:  () => getSavedTrackIds(ids),
    enabled:  ids.length > 0,
  });
}

// toggle like/unlike, invalidate after
export function useToggleLike() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, liked }: { id: string; liked: boolean }) =>
      liked ? unsaveTrack(id) : saveTrack(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["library", "saved-ids"] });
      qc.invalidateQueries({ queryKey: ["library", "liked"] });
    },
  });
}

// whether the current user follows an artist
export function useIsArtistFollowed(id: string | undefined) {
  return useQuery({
    queryKey: ["library", "follows", id],
    queryFn:  () => isArtistFollowed(id!),
    enabled:  !!id,
  });
}

// toggle follow/unfollow for an artist, invalidate follow + artists queries after
export function useToggleFollow() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, following }: { id: string; following: boolean }) =>
      following ? unfollowArtist(id) : followArtist(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["library", "follows"] });
      qc.invalidateQueries({ queryKey: ["library", "artists"] });
    },
  });
}
