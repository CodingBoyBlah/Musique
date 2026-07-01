import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface PinnedItem {
  id:        string;
  name:      string;
  image_url: string | null;
  type:      "playlist" | "album";
}

interface PinsStore {
  pins:       PinnedItem[];
  isPinned:   (id: string) => boolean;
  togglePin:  (item: PinnedItem) => void;
  removePin:  (id: string) => void;
}

export const usePinsStore = create<PinsStore>()(
  persist(
    (set, get) => ({
      pins: [],

      isPinned: (id) => get().pins.some((p) => p.id === id),

      togglePin: (item) =>
        set((s) =>
          s.pins.some((p) => p.id === item.id)
            ? { pins: s.pins.filter((p) => p.id !== item.id) }
            : { pins: [...s.pins, item] },
        ),

      removePin: (id) =>
        set((s) => ({ pins: s.pins.filter((p) => p.id !== id) })),
    }),
    { name: "spotify-pins" },
  ),
);
