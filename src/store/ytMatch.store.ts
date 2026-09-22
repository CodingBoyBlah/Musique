import { create } from "zustand";

interface YtMatchStore {
  /** Spotify track id whose YouTube match is being inspected, or null. */
  trackId: string | null;
  /** Shown in the modal header so it doesn't have to refetch the track. */
  label:   string | null;
  open:    (trackId: string, label: string) => void;
  close:   () => void;
}

/**
 * Drives the "which YouTube upload is this?" picker.
 *
 * Kept as a global store rather than local state because it's opened from
 * several places - the player bar badge and the track context menu - for
 * whatever track is relevant there.
 */
export const useYtMatchStore = create<YtMatchStore>((set) => ({
  trackId: null,
  label:   null,
  open:    (trackId, label) => set({ trackId, label }),
  close:   () => set({ trackId: null, label: null }),
}));
