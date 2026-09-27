import { isEpisodeId } from "./episode";
import { getRecommendations } from "../api/spotify";
import { getAutoplayTracks, getStation, getTrackRadio } from "../api/internal";
import { toast } from "../store/toast.store";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import type { TrackItem } from "../types/spotify";



let isReplenishing = false;

export async function replenishQueue(seedTrack?: TrackItem | null): Promise<void> {
  if (isReplenishing) return;
  if (seedTrack && isEpisodeId(seedTrack.id)) return;
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

    /* spotify's own autoplay first - it's what the official client would
    queue here. the local taste engine takes over when it has nothing */
    if (seed) {
      const recent = [...q.history.slice(-30).map((t) => t.id), seed.id].filter((id) => !isEpisodeId(id));
      const auto = await getAutoplayTracks(`spotify:track:${seed.id}`, recent).catch(() => []);
      const fresh = auto.filter((t) => !exclude.has(t.id));
      if (fresh.length > 0) {
        const existing = new Set(useQueueStore.getState().queue.map((t) => t.id));
        useQueueStore.getState().appendTracks(fresh.filter((t) => !existing.has(t.id)).slice(0, 25));
        return;
      }
    }

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

// play a list as a radio: first track now, the rest queued behind it
function playList(tracks: TrackItem[], contextId: string): boolean {
  const start = useQueueStore.getState().playContext(tracks, 0, contextId);
  if (!start) return false;
  usePlayerStore.getState().setCurrentTrack(start);
  playTrack(start.id).catch(() => {});
  return true;
}

/* "start radio" on a track: that track now, then spotify's radio for it.
falls back to the local engine when spotify has no radio for it. */
export async function playTrackRadio(seed: TrackItem): Promise<boolean> {
  try {
    const radio = await getTrackRadio(seed.id);
    const rest = radio.tracks.filter((t) => t.id !== seed.id);
    if (rest.length > 0 && playList([seed, ...rest], `radio-${seed.id}`)) {
      toast(radio.title ?? `${seed.name} Radio`);
      return true;
    }
  } catch {
    /* fall through to the local engine */
  }
  return startRadio(seed);
}

/* a station from an artist / album / playlist uri */
export async function playStation(uri: string, name: string): Promise<boolean> {
  try {
    const station = await getStation(uri);
    if (station.tracks.length > 0 && playList(station.tracks, `station-${uri}`)) {
      toast(`${name} Radio`);
      return true;
    }
  } catch {
    /* handled below */
  }
  toast.error(`Couldn't start ${name} Radio`);
  return false;
}

export async function startRadio(seed: TrackItem | null): Promise<boolean> {
  try {
    const q = useQueueStore.getState();

    // spotify's radio for the last track, when it has one
    if (seed && !isEpisodeId(seed.id)) {
      const radio = await getTrackRadio(seed.id).catch(() => null);
      const played = new Set(q.history.slice(-50).map((t) => t.id));
      const fresh = (radio?.tracks ?? []).filter((t) => t.id !== seed.id && !played.has(t.id));
      if (fresh.length > 0 && playList(fresh, "radio")) return true;
    }

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
