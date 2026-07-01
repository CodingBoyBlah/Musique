import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { TrackItem } from "../types/spotify";
import { usePlayerStore } from "./player.store";

type Repeat = "none" | "one" | "all";

function shuffled<T>(arr: T[]): T[] {
  const a = [...arr];
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

interface QueueStore {
  queue:         TrackItem[];
  history:       TrackItem[];
  contextTracks: TrackItem[];     // full ordered list of the current play context
  contextId:     string | null;   // id of the playlist/album/etc playing right now
  shuffle:       boolean;
  repeat:        Repeat;

  enqueue:       (track: TrackItem) => void;
  playNext:      (track: TrackItem) => void;
  removeAt:      (idx: number) => void;
  reorder:       (from: number, to: number) => void;
  clearQueue:    () => void;
  clearHistory:  () => void;
  toggleShuffle: () => void;
  cycleRepeat:   () => void;

  
  playContext:         (tracks: TrackItem[], startIndex: number, contextId?: string | null) => TrackItem | null;
  // same but shuffled - random track first, rest queued in random order
  playContextShuffled: (tracks: TrackItem[], contextId?: string | null) => TrackItem | null;

  // next track to play (mutates queue/history) null = nothing left.
  advance:  (current: TrackItem | null) => TrackItem | null;
  // track to play when the user hits prev (mutates queue/history)
  previous: (current: TrackItem | null) => TrackItem | null;
  // peek at the next track without consuming it. used for preloading.
  peek:     (current: TrackItem | null) => TrackItem | null;
}

export const useQueueStore = create<QueueStore>()(
  persist(
    (set, get) => ({
      queue:         [],
      history:       [],
      contextTracks: [],
      contextId:     null,
      shuffle:       false,
      repeat:        "none",

      enqueue: (track) =>
        set((s) => ({ queue: [...s.queue, track] })),

      playContext: (tracks, startIndex, contextId = null) => {
        const start = tracks[startIndex] ?? null;
        if (!start) return null;
        set({
          contextTracks: tracks,
          contextId,
          history:       [],
          queue:         tracks.slice(startIndex + 1),
          shuffle:       false,
        });
        return start;
      },

      playContextShuffled: (tracks, contextId = null) => {
        if (tracks.length === 0) return null;
        const order = shuffled(tracks);
        const [start, ...rest] = order;
        set({
          contextTracks: tracks,
          contextId,
          history:       [],
          queue:         rest,
          shuffle:       true,
        });
        return start;
      },

      playNext: (track) =>
        set((s) => ({ queue: [track, ...s.queue] })),

      removeAt: (idx) =>
        set((s) => ({ queue: s.queue.filter((_, i) => i !== idx) })),

      reorder: (from, to) =>
        set((s) => {
          const q = [...s.queue];
          const [item] = q.splice(from, 1);
          q.splice(to, 0, item);
          return { queue: q };
        }),

      clearQueue:   () => set({ queue: [] }),
      clearHistory: () => set({ history: [] }),

      toggleShuffle: () =>
        set((s) => {
          if (!s.shuffle) {
            // turning on: shuffle the upcoming queue
            return { shuffle: true, queue: shuffled(s.queue) };
          }
          // turning off: put back original order of whatever's still upcoming
          if (s.contextTracks.length > 0) {
            const currentId = usePlayerStore.getState().currentId;
            const played    = new Set(s.history.map((t) => t.id));
            if (currentId) played.add(currentId);
            const upcoming = s.contextTracks.filter((t) => !played.has(t.id));
            return { shuffle: false, queue: upcoming };
          }
          return { shuffle: false };
        }),

      cycleRepeat: () =>
        set((s) => {
          const next: Repeat =
            s.repeat === "none" ? "all" : s.repeat === "all" ? "one" : "none";
          return { repeat: next };
        }),

      advance: (current) => {
        const { queue, history, contextTracks, repeat } = get();

        if (repeat === "one" && current) return current;

        // normal case: pull the next queued track, push current onto history
        if (queue.length > 0) {
          const [next, ...rest] = queue;
          const newHistory = current ? [...history, current].slice(-50) : history;
          set({ queue: rest, history: newHistory });
          return next;
        }

        // queue empty + repeat all: restart the whole context from the top
        if (repeat === "all" && contextTracks.length > 0) {
          const [next, ...rest] = contextTracks;
          set({ queue: rest, history: [] });
          return next;
        }

        // nothing to advance to. do NOT touch history (caller may start radio).
        return null;
      },

      previous: (current) => {
        const { queue, history } = get();
        if (history.length === 0) return current;

        const newHistory = [...history];
        const prev       = newHistory.pop()!;
        const newQueue   = current ? [current, ...queue] : [...queue];

        set({ queue: newQueue, history: newHistory });
        return prev;
      },

      peek: (current) => {
        const { queue, history, repeat } = get();
        if (repeat === "one" && current) return current;
        if (queue.length > 0) return queue[0];
        if (repeat === "all" && history.length > 0) {
          const refill = history.filter((t) => t.id !== current?.id);
          return refill[0] ?? null;
        }
        return null;
      },
    }),
    {
      name: "spotify-queue",
      partialize: (s) => ({
        queue:         s.queue,
        history:       s.history.slice(-30),
        contextTracks: s.contextTracks,
        contextId:     s.contextId,
        shuffle:       s.shuffle,
        repeat:        s.repeat,
      }),
    },
  ),
);
