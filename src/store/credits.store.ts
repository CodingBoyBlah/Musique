import { create } from "zustand";
import type { TrackItem } from "../types/spotify";

// which track's credits dialog is open (one global dialog, mounted in Layout)
interface CreditsStore {
  track: TrackItem | null;
  open: (track: TrackItem) => void;
  close: () => void;
}

export const useCreditsStore = create<CreditsStore>((set) => ({
  track: null,
  open: (track) => set({ track }),
  close: () => set({ track: null }),
}));
