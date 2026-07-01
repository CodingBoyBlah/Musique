import { useQuery } from "@tanstack/react-query";
import { getAlbum } from "../api/spotify";

// single-album query. off when id is absent. staleTime 5min to match prefetchAlbum.
export function useAlbum(id: string | undefined) {
  return useQuery({
    queryKey: ["album", id],
    queryFn:  () => getAlbum(id!),
    enabled:  !!id,
    staleTime: 300_000,
  });
}
