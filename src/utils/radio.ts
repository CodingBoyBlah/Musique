import { getRecommendations } from "../api/spotify";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import type { TrackItem } from "../types/spotify";



export async function startRadio(seed: TrackItem | null): Promise<boolean> {
  try {
    const q = useQueueStore.getState();

    const seedArtists = new Set<string>();
    seed?.artists.forEach((a) => seedArtists.add(a.id));
    // artists from the context just played (playlist/album), keeps the radio in  the same neighbourhood instead of orbiting one artist
    q.contextTracks.slice(0, 50).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));
    // recently played artists
    q.history.slice(-15).forEach((t) => t.artists.forEach((a) => seedArtists.add(a.id)));

    const exclude = new Set<string>();
    q.history.forEach((t) => exclude.add(t.id));
    q.contextTracks.forEach((t) => exclude.add(t.id));
    if (seed) exclude.add(seed.id);

    const recs = await getRecommendations([...seedArtists], 30, [...exclude]);
    if (recs.length === 0) { usePlayerStore.getState().setCurrentTrack(null); return false; }

    const start = useQueueStore.getState().playContext(recs, 0, "radio");
    if (start) {
      usePlayerStore.getState().setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
      return true;
    }
    return false;
  } catch {
    usePlayerStore.getState().setCurrentTrack(null);
    return false;
  }
}
