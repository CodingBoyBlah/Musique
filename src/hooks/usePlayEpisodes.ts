import { useCallback } from "react";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack, resumeOrPlay } from "../api/playback";
import { transportTogglePlay } from "./usePlayerControls";
import { episodeToTrack } from "../utils/episode";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import type { EpisodeItem } from "../types/podcast";

/* play episode i of a list, queueing the rest after it. picks up where you
left off (spotify's resume point) unless you'd already finished it. clicking
the episode that's already loaded just toggles play/pause. */
export function usePlayEpisodes(episodes: EpisodeItem[], contextId: string) {
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);

  return useCallback(
    (index: number) => {
      const ep = episodes[index];
      if (!ep) return;
      const tracks = episodes.map(episodeToTrack);
      if (usePlayerStore.getState().currentId === tracks[index].id) {
        transportTogglePlay();
        return;
      }
      const start = playContext(tracks, index, contextId);
      if (!start) return;
      setCurrentTrack(start);
      const pos = !ep.fully_played && ep.resume_position_ms ? ep.resume_position_ms : 0;
      (pos > 0 ? resumeOrPlay(start.id, pos) : playTrack(start.id)).catch((e) =>
        toast.error(`Couldn't play episode: ${errMsg(e)}`),
      );
    },
    [episodes, contextId, playContext, setCurrentTrack],
  );
}
