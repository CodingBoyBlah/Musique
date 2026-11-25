import { create } from "zustand";

interface WidevineStore {
  /** The setup dialog is showing. */
  open: boolean;
  /** The dialog was dismissed this session, so it does not reopen on its own. */
  dismissed: boolean;
  show: () => void;
  close: () => void;
}

/**
 * Drives the Widevine setup dialog. Apple Music tracks are protected, so the
 * dialog opens on its own after an Apple Music sign-in when the module is
 * missing, and again from playback when a track needs it.
 */
export const useWidevineStore = create<WidevineStore>((set) => ({
  open: false,
  dismissed: false,
  show: () => set({ open: true }),
  close: () => set({ open: false, dismissed: true }),
}));
