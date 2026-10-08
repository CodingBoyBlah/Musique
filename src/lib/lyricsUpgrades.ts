import { listen } from "@tauri-apps/api/event";
import type { QueryClient } from "@tanstack/react-query";
import type { Lyrics } from "../api/lyrics";

interface Subscriber {
  client: QueryClient;
  trackId: () => string | undefined;
}

const subscribers = new Set<Subscriber>();
let unlisten: (() => void) | null = null;
let pending = false;

function connect() {
  if (pending || unlisten) return;
  pending = true;
  listen<Lyrics>("lyrics:upgraded", ({ payload }) => {
    if (!payload) return;
    const updated = new Set<QueryClient>();
    for (const subscriber of subscribers) {
      if (payload.track_id !== subscriber.trackId() || updated.has(subscriber.client)) continue;
      updated.add(subscriber.client);
      subscriber.client.setQueryData(["lyrics", payload.track_id], payload);
    }
  }).then((release) => {
    pending = false;
    // A consumer may remount before registration resolves. Reuse that
    // registration rather than creating another native listener.
    if (subscribers.size) unlisten = release;
    else release();
  }).catch(() => { pending = false; });
}

/** Share the native subscription while keeping query caches and tracks separate. */
export function subscribeLyricsUpgrades(client: QueryClient, trackId: Subscriber["trackId"]): () => void {
  const subscriber = { client, trackId };
  subscribers.add(subscriber);
  connect();
  return () => {
    subscribers.delete(subscriber);
    if (!subscribers.size && unlisten) {
      unlisten();
      unlisten = null;
    }
  };
}
