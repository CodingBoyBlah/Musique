import type { PersistStorage, StorageValue } from "zustand/middleware";

/* Storage backend for zustand's `persist` that skips serialization and writes
   which would not change anything.

   `persist` with `createJSONStorage` re-serializes on EVERY store update, not
   just when a field it actually persists changes. The player store ticks
   `positionMs` once a second for the whole time music is playing, so the app
   was running a `JSON.stringify` of the current track object plus a synchronous
   `localStorage.setItem` check every single second.

   We perform shallow comparison against the last persisted state object before
   stringifying, avoiding redundant `JSON.stringify` operations entirely on
   non-persisted updates (like position ticks), while ensuring disk writes
   are deduped against the last written string. */

function shallowEqual(a: unknown, b: unknown): boolean {
  if (Object.is(a, b)) return true;
  if (typeof a !== "object" || a === null || typeof b !== "object" || b === null) return false;
  const keysA = Object.keys(a);
  const keysB = Object.keys(b);
  if (keysA.length !== keysB.length) return false;
  for (let i = 0; i < keysA.length; i++) {
    const key = keysA[i];
    if (
      !Object.prototype.hasOwnProperty.call(b, key) ||
      !Object.is((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key])
    ) {
      return false;
    }
  }
  return true;
}

export function dedupedStorage<S>(): PersistStorage<S> {
  const lastWritten = new Map<string, string>();
  const lastState = new Map<string, StorageValue<S>>();
  return {
    getItem(name: string): StorageValue<S> | null {
      if (typeof localStorage === "undefined") return null;
      const raw = localStorage.getItem(name);
      if (raw === null) {
        lastWritten.delete(name);
        lastState.delete(name);
        return null;
      }
      const parsed = JSON.parse(raw) as StorageValue<S>;
      lastWritten.set(name, raw);
      lastState.set(name, parsed);
      return parsed;
    },
    setItem(name: string, value: StorageValue<S>): void {
      if (typeof localStorage === "undefined") return;
      const prev = lastState.get(name);
      if (prev && prev.version === value.version && shallowEqual(prev.state, value.state)) {
        return;
      }

      const serialized = JSON.stringify(value);
      if (lastWritten.get(name) !== serialized) {
        localStorage.setItem(name, serialized);
      }
      lastWritten.set(name, serialized);
      lastState.set(name, value);
    },
    removeItem(name: string): void {
      if (typeof localStorage !== "undefined") {
        localStorage.removeItem(name);
      }
      lastWritten.delete(name);
      lastState.delete(name);
    },
  };
}
