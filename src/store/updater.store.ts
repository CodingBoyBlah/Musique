import { create } from "zustand";

// state machine for the in-app updater UI. Not persisted - it's per-launch.
//   idle        -> no update (or not checked yet); prompt hidden
//   available   -> an update exists; show version + notes + Later/Update
//   downloading -> streaming the update; show progress %
//   installed   -> applied on disk; show "Restart now"
//   error       -> something failed; show the message
export type UpdateStage =
  "idle" | "available" | "downloading" | "installed" | "error";

interface UpdaterState {
  stage: UpdateStage;
  open: boolean; // whether the prompt is visible
  version: string | null; // the NEW version
  notes: string | null; // release notes (markdown/plain body)
  progress: number; // 0..100
  error: string | null;

  showAvailable: (version: string, notes: string | null) => void;
  setDownloading: () => void;
  setProgress: (pct: number) => void;
  setInstalled: () => void;
  setError: (msg: string) => void;
  dismiss: () => void; // "Later"
}

export const useUpdaterStore = create<UpdaterState>((set) => ({
  stage: "idle",
  open: false,
  version: null,
  notes: null,
  progress: 0,
  error: null,

  showAvailable: (version, notes) =>
    set({
      stage: "available",
      open: true,
      version,
      notes,
      progress: 0,
      error: null,
    }),
  setDownloading: () => set({ stage: "downloading", progress: 0 }),
  setProgress: (pct) => set({ progress: pct }),
  setInstalled: () => set({ stage: "installed", progress: 100 }),
  setError: (msg) => set({ stage: "error", error: msg }),
  dismiss: () => set({ open: false }),
}));
