import { useQuery } from "@tanstack/react-query";
import { getCanvas } from "../api/internal";
import { usePrefsStore } from "../store/prefs.store";
import { isEpisodeId } from "../utils/episode";

// spotify canvas for the given track, when the setting is on. cached for the
// session - the backend already caches a day, including "no canvas"
export function useCanvas(trackId: string | null | undefined) {
  const enabled = usePrefsStore((s) => s.showCanvas);
  const { data } = useQuery({
    queryKey: ["canvas", trackId],
    queryFn: () => getCanvas(trackId!),
    enabled: enabled && !!trackId && !isEpisodeId(trackId),
    staleTime: Infinity,
    gcTime: 30 * 60_000,
    retry: false,
  });
  return enabled ? data ?? null : null;
}
