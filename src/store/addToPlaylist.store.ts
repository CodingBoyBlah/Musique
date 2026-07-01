import { create } from "zustand";
import type { TrackItem } from "../types/spotify";

interface AddToPlaylistStore {
  track: TrackItem | null;
  open:  (track: TrackItem) => void;
  close: () => void;
}


export const useAddToPlaylistStore = create<AddToPlaylistStore>((set) => ({
  track: null,
  open:  (track) => set({ track }),
  close: () => set({ track: null }),
}));
