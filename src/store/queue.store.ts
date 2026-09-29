import { create } from "zustand";
import { persist } from "zustand/middleware";
import { dedupedStorage } from "../lib/persistStorage";
import type { TrackItem } from "../types/spotify";
import { usePlayerStore } from "./player.store";
import { remoteAddToQueue, remoteSetShuffle, remoteSetRepeat, type RemoteRepeat } from "../api/connect";
import { toast } from "./toast.store";
import { jamRole } from "./jam.store";
import { jamCycleRepeat, jamEnqueue, jamToggleShuffle } from "../lib/jam";

const PERSIST_CAP = 1000;

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
  appendTracks:  (tracks: TrackItem[]) => void;
  playNext:      (track: TrackItem) => void;
  removeAt:      (idx: number) => void;
  reorder:       (from: number, to: number) => void;
  // replace the upcoming order wholesale (live drag-to-reorder in the queue)
  setQueue:      (queue: TrackItem[]) => void;
  clearQueue:    () => void;
  clearHistory:  () => void;
  clearAll:      () => void;
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

      // each queue entry gets its own object, even when the same track is
      // queued twice. the queue panel keys rows by object identity, so two
      // entries sharing one object would share one row (and removing one
      // would animate the other out).
      enqueue: (track) => {
        // in a jam the queue is the jam's, shared with everyone in it
        if (jamRole()) {
          jamEnqueue(track);
          return;
        }
        // another device is playing: the queue that matters is spotify's own
        if (usePlayerStore.getState().isRemotePlayback) {
          remoteAddToQueue(track.id)
            .then(() => toast(`Queued on ${usePlayerStore.getState().activeDevice?.name ?? "device"}`))
            .catch(() => toast.error("Couldn't queue on the remote device"));
          return;
        }
        set((s) => ({ queue: [...s.queue, { ...track }] }));
      },

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

      playNext: (track) => {
        // spotify's jam has one queue, in the order people add to it
        if (jamRole()) {
          jamEnqueue(track);
          return;
        }
        set((s) => ({ queue: [{ ...track }, ...s.queue] }));
      },

      appendTracks: (tracks) =>
        set((s) => {
          const existing = new Set(s.queue.map((t) => t.id));
          const fresh = tracks.filter((t) => !existing.has(t.id));
          return { queue: [...s.queue, ...fresh] };
        }),

      removeAt: (idx) =>
        set((s) => ({ queue: s.queue.filter((_, i) => i !== idx) })),

      reorder: (from, to) =>
        set((s) => {
          const q = [...s.queue];
          const [item] = q.splice(from, 1);
          q.splice(to, 0, item);
          return { queue: q };
        }),

      setQueue: (queue) => set({ queue }),

      clearQueue:   () => set({ queue: [] }),
      clearHistory: () => set({ history: [] }),
      clearAll:     () => set({ queue: [], history: [], contextTracks: [], contextId: null }),

      toggleShuffle: () => {
        if (jamRole()) {
          jamToggleShuffle();
          return;
        }
        const p = usePlayerStore.getState();
        if (p.isRemotePlayback) {
          const next = !p.remoteShuffle;
          p.setRemoteShuffle(next);
          remoteSetShuffle(next).catch(() => {
            usePlayerStore.getState().setRemoteShuffle(!next);
            toast.error("Couldn't change shuffle on the remote device");
          });
          return;
        }
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
        });
      },

      cycleRepeat: () => {
        if (jamRole()) {
          jamCycleRepeat();
          return;
        }
        const p = usePlayerStore.getState();
        if (p.isRemotePlayback) {
          const prev = p.remoteRepeat;
          const next: RemoteRepeat = prev === "off" ? "context" : prev === "context" ? "track" : "off";
          p.setRemoteRepeat(next);
          remoteSetRepeat(next).catch(() => {
            usePlayerStore.getState().setRemoteRepeat(prev);
            toast.error("Couldn't change repeat on the remote device");
          });
          return;
        }
        set((s) => {
          const next: Repeat =
            s.repeat === "none" ? "all" : s.repeat === "all" ? "one" : "none";
          return { repeat: next };
        });
      },

      advance: (current) => {
        const { queue, history, contextTracks, repeat, shuffle } = get();

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
          set({ queue: rest, history: current ? [...history, current].slice(-50) : history });
          return next;
        }

        // queue empty: queue should never end!
        // Seamlessly loop context tracks (shuffled or in order)
        if (contextTracks.length > 0) {
          const order = shuffle && contextTracks.length > 1
            ? shuffled(contextTracks)
            : [...contextTracks];
          const [next, ...rest] = order;
          const newHistory = current ? [...history, current].slice(-50) : history;
          set({ queue: rest, history: newHistory });
          return next;
        }

        // fallback: replay from history so music never stops
        if (history.length > 0) {
          const order = shuffled(history);
          const [next, ...rest] = order;
          set({ queue: rest, history: current ? [current] : [] });
          return next;
        }

        return current;
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
        const { queue, history, contextTracks, repeat } = get();
        if (repeat === "one" && current) return current;
        if (queue.length > 0) return queue[0];
        if (repeat === "all" && contextTracks.length > 0) {
          return contextTracks[0] ?? null;
        }
        if (contextTracks.length > 0) {
          const candidates = contextTracks.filter((t) => t.id !== current?.id);
          return candidates[0] ?? contextTracks[0] ?? null;
        }
        if (history.length > 0) {
          const candidates = history.filter((t) => t.id !== current?.id);
          return candidates[0] ?? null;
        }
        return current;
      },
    }),
    {
      name: "spotify-queue",
      storage: dedupedStorage(),
      // Capped so a huge context can't blow the localStorage quota, but high
      // enough that real playlists survive a restart whole - repeat-all loops
      // over whatever was restored, so a low cap silently drops the tail.
      // This store only persists on queue changes, not on the position tick.
      partialize: (s) => ({
        queue:         s.queue.slice(0, PERSIST_CAP),
        history:       s.history.slice(-30),
        contextTracks: s.contextTracks.slice(0, PERSIST_CAP),
        contextId:     s.contextId,
        shuffle:       s.shuffle,
        repeat:        s.repeat,
      }),
    },
  ),
);
