import { useQuery } from "@tanstack/react-query";
import { getPlaylist, getCachedPlaylist } from "../api/spotify";

/* cache-first playlist load. the cached copy (from sqlite) paints instantly and
works offline- the live query revalidates in the background and only does
real work when the playlits snapshot changed (rust short-circuits when it
hasnt). upshot: opening a playlist no longer refetches every launch. */
export function usePlaylist(id: string | undefined) {
  const cached = useQuery({
    queryKey:  ["playlist", id, "cached"],
    queryFn:   () => getCachedPlaylist(id!),
    enabled:   !!id,
    staleTime: Infinity,
  });

  const fresh = useQuery({
    queryKey:        ["playlist", id],
    queryFn:         () => getPlaylist(id!),
    enabled:         !!id,
    staleTime:       120_000,
    placeholderData: cached.data ?? undefined,
  });

  return {
    ...fresh,
    data:      fresh.data ?? cached.data ?? undefined,
    // only spin when we've got literally nothing to render yet
    isLoading: fresh.isLoading && !cached.data,
  };
}
