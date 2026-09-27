import { useQuery } from "@tanstack/react-query";
import { getPlaylistFolders } from "../api/internal";
import { useAuthStore } from "../store/auth.store";

// the library's folder tree. under ["library"] so a sync refreshes it; a
// failure just means the flat list is used instead
export function usePlaylistFolders() {
  const loggedIn = useAuthStore((s) => s.loggedIn);
  return useQuery({
    queryKey: ["library", "rootlist"],
    queryFn: getPlaylistFolders,
    enabled: loggedIn,
    staleTime: 5 * 60_000,
    retry: false,
  });
}
