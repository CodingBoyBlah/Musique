import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  type AudioQuality,
  getAudioQuality,
  setAudioQuality as setApiAudioQuality,
} from "../api/playback";

interface PrefsStore {
  // audio streaming bitrate (96: Normal, 160: High, 320: Very high)
  audioQuality: AudioQuality;
  setAudioQuality: (v: AudioQuality) => void;

  // os notification each time a new track starts
  notifyOnTrack: boolean;
  setNotifyOnTrack: (v: boolean) => void;

  // ask before closing the window while music is still playing
  promptOnClose: boolean;
  setPromptOnClose: (v: boolean) => void;

  // push the now-playing track to discord as rich presence
  discordPresence: boolean;
  setDiscordPresence: (v: boolean) => void;
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

      notifyOnTrack: true,
      setNotifyOnTrack: (v) => set({ notifyOnTrack: v }),

      promptOnClose: true,
      setPromptOnClose: (v) => set({ promptOnClose: v }),

      discordPresence: true,
      setDiscordPresence: (v) => set({ discordPresence: v }),
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
