import { useQuery } from "@tanstack/react-query";
import { search } from "../api/spotify";

// tanstack-query wrapper for catalog search. stays off while the query is
// blank/whitespace-only.
export function useSearch(query: string) {
  return useQuery({
    queryKey: ["search", query],
    queryFn:  () => search(query),
    enabled:  query.trim().length > 0,
    staleTime: 30_000,
  });
}
