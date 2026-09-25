import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { search } from "../api/spotify";

// tanstack-query wrapper for catalog search. stays off while the query is
// blank/whitespace-only. keeps the previous list visible while the next query
// is in flight so the top-bar results never flash empty while typing.
export function useSearch(query: string) {
  return useQuery({
    queryKey: ["search", query],
    queryFn:  () => search(query),
    enabled:  query.trim().length > 0,
    staleTime: 300_000,
    placeholderData: keepPreviousData,
  });
}
