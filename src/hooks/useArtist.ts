import { useQuery } from "@tanstack/react-query";
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
