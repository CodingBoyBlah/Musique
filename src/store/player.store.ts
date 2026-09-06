import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { TrackItem } from "../types/spotify";

interface PlayerStore {
  queueOpen:    boolean;
  toggleQueue:  () => void;

  lyricsOpen:   boolean;
  toggleLyrics: () => void;
  setLyricsOpen: (open: boolean) => void;

  // full-screen immersive now-playing view
  immersiveOpen: boolean;
  setImmersiveOpen: (open: boolean) => void;
  toggleImmersive: () => void;
  // which panel the immersive view shows on its right side
  immersivePanel: "lyrics" | "queue";
  setImmersivePanel: (p: "lyrics" | "queue") => void;

  // manual sync nudge for lyrics, in ms. negative = highlight later (the common
  // case - reported playback position runs ahead of what you actually hear
  // because of the audio output buffer, so LRC lines light up early). persisted.
  lyricsOffsetMs: number;
  adjustLyricsOffset: (deltaMs: number) => void;
  setLyricsOffset: (ms: number) => void;

  isPlaying:    boolean;
  sessionReady: boolean;  // true once we've gotten any player event
  // wall-clock ms of the last real "playing" event from librespot. the play
  // watchdog checks against this to catch a silent failed load (UI says playing
  // but no audio) and revert the optimistic state.
  lastPlayingAt: number;
  currentId:    string | null;
  currentTrack: TrackItem | null;
  positionMs:   number;
  durationMs:   number;

  volume:    number;   // 0–100
  muted:     boolean;

  targetState:     "playing" | "paused" | null;
  targetStateTime: number;
  setTargetState:  (state: "playing" | "paused" | null) => void;
  clearTargetState: () => void;
  clear:           () => void;

  setCurrentTrack: (track: TrackItem | null) => void;
  setPlaying:      (playing: boolean) => void;
  onEvent:         (payload: unknown) => void;
  incrementPos:    () => void;
  setPosition:     (ms: number) => void;
  setVolume:       (v: number) => void;
  setMuted:        (m: boolean) => void;
  setSessionReady: () => void;
}

export const usePlayerStore = create<PlayerStore>()(
  persist(
    (set) => ({
      queueOpen:    false,
      // queue + lyrics share the right rail, so opening one closes the other
      toggleQueue:  () => set((s) => ({ queueOpen: !s.queueOpen, lyricsOpen: false })),

      lyricsOpen:    false,
      toggleLyrics:  () => set((s) => ({ lyricsOpen: !s.lyricsOpen, queueOpen: false })),
      setLyricsOpen: (open) => set({ lyricsOpen: open }),

      immersiveOpen:     false,
      setImmersiveOpen:  (open) => set({ immersiveOpen: open }),
      toggleImmersive:   () => set((s) => ({ immersiveOpen: !s.immersiveOpen })),
      immersivePanel:    "lyrics",
      setImmersivePanel: (p) => set({ immersivePanel: p }),

      lyricsOffsetMs: -250,
      adjustLyricsOffset: (deltaMs) =>
        set((s) => ({ lyricsOffsetMs: Math.max(-5000, Math.min(5000, s.lyricsOffsetMs + deltaMs)) })),
      setLyricsOffset: (ms) =>
        set({ lyricsOffsetMs: Math.max(-5000, Math.min(5000, Math.round(ms))) }),

      isPlaying:    false,
      sessionReady: false,
      lastPlayingAt: 0,
      currentId:    null,
      currentTrack: null,
      positionMs:   0,
      durationMs:   0,

      volume: 80,
      muted:  false,

      targetState:     null,
      targetStateTime: 0,
      setTargetState:  (state) => set({ targetState: state, targetStateTime: Date.now() }),
      clearTargetState: () => set({ targetState: null, targetStateTime: 0 }),

      clear: () =>
        set({
          isPlaying: false,
          sessionReady: false,
          lastPlayingAt: 0,
          currentId: null,
          currentTrack: null,
          positionMs: 0,
          durationMs: 0,
          targetState: null,
          targetStateTime: 0,
          queueOpen: false,
          lyricsOpen: false,
          immersiveOpen: false,
        }),

      setSessionReady: () => set({ sessionReady: true }),

      setPlaying: (playing) => set({ isPlaying: playing }),

      setCurrentTrack: (track) =>
        set(() => ({
          currentTrack: track,
          currentId:    track?.id ?? null,
          durationMs:   track?.duration_ms ?? 0,
          positionMs:   0,
        })),

      onEvent: (payload) => {
        const msg = payload as {
          type:        string;
          track_id?:   string | null;
          position_ms?: number;
          duration_ms?: number;
        };
        switch (msg.type) {
          case "playing":
            set((s) => {
              // Ignore stale playing event if user recently requested pause
              if (s.targetState === "paused" && Date.now() - s.targetStateTime < 1500) {
                return s;
              }
              return {
                isPlaying:       true,
                sessionReady:    true,
                lastPlayingAt:   Date.now(),
                currentId:       msg.track_id ?? s.currentId,
                positionMs:      msg.position_ms ?? s.positionMs,
                durationMs:      msg.duration_ms ?? s.durationMs,
                targetState:     null,
                targetStateTime: 0,
              };
            });
            break;
          case "paused":
            set((s) => {
              // Ignore stale paused event if user recently requested play
              if (s.targetState === "playing" && Date.now() - s.targetStateTime < 1500) {
                return s;
              }
              return {
                isPlaying:       false,
                sessionReady:    true,
                currentId:       msg.track_id ?? s.currentId,
                positionMs:      msg.position_ms ?? s.positionMs,
                durationMs:      msg.duration_ms ?? s.durationMs,
                targetState:     null,
                targetStateTime: 0,
              };
            });
            break;
          case "position_changed":
            set((s) => ({
              currentId:  msg.track_id ?? s.currentId,
              positionMs: msg.position_ms ?? s.positionMs,
            }));
            break;
          case "stopped":
          case "unavailable":
            set(() => ({ isPlaying: false, positionMs: 0 }));
            break;
          case "end_of_track":
            // Keep isPlaying alive so the upcoming track transitions seamlessly
            set(() => ({ positionMs: 0 }));
            break;
          default:
            break;
        }
      },

      incrementPos: () =>
        set((s) => {
          if (!s.isPlaying) return s;
          const next = s.durationMs > 0
            ? Math.min(s.positionMs + 1000, s.durationMs)
            : s.positionMs + 1000;
          return { positionMs: next };
        }),

      // optimistic position update (e.g on seek) so the bar moves instantly
      // instead of waiting for the next librespot position event
      setPosition: (ms) =>
        set((s) => ({ positionMs: Math.max(0, s.durationMs > 0 ? Math.min(ms, s.durationMs) : ms) })),

      setVolume: (v) => set({ volume: Math.max(0, Math.min(100, Math.round(v))) }),
      setMuted:  (m) => set({ muted: m }),
    }),
    {
      name: "spotify-player",
      partialize: (s) => ({
        volume: s.volume,
        muted:  s.muted,
        lyricsOffsetMs: s.lyricsOffsetMs,
        // persist the identity of what's loaded so a webview reload (HMR,
        // alt-tab + ctrl+s in dev) repopulates the player bar instantly
        // instead of going blank. isPlaying/position stay live, they reconcile
        // from the next librespot event.
        currentTrack: s.currentTrack,
        currentId:    s.currentId,
        durationMs:   s.durationMs,
      }),
    },
  ),
);
