import { create } from "zustand";
import type { JamSession } from "../api/social";
import type { ConnectState } from "../api/playback";
import type { TrackItem } from "../types/spotify";

/* the spotify jam this app is in, and the connect state spotify is driving it
with. while `session` is set the queue belongs to the jam (spirc holds it,
everyone edits it) and the app's own queue steps aside - see lib/jam.ts. */
interface JamStore {
  session: JamSession | null;
  // what this device's connect player last reported: the jam's current
  // track and queue when in a jam
  connect: ConnectState | null;
  // track rows for the uris in `connect`, by app id
  meta: Record<string, TrackItem>;
  // a guest paused just for themselves
  held: boolean;
  // wall clock of the last play/skip this app asked for, so the connect state
  // that was already in flight doesn't flip the now-playing card back
  localActionAt: number;

  setSession: (s: JamSession | null) => void;
  setConnect: (c: ConnectState | null) => void;
  addMeta: (tracks: TrackItem[]) => void;
  setHeld: (held: boolean) => void;
  markLocalAction: () => void;
}

export const useJamStore = create<JamStore>()((set) => ({
  session: null,
  connect: null,
  meta: {},
  held: false,
  localActionAt: 0,

  setSession: (session) => set((s) => (session ? { session } : { session: null, held: false, meta: s.meta })),
  setConnect: (connect) => set({ connect }),
  addMeta: (tracks) =>
    set((s) => {
      if (tracks.length === 0) return s;
      const meta = { ...s.meta };
      for (const t of tracks) meta[t.id] = t;
      // bounded: a long jam shouldn't grow this forever
      const keys = Object.keys(meta);
      if (keys.length > 600) for (const k of keys.slice(0, keys.length - 400)) delete meta[k];
      return { meta };
    }),
  setHeld: (held) => set({ held }),
  markLocalAction: () => set({ localActionAt: Date.now() }),
}));

export type JamRole = "host" | "guest" | null;

export function jamRole(): JamRole {
  const s = useJamStore.getState().session;
  return s ? (s.is_host ? "host" : "guest") : null;
}

export const useJamRole = (): JamRole =>
  useJamStore((s) => (s.session ? (s.session.is_host ? "host" : "guest") : null));
