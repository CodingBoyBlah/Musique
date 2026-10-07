import { isEpisodeId } from "../utils/episode";
import { useEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getLyrics, type Lyrics } from "../api/lyrics";
import { subscribeLyricsUpgrades } from "../lib/lyricsUpgrades";
import type { TrackItem } from "../types/spotify";

// lyrics for a track. cached hard (backend also caches in sqlite), so reopening the panel or replaying a song is instant + works offline.
export function useLyrics(track: TrackItem | null) {
  const [debouncedTrack, setDebouncedTrack] = useState<TrackItem | null>(track);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedTrack(track);
    }, 280);
    return () => clearTimeout(timer);
  }, [track?.id]);

  const queryClient = useQueryClient();
  const trackId = debouncedTrack?.id;

  const { data, isLoading, isError, isFetching, refetch } = useQuery<Lyrics>({
    queryKey:  ["lyrics", trackId],
    queryFn:   () => getLyrics(debouncedTrack!),
    enabled:   !!debouncedTrack && !isEpisodeId(debouncedTrack.id),
    staleTime: Infinity,
    gcTime:    60 * 60_000,
    retry:     1,
  });

  /* read inside the listener rather than re-subscribing per track: the event
     carries its own track_id, so one listener for the hook's whole life is
     enough and track changes never tear down the tauri subscription. */
  const trackIdRef = useRef(trackId);
  trackIdRef.current = trackId;

  /* the word-by-word upgrade.
     the backend answers with line-level lyrics straight away and keeps hunting
     for a word-level source behind it; when one passes the sync check it lands
     here. written into the cache in place - refetching would drop the query
     back through its loading state and flash the panel, and the whole point is
     that the reader never sees the swap happen.
     Coalesced: multiple mounted surfaces (e.g. lyrics panel + immersive view)
     share a single Tauri listener instead of duplicating IPC registrations. */
  useEffect(() => subscribeLyricsUpgrades(queryClient, () => trackIdRef.current), [queryClient]);

  return { data, isLoading, isError, isFetching, refetch };
}
