import { create } from "zustand";
import { persist } from "zustand/middleware";


interface PrefsStore {
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
    (set) => ({
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
