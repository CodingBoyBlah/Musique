import { createJSONStorage } from "zustand/middleware";

/* Storage backend for zustand's `persist` that skips writes which would not
   change anything.

   `persist` re-serializes and writes on EVERY store update, not just when a
   field it actually persists changes. The player store ticks `positionMs` once
   a second for the whole time music is playing, and the queue store updates on
   any queue touch - so the app was running a `JSON.stringify` of the current
   track object (and, for the queue, up to 150 more) plus a synchronous
   `localStorage.setItem` every single second, almost always producing a string
   byte-identical to the one already stored. localStorage writes are synchronous
   and disk-backed, so that landed on the main thread, forever, during playback.

   Comparing against the last value written keeps the persisted state exactly as
   it was and drops the redundant writes. */
const lastWritten = new Map<string, string>();

const dedupedLocalStorage = {
  getItem(name: string): string | null {
    const raw = localStorage.getItem(name);
    if (raw !== null) lastWritten.set(name, raw);
    return raw;
  },
  setItem(name: string, value: string): void {
    if (lastWritten.get(name) === value) return;
    lastWritten.set(name, value);
    localStorage.setItem(name, value);
  },
  removeItem(name: string): void {
    lastWritten.delete(name);
    localStorage.removeItem(name);
  },
};

export const dedupedStorage = () => createJSONStorage(() => dedupedLocalStorage);
