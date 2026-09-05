import { useCallback } from "react";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import {
  pausePlayback, resumeOrPlay, seekPlayback, playTrack,
} from "../api/playback";
import { replenishQueue } from "../utils/radio";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";

let transportInFlight = false;
let pendingTarget: "play" | "pause" | null = null;

async function executeTransport() {
  if (transportInFlight) return;
  transportInFlight = true;

  try {
    while (pendingTarget !== null) {
      const target = pendingTarget;
      pendingTarget = null;
      const s = usePlayerStore.getState();

      if (target === "pause") {
        await pausePlayback().catch((e) => console.error("[transport] pause error:", e));
      } else if (target === "play" && s.currentTrack) {
        const id = s.currentTrack.id;
        const pos = s.sessionReady ? s.positionMs : 0;
        await resumeOrPlay(id, pos).catch((e) => {
          console.error("[transport] play error:", e);
          usePlayerStore.getState().setPlaying(false);
          usePlayerStore.getState().clearTargetState();
          toast(errMsg(e));
        });
      }
    }
  } finally {
    transportInFlight = false;
    if (pendingTarget !== null) {
      executeTransport();
    }
  }
}

export function transportTogglePlay(): void {
  const s = usePlayerStore.getState();
  if (!s.currentTrack) return;

  const targetPlay = !s.isPlaying;
  s.setTargetState(targetPlay ? "playing" : "paused");
  s.setPlaying(targetPlay);

  pendingTarget = targetPlay ? "play" : "pause";
  executeTransport();
}

export function transportPlay(): void {
  const s = usePlayerStore.getState();
  if (!s.currentTrack) return;
  s.setTargetState("playing");
  s.setPlaying(true);
  pendingTarget = "play";
  executeTransport();
}

export function transportPause(): void {
  const s = usePlayerStore.getState();
  s.setTargetState("paused");
  s.setPlaying(false);
  pendingTarget = "pause";
  executeTransport();
}

export function transportNext(): void {
  const { currentTrack, setCurrentTrack } = usePlayerStore.getState();
  const n = useQueueStore.getState().advance(currentTrack);
  if (n) {
    setCurrentTrack(n);
    usePlayerStore.getState().setPlaying(true);
    usePlayerStore.getState().setTargetState("playing");
    playTrack(n.id).catch((e) => console.error("[transport] playTrack error:", e));
    replenishQueue(n).catch(() => {});
  }
}

export function transportPrev(): void {
  const { currentTrack, positionMs, setCurrentTrack } = usePlayerStore.getState();
  if (positionMs > 3000) {
    transportSeek(0);
  } else {
    const p = useQueueStore.getState().previous(currentTrack);
    if (p) {
      setCurrentTrack(p);
      usePlayerStore.getState().setPlaying(true);
      usePlayerStore.getState().setTargetState("playing");
      playTrack(p.id).catch((e) => console.error("[transport] prev error:", e));
    }
  }
}

export function transportSeek(ms: number): void {
  usePlayerStore.getState().setPosition(ms);
  seekPlayback(ms).catch(() => {});
}

// transport actions (play/pause/next/prev/seek) shared across PlayerBar + Immersive.
// Serializes commands so rapid clicking can never cause race conditions or stalled audio.
export function usePlayerControls() {
  const togglePlay = useCallback(() => transportTogglePlay(), []);
  const play = useCallback(() => transportPlay(), []);
  const pause = useCallback(() => transportPause(), []);
  const next = useCallback(() => transportNext(), []);
  const prev = useCallback(() => transportPrev(), []);
  const seek = useCallback((ms: number) => transportSeek(ms), []);

  return { togglePlay, play, pause, next, prev, seek };
}
