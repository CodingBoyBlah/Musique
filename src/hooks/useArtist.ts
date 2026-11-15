import { useQueries, useQuery } from "@tanstack/react-query";
import { getArtist } from "../api/spotify";

// single-artist query. off when id is absent. staleTime 5min to match prefetchArtist.
export function useArtist(id: string | undefined) {
  return useQuery({
    queryKey: ["artist", id],
    queryFn:  () => getArtist(id!),
    enabled:  !!id,
    staleTime: 300_000,
  });
}

// several artists at once, in parallel, sharing useArtist's cache entries -
// so the album header's avatars double as a prefetch of each artist page
export function useArtists(ids: string[]) {
  return useQueries({
    queries: ids.map((id) => ({
      queryKey: ["artist", id],
      queryFn:  () => getArtist(id),
      staleTime: 300_000,
    })),
  });
}
