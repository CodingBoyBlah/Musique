import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  type AudioQuality,
  getAudioQuality,
  setAudioQuality as setApiAudioQuality,
  getAudioCacheLimit,
  setAudioCacheLimit as setApiAudioCacheLimit,
} from "../api/playback";
import { requestNotificationPermission } from "../api/media";

// what the sidebar's last section lists: only what you pinned, or every
// playlist in your library
export type SidebarMode = "pins" | "playlists";

interface PrefsStore {
  // audio streaming bitrate (96: Normal, 160: High, 320: Very high)
  audioQuality: AudioQuality;
  setAudioQuality: (v: AudioQuality) => void;

  // audio cache limit on disk in megabytes
  audioCacheLimitMb: number;
  setAudioCacheLimitMb: (v: number) => void;

  // os notification each time a new track starts
  notifyOnTrack: boolean;
  setNotifyOnTrack: (v: boolean) => void;

  // ask before closing the window while music is still playing
  promptOnClose: boolean;
  setPromptOnClose: (v: boolean) => void;

  // push the now-playing track to discord as rich presence
  discordPresence: boolean;
  setDiscordPresence: (v: boolean) => void;

  // let the immersive view's blurred-cover background drift. off = still.
  ambientMotion: boolean;
  setAmbientMotion: (v: boolean) => void;

  // whole-app zoom factor (1 = 100%). applied to the webview by lib/zoom.ts
  uiZoom: number;
  setUiZoom: (v: number) => void;

  sidebarMode: SidebarMode;
  setSidebarMode: (v: SidebarMode) => void;
}

export const usePrefsStore = create<PrefsStore>()(
  persist(
    (set, get) => ({
      audioQuality: "320",
      setAudioQuality: (v) => {
        const prev = get().audioQuality;
        set({ audioQuality: v });
        setApiAudioQuality(v).catch((err) => {
          console.error("[prefs] set_audio_quality failed:", err);
          set({ audioQuality: prev });
        });
      },

      audioCacheLimitMb: 2048,
      setAudioCacheLimitMb: (v) => {
        const prev = get().audioCacheLimitMb;
        set({ audioCacheLimitMb: v });
        setApiAudioCacheLimit(v).catch((err) => {
          console.error("[prefs] set_audio_cache_limit failed:", err);
          set({ audioCacheLimitMb: prev });
        });
      },

      notifyOnTrack: true,
      setNotifyOnTrack: (v) => {
        set({ notifyOnTrack: v });
        if (v) {
          requestNotificationPermission().catch(() => {});
        }
      },

      promptOnClose: true,
      setPromptOnClose: (v) => set({ promptOnClose: v }),

      discordPresence: true,
      setDiscordPresence: (v) => set({ discordPresence: v }),

      ambientMotion: true,
      setAmbientMotion: (v) => set({ ambientMotion: v }),

      uiZoom: 1,
      setUiZoom: (v) => set({ uiZoom: v }),

      sidebarMode: "pins",
      setSidebarMode: (v) => set({ sidebarMode: v }),
    }),
    { name: "spotify-prefs" },
  ),
);

// Sync with backend on startup
getAudioQuality()
  .then((q) => {
    if (q === "96" || q === "160" || q === "320") {
      usePrefsStore.setState({ audioQuality: q });
    }
  })
  .catch(() => {});

getAudioCacheLimit()
  .then((mb) => {
    if (typeof mb === "number" && mb > 0) {
      usePrefsStore.setState({ audioCacheLimitMb: mb });
    }
  })
  .catch(() => {});
