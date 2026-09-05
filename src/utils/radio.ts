import { getRecommendations } from "../api/spotify";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import type { TrackItem } from "../types/spotify";



let isReplenishing = false;

export async function replenishQueue(seedTrack?: TrackItem | null): Promise<void> {
  if (isReplenishing) return;
  const q = useQueueStore.getState();
  if (q.queue.length > 3) return;

  isReplenishing = true;
  try {
    const seed = seedTrack ?? usePlayerStore.getState().currentTrack;
    const seedArtists = new Set<string>();
    seed?.artists.forEach((a) => seedArtists.add(a.id));
    q.contextTracks.slice(0, 30).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));
    q.history.slice(-15).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));

    const exclude = new Set<string>();
    q.history.forEach((t) => exclude.add(t.id));
    q.contextTracks.forEach((t) => exclude.add(t.id));
    q.queue.forEach((t) => exclude.add(t.id));
    if (seed) exclude.add(seed.id);

    let recs = await getRecommendations([...seedArtists], 25, [...exclude]);
    if (recs.length === 0) {
      const relaxedExclude = new Set<string>();
      if (seed) relaxedExclude.add(seed.id);
      q.queue.forEach((t) => relaxedExclude.add(t.id));
      recs = await getRecommendations([...seedArtists], 25, [...relaxedExclude]);
    }

    if (recs.length > 0) {
      const existing = new Set(useQueueStore.getState().queue.map((t) => t.id));
      const fresh = recs.filter((t) => !existing.has(t.id) && t.id !== seed?.id);
      if (fresh.length > 0) {
        useQueueStore.getState().appendTracks(fresh);
      }
    }
  } catch (err) {
    console.error("[radio] replenishQueue failed:", err);
  } finally {
    isReplenishing = false;
  }
}

export async function startRadio(seed: TrackItem | null): Promise<boolean> {
  try {
    const q = useQueueStore.getState();

    const seedArtists = new Set<string>();
    seed?.artists.forEach((a) => seedArtists.add(a.id));
    q.contextTracks.slice(0, 50).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));
    q.history.slice(-15).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));

    const exclude = new Set<string>();
    if (seed) exclude.add(seed.id);
    q.history.slice(-10).forEach((t) => exclude.add(t.id));

    let recs = await getRecommendations([...seedArtists], 30, [...exclude]);
    if (recs.length === 0) {
      recs = await getRecommendations([...seedArtists], 30);
    }

    if (recs.length > 0) {
      const start = useQueueStore.getState().playContext(recs, 0, "radio");
      if (start) {
        usePlayerStore.getState().setCurrentTrack(start);
        playTrack(start.id).catch(() => {});
        return true;
      }
    }

    // Fallback: loop contextTracks or history, NEVER nullify currentTrack
    if (q.contextTracks.length > 0) {
      const next = useQueueStore.getState().advance(seed);
      if (next) {
        usePlayerStore.getState().setCurrentTrack(next);
        playTrack(next.id).catch(() => {});
        return true;
      }
    }
    return false;
  } catch (e) {
    console.error("[radio] startRadio failed:", e);
    // Never kill playback
    const q = useQueueStore.getState();
    if (q.contextTracks.length > 0) {
      const next = useQueueStore.getState().advance(seed);
      if (next) {
        usePlayerStore.getState().setCurrentTrack(next);
        playTrack(next.id).catch(() => {});
        return true;
      }
    }
    return false;
  }
}
