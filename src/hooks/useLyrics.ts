import { useQuery } from "@tanstack/react-query";
import { getLyrics, type Lyrics } from "../api/lyrics";
import type { TrackItem } from "../types/spotify";

// lyrics for a track. cached hard (backend also caches in sqlite), so reopening the panel or replaying a song is instant + works offline.
export function useLyrics(track: TrackItem | null) {
  return useQuery<Lyrics>({
    queryKey:  ["lyrics", track?.id],
    queryFn:   () => getLyrics(track!),
    enabled:   !!track,
    staleTime: Infinity,
    gcTime:    60 * 60_000,
    retry:     1,
  });
}
