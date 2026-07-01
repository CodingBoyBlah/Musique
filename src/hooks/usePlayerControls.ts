import { useCallback } from "react";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import {
  pausePlayback, resumeOrPlay, seekPlayback, playTrack,
} from "../api/playback";
import { startRadio } from "../utils/radio";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";

// transport actions (play/pause/next/prev/seek) shared by PlayerBar + Immersive so the two dont drift apart. reads live store state on each call so its safe to call from wherever
export function usePlayerControls() {
  const togglePlay = useCallback(() => {
    const s = usePlayerStore.getState();
    if (s.isPlaying) {
      s.setPlaying(false);                         // optimistic
      pausePlayback().catch(() => {});
      return;
    }
    if (!s.currentTrack) return;
    s.setPlaying(true);                            // optimistic
    const clickedAt = Date.now();
    const id = s.currentTrack.id;
    resumeOrPlay(s.currentTrack.id, s.sessionReady ? s.positionMs : 0)
      .then(() => {
        /* watchdog. if no real "playing" event lands within 5s the load
        silently died (dead session / unavailable track). revert the
        optimistic flip + actually tell the user, dont just bounce the button */
        setTimeout(() => {
          const cur = usePlayerStore.getState();
          if (cur.isPlaying && cur.currentId === id && cur.lastPlayingAt < clickedAt) {
            cur.setPlaying(false);
            toast("Playback didn't start. Try again in a moment.");
          }
        }, 5000);
      })
      .catch((e) => {
        s.setPlaying(false);
        toast(errMsg(e));
      });
  }, []);

  const next = useCallback(() => {
    const { currentTrack, setCurrentTrack } = usePlayerStore.getState();
    const n = useQueueStore.getState().advance(currentTrack);
    if (n) { setCurrentTrack(n); playTrack(n.id).catch(() => {}); }
    else if (currentTrack) { startRadio(currentTrack); }  // queue dried up, fall back to radio
  }, []);

  const seek = useCallback((ms: number) => {
    usePlayerStore.getState().setPosition(ms);
    seekPlayback(ms).catch(() => {});
  }, []);

  const prev = useCallback(() => {
    const { currentTrack, positionMs, setCurrentTrack } = usePlayerStore.getState();
    if (positionMs > 3000) {
      seek(0);
    } else {
      const p = useQueueStore.getState().previous(currentTrack);
      if (p) { setCurrentTrack(p); playTrack(p.id).catch(() => {}); }
    }
  }, [seek]);

  return { togglePlay, next, prev, seek };
}
