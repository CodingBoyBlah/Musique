import { useCallback } from "react";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import {
  pausePlayback, resumeOrPlay, seekPlayback, playTrack, preloadTrack,
} from "../api/playback";
import {
  remotePlay, remotePause, remoteNext, remotePrevious, remoteSeek, getPlaybackState,
} from "../api/connect";
import { replenishQueue } from "../utils/radio";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";

let transportInFlight = false;
let pendingTarget: "play" | "pause" | null = null;
let playDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let lastTargetTrackId: string | null = null;

export function safePlayTrack(id: string, debounceMs = 40): void {
  usePlayerStore.getState().setIsRemotePlayback(false);
  lastTargetTrackId = id;
  if (playDebounceTimer) clearTimeout(playDebounceTimer);
  playDebounceTimer = setTimeout(() => {
    if (lastTargetTrackId === id) {
      playTrack(id).catch((e) => console.error("[transport] safePlayTrack error:", e));
    }
  }, debounceMs);
}

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
  if (s.isRemotePlayback) {
    const targetPlay = !s.isPlaying;
    s.setPlaying(targetPlay);
    if (targetPlay) {
      remotePlay()
        .then(() => {
          setTimeout(async () => {
            const st = await getPlaybackState().catch(() => null);
            if (st) usePlayerStore.getState().syncRemotePlayback(st);
          }, 300);
        })
        .catch((e) => {
          console.error("[transport] remote play error, falling back to local playback:", e);
          const store = usePlayerStore.getState();
          store.setIsRemotePlayback(false);
          if (store.currentTrack) {
            resumeOrPlay(store.currentTrack.id, store.positionMs).catch(() => {});
          } else {
            store.setPlaying(false);
            toast("Remote device unavailable");
          }
        });
    } else {
      remotePause()
        .then(() => {
          setTimeout(async () => {
            const st = await getPlaybackState().catch(() => null);
            if (st) usePlayerStore.getState().syncRemotePlayback(st);
          }, 300);
        })
        .catch((e) => {
          console.error("[transport] remote pause error:", e);
          usePlayerStore.getState().setPlaying(true);
          toast("Unable to control remote device");
        });
    }
    return;
  }
  if (!s.currentTrack) return;

  const targetPlay = !s.isPlaying;
  s.setTargetState(targetPlay ? "playing" : "paused");
  s.setPlaying(targetPlay);

  pendingTarget = targetPlay ? "play" : "pause";
  executeTransport();
}

export function transportPlay(): void {
  const s = usePlayerStore.getState();
  if (s.isRemotePlayback) {
    s.setPlaying(true);
    remotePlay()
      .then(() => {
        setTimeout(async () => {
          const st = await getPlaybackState().catch(() => null);
          if (st) usePlayerStore.getState().syncRemotePlayback(st);
        }, 300);
      })
      .catch((e) => {
        console.error("[transport] remote play error, falling back to local playback:", e);
        const store = usePlayerStore.getState();
        store.setIsRemotePlayback(false);
        if (store.currentTrack) {
          resumeOrPlay(store.currentTrack.id, store.positionMs).catch(() => {});
        } else {
          store.setPlaying(false);
          toast("Remote device unavailable");
        }
      });
    return;
  }
  if (!s.currentTrack) return;
  s.setTargetState("playing");
  s.setPlaying(true);
  pendingTarget = "play";
  executeTransport();
}

export function transportPause(): void {
  const s = usePlayerStore.getState();
  if (s.isRemotePlayback) {
    s.setPlaying(false);
    remotePause()
      .then(() => {
        setTimeout(async () => {
          const st = await getPlaybackState().catch(() => null);
          if (st) usePlayerStore.getState().syncRemotePlayback(st);
        }, 300);
      })
      .catch((e) => {
        console.error("[transport] remote pause error:", e);
        usePlayerStore.getState().setPlaying(true);
        toast("Unable to pause remote device");
      });
    return;
  }
  s.setTargetState("paused");
  s.setPlaying(false);
  pendingTarget = "pause";
  executeTransport();
}

export function transportNext(): void {
  const s = usePlayerStore.getState();
  if (s.isRemotePlayback) {
    remoteNext()
      .then(() => {
        setTimeout(async () => {
          const st = await getPlaybackState().catch(() => null);
          if (st) usePlayerStore.getState().syncRemotePlayback(st);
        }, 350);
      })
      .catch((e) => {
        console.error("[transport] remote next error:", e);
        toast("Unable to skip on remote device");
      });
    return;
  }
  const { currentTrack, setCurrentTrack } = s;
  const n = useQueueStore.getState().advance(currentTrack);
  if (n) {
    setCurrentTrack(n);
    usePlayerStore.getState().setPlaying(true);
    usePlayerStore.getState().setTargetState("playing");
    safePlayTrack(n.id);
    replenishQueue(n).catch(() => {});
    const upcoming = useQueueStore.getState().peek(n);
    if (upcoming) {
      setTimeout(() => preloadTrack(upcoming.id).catch(() => {}), 1500);
    }
  }
}

export function transportPrev(): void {
  const s = usePlayerStore.getState();
  if (s.isRemotePlayback) {
    if (s.positionMs > 3000) {
      transportSeek(0);
    } else {
      remotePrevious()
        .then(() => {
          setTimeout(async () => {
            const st = await getPlaybackState().catch(() => null);
            if (st) usePlayerStore.getState().syncRemotePlayback(st);
          }, 350);
        })
        .catch((e) => {
          console.error("[transport] remote prev error:", e);
          toast("Unable to skip on remote device");
        });
    }
    return;
  }
  const { currentTrack, positionMs, setCurrentTrack } = s;
  if (positionMs > 3000) {
    transportSeek(0);
  } else {
    const p = useQueueStore.getState().previous(currentTrack);
    if (p) {
      setCurrentTrack(p);
      usePlayerStore.getState().setPlaying(true);
      usePlayerStore.getState().setTargetState("playing");
      safePlayTrack(p.id);
      const upcoming = useQueueStore.getState().peek(p);
      if (upcoming) {
        setTimeout(() => preloadTrack(upcoming.id).catch(() => {}), 1500);
      }
    }
  }
}

export function transportSeek(ms: number): void {
  const s = usePlayerStore.getState();
  s.setPosition(ms);
  if (s.isRemotePlayback) {
    remoteSeek(ms)
      .then(() => {
        setTimeout(async () => {
          const st = await getPlaybackState().catch(() => null);
          if (st) usePlayerStore.getState().syncRemotePlayback(st);
        }, 400);
      })
      .catch((e) => {
        console.error("[transport] remote seek error:", e);
        toast("Unable to seek on remote device");
      });
  } else {
    seekPlayback(ms).catch(() => {});
  }
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
