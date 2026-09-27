import { create } from "zustand";
import { persist } from "zustand/middleware";
import { dedupedStorage } from "../lib/persistStorage";
import type { TrackItem } from "../types/spotify";
import type { SpotifyDevice, RemotePlaybackState, RemoteRepeat } from "../api/connect";

interface PlayerStore {
  devicesOpen: boolean;
  toggleDevices: () => void;
  setDevicesOpen: (open: boolean) => void;

  devices: SpotifyDevice[];
  setDevices: (devices: SpotifyDevice[]) => void;

  activeDevice: SpotifyDevice | null;
  setActiveDevice: (device: SpotifyDevice | null) => void;

  musiqueDeviceId: string | null;
  setMusiqueDeviceId: (id: string | null) => void;

  isRemotePlayback: boolean;
  setIsRemotePlayback: (isRemote: boolean) => void;

  syncRemotePlayback: (state: RemotePlaybackState | null) => void;
  // shuffle/repeat as the remote device reports them. local playback keeps its
  // own in queue.store; these only drive the buttons while another device plays
  remoteShuffle: boolean;
  remoteRepeat:  RemoteRepeat;
  setRemoteShuffle: (on: boolean) => void;
  setRemoteRepeat:  (r: RemoteRepeat) => void;

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
  immersivePanel: "lyrics" | "queue" | "credits";
  setImmersivePanel: (p: "lyrics" | "queue" | "credits") => void;

  // manual sync nudge for lyrics, in ms. purely a personal preference now:
  // the systematic error it used to paper over (reported position runs ahead of
  // what you hear, by however much is sitting in the sink queue + device buffer)
  // is measured for real and corrected in useLyricClock, so this defaults to 0.
  // negative = highlight later. persisted.
  lyricsOffsetMs: number;
  adjustLyricsOffset: (deltaMs: number) => void;
  setLyricsOffset: (ms: number) => void;

  // provider-supplied translation / romanization, shown under each line when
  // the source ships them. persisted, off by default.
  lyricsShowTranslation: boolean;
  setLyricsShowTranslation: (on: boolean) => void;
  lyricsShowRoman: boolean;
  setLyricsShowRoman: (on: boolean) => void;

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
  setLastPlayingAt: (time: number) => void;
  onEvent:         (payload: unknown) => void;
  incrementPos:    () => void;
  setPosition:     (ms: number) => void;
  setVolume:       (v: number) => void;
  setMuted:        (m: boolean) => void;
  setSessionReady: () => void;
}

export const usePlayerStore = create<PlayerStore>()(
  persist(
    (set, get) => ({
      devicesOpen: false,
      toggleDevices: () => set((s) => ({ devicesOpen: !s.devicesOpen })),
      setDevicesOpen: (open) => set({ devicesOpen: open }),

      devices: [],
      setDevices: (devices) => set({ devices }),

      activeDevice: null,
      setActiveDevice: (device) => set({ activeDevice: device }),

      musiqueDeviceId: null,
      setMusiqueDeviceId: (id) => set({ musiqueDeviceId: id }),

      isRemotePlayback: false,
      setIsRemotePlayback: (isRemote) => set({ isRemotePlayback: isRemote }),

      remoteShuffle: false,
      remoteRepeat:  "off",
      setRemoteShuffle: (on) => set({ remoteShuffle: on }),
      setRemoteRepeat:  (r) => set({ remoteRepeat: r }),

      syncRemotePlayback: (state) =>
        set((s) => {
          if (!state || !state.device || !state.device.is_active) {
            // Keep remote session stable across transient 204s / buffering dips
            return s;
          }

          const devName = state.device.name?.toLowerCase() ?? "";
          const isThisDevice =
            Boolean(s.musiqueDeviceId && state.device.id === s.musiqueDeviceId) ||
            devName === "musique";

          // If local Librespot started playing within 5s, ignore stale poll snapshots from other devices
          const localJustStarted = !s.isRemotePlayback && (Date.now() - s.lastPlayingAt < 5000);
          if (localJustStarted && !isThisDevice) {
            return s;
          }

          // Only maintain or enter remote playback mode if:
          // 1. We are already in remote playback mode (s.isRemotePlayback), OR
          // 2. Music is genuinely actively playing on an external device (state.is_playing && !isThisDevice)
          const shouldBeRemote = !isThisDevice && (s.isRemotePlayback || state.is_playing);

          if (!shouldBeRemote) {
            return {
              isRemotePlayback: false,
              activeDevice: state.device,
            };
          }

          // Estimate position with network latency compensation from snapshot timestamp
          let remotePos = s.positionMs;
          if (state.progress_ms != null) {
            const rawPos = Number(state.progress_ms);
            const latencyOffset = state.timestamp
              ? Math.max(0, Math.min(3000, Date.now() - Number(state.timestamp)))
              : 0;
            const estimatedPos = rawPos + (state.is_playing ? latencyOffset : 0);

            const isTrackChange = state.track?.id !== s.currentId;
            const isPlayStateChange = state.is_playing !== s.isPlaying;
            const drift = Math.abs(s.positionMs - estimatedPos);

            // Sync position on track change, play/pause change, or significant drift (> 2500ms)
            // Otherwise preserve local smooth 1s increment to prevent scrubber jitter
            if (isTrackChange || isPlayStateChange || drift > 2500) {
              remotePos = estimatedPos;
            }
          }

          return {
            isRemotePlayback: true,
            activeDevice: state.device,
            isPlaying: state.is_playing,
            currentTrack: state.track ?? s.currentTrack,
            currentId: state.track?.id ?? s.currentId,
            durationMs: state.track?.duration_ms ?? s.durationMs,
            positionMs: remotePos,
            volume: state.device.volume_percent != null ? state.device.volume_percent : s.volume,
            remoteShuffle: state.shuffle_state,
            remoteRepeat: (["off", "context", "track"].includes(state.repeat_state)
              ? state.repeat_state
              : "off") as RemoteRepeat,
          };
        }),

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

      // 0, because the drift this used to cancel is now measured and removed
      // in useLyricClock (see lib/outputLatency.ts). a non-zero value here would
      // be corrected twice.
      lyricsOffsetMs: 0,
      adjustLyricsOffset: (deltaMs) =>
        set((s) => ({ lyricsOffsetMs: Math.max(-5000, Math.min(5000, s.lyricsOffsetMs + deltaMs)) })),
      setLyricsOffset: (ms) =>
        set({ lyricsOffsetMs: Math.max(-5000, Math.min(5000, Math.round(ms))) }),

      lyricsShowTranslation: false,
      setLyricsShowTranslation: (on) => set({ lyricsShowTranslation: on }),
      lyricsShowRoman: false,
      setLyricsShowRoman: (on) => set({ lyricsShowRoman: on }),

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
          devicesOpen: false,
          isRemotePlayback: false,
          activeDevice: null,
        }),

      setSessionReady: () => set({ sessionReady: true }),

      setPlaying: (playing) =>
        set((s) => ({
          isPlaying: playing,
          lastPlayingAt: playing ? Date.now() : s.lastPlayingAt,
        })),

      setLastPlayingAt: (time) => set({ lastPlayingAt: time }),

      setCurrentTrack: (track) =>
        set(() => ({
          isRemotePlayback: false,
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
        // If we are currently controlling a remote device, ignore local audio sink events
        // (like stopped/paused/position_changed from shutting down the local sink)
        // so remote state isn't wiped out.
        if (get().isRemotePlayback && msg.type !== "playing") {
          return;
        }
        switch (msg.type) {
          case "playing":
            set((s) => {
              // Ignore stale playing event if user recently requested pause
              if (s.targetState === "paused" && Date.now() - s.targetStateTime < 1500) {
                return s;
              }
              return {
                isRemotePlayback: false,
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
              if (s.targetState === "playing" && Date.now() - s.targetStateTime < 3000) {
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
            set((s) => {
              // Ignore stale stopped event if user recently requested play (e.g. previous track tearing down)
              if (s.targetState === "playing" && Date.now() - s.targetStateTime < 3000) {
                return s;
              }
              return { isPlaying: false, positionMs: 0 };
            });
            break;
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
      storage: dedupedStorage(),
      /* v1 moved output-latency compensation out of `lyricsOffsetMs` and into a
         real measurement. Anyone upgrading still has the old value persisted,
         and leaving it would subtract the buffer twice - lyrics would land as
         far LATE as they used to be early. The legacy default (-250) becomes 0;
         a value the user actually tuned keeps its intent by having the old
         assumed baseline added back, leaving just their personal part. */
      version: 1,
      migrate: (persisted, version) => {
        const st = (persisted ?? {}) as Partial<PlayerStore>;
        if (version < 1 && typeof st.lyricsOffsetMs === "number") {
          const LEGACY_BASELINE_MS = -250;
          st.lyricsOffsetMs =
            st.lyricsOffsetMs === LEGACY_BASELINE_MS ? 0 : st.lyricsOffsetMs - LEGACY_BASELINE_MS;
        }
        return st as PlayerStore;
      },
      partialize: (s) => ({
        volume: s.volume,
        muted:  s.muted,
        lyricsOffsetMs: s.lyricsOffsetMs,
        lyricsShowTranslation: s.lyricsShowTranslation,
        lyricsShowRoman: s.lyricsShowRoman,
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
