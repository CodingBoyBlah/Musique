/* spotify jam, the way spotify's own clients do it.

a jam is spotify's queue, not ours. social-connect drives this device's
connect player (spirc) like any connect remote would: it transfers the jam
here, relays every skip and pause, and every song anyone adds. so while in a
jam:

- what's playing and what's next come from spirc ("connect:state"), and the
  app's own queue steps aside (no advancing it at the end of a track, no radio)
- the host's controls drive spirc directly; social-connect sees the host's
  device change and brings everyone else along
- a guest's skip / add / play goes to the jam (jamCommand), not their own
  player. their pause is theirs alone, and resuming rejoins the jam where it
  has got to (spirc's jam hold). with "queue only" on, guests can only add

every jam branch in the transport, the queue store and playTrack lands here. */

import type { TrackItem } from "../types/spotify";
import {
  connectAddToQueue, connectLoadTracks, connectNext, connectPrev, connectSetRepeat,
  connectSetShuffle, connectSkipTo, getConnectState, jamHold, pausePlayback,
  resumePlayback, seekPlayback, type ConnectState,
} from "../api/playback";
import { getTracksMetadata } from "../api/internal";
import { jamCommand, type JamCommand } from "../api/social";
import { jamRole, useJamStore } from "../store/jam.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { toast } from "../store/toast.store";
import { errMsg } from "./err";
import { isEpisodeId } from "../utils/episode";

// the most spotify keeps queued ahead, so the most worth handing it
const MAX_LOAD = 100;

export const idToUri = (id: string): string => (isEpisodeId(id) ? id : `spotify:track:${id}`);

// connect speaks uris, the app bare track ids (episodes keep their prefix)
export function uriToId(uri: string): string | null {
  if (uri.startsWith("spotify:episode:")) return uri;
  if (uri.startsWith("spotify:track:")) return uri.slice("spotify:track:".length);
  return null;
}

const fail = (what: string) => (e: unknown) => toast.error(`${what}: ${errMsg(e)}`);

// ── mirroring spirc ──────────────────────────────────────────────────────

const inflight = new Set<string>();

// track rows for jam queue uris, fetched once each
export async function ensureMeta(ids: string[]): Promise<void> {
  const { meta, addMeta } = useJamStore.getState();
  const missing = [...new Set(ids)].filter((id) => !meta[id] && !inflight.has(id));
  if (missing.length === 0) return;
  missing.forEach((id) => inflight.add(id));
  try {
    addMeta(await getTracksMetadata(missing));
  } catch {
    /* rows without metadata just aren't shown */
  } finally {
    missing.forEach((id) => inflight.delete(id));
  }
}

function positionOf(state: ConnectState): number {
  const drift = state.is_playing && !state.is_paused ? Math.max(0, Date.now() - state.timestamp) : 0;
  return Math.max(0, state.position_ms + Math.min(drift, 60 * 60 * 1000));
}

/* put what spotify is playing on this device into the player bar. `force`
   skips the grace period after a local action (used to undo an optimistic
   now-playing that turned into a queue add) */
async function syncNowPlaying(state: ConnectState, force = false): Promise<void> {
  const currentId = state.track ? uriToId(state.track.uri) : null;
  if (!currentId) return;
  await ensureMeta([currentId]);

  const latest = useJamStore.getState();
  // a newer state landed while the metadata loaded
  if (latest.connect !== state && !force) return;
  if (!force && Date.now() - latest.localActionAt < 1500) return;

  const player = usePlayerStore.getState();
  if (player.currentTrack?.id === currentId) return;
  const track = latest.meta[currentId];
  if (!track) return;
  player.setCurrentTrack(track);
  usePlayerStore.getState().setPosition(positionOf(state));
}

export function applyConnectState(state: ConnectState): void {
  const jam = useJamStore.getState();
  jam.setConnect(state);
  if (!jam.session || !state.active) return;
  syncNowPlaying(state).catch(() => {});
  const upcoming = state.next.slice(0, 40).map((e) => uriToId(e.uri)).filter((id): id is string => !!id);
  ensureMeta(upcoming).catch(() => {});
}

export async function refreshConnectState(force = false): Promise<void> {
  const state = await getConnectState().catch(() => null);
  if (!state) return;
  useJamStore.getState().setConnect(state);
  if (useJamStore.getState().session && state.active) await syncNowPlaying(state, force);
}

// ── transport ────────────────────────────────────────────────────────────

function sessionId(): string | null {
  return useJamStore.getState().session?.session_id ?? null;
}

// a guest in a "queue only" jam can add songs and nothing else
function guestMayControl(what: string): boolean {
  const s = useJamStore.getState().session;
  if (s && !s.is_host && s.queue_only_mode) {
    toast(`Only the host can ${what} in this Jam`);
    return false;
  }
  return true;
}

function send(command: JamCommand, what: string): Promise<void> {
  const sid = sessionId();
  if (!sid) return Promise.resolve();
  return jamCommand(sid, command).catch((e) => {
    fail(`Couldn't ${what}`)(e);
    throw e;
  });
}

export function jamSetPlaying(play: boolean): void {
  const p = usePlayerStore.getState();
  p.setTargetState(play ? "playing" : "paused");
  p.setPlaying(play);
  if (jamRole() === "guest") {
    // spotify: a guest's pause is just theirs. coming back rejoins the jam
    // where it is now, not where they left it
    useJamStore.getState().setHeld(!play);
    jamHold(!play).catch(fail(play ? "Couldn't rejoin the Jam" : "Couldn't pause"));
    return;
  }
  // the host's pause is the jam's pause: spirc reports it, everyone follows
  (play ? resumePlayback() : pausePlayback()).catch(fail(play ? "Couldn't resume" : "Couldn't pause"));
}

export function jamNext(): void {
  if (jamRole() === "host") {
    connectNext().catch(fail("Couldn't skip"));
    return;
  }
  if (!guestMayControl("skip")) return;
  send({ endpoint: "skip_next" }, "skip").catch(() => {});
}

export function jamPrev(): void {
  const { positionMs } = usePlayerStore.getState();
  if (jamRole() === "host") {
    if (positionMs > 3000) seekPlayback(0).catch(() => {});
    else connectPrev().catch(fail("Couldn't go back"));
    return;
  }
  if (!guestMayControl("skip")) return;
  if (positionMs > 3000) send({ endpoint: "seek_to", value: 0 }, "restart the song").catch(() => {});
  else send({ endpoint: "skip_prev" }, "go back").catch(() => {});
}

export function jamSeek(ms: number): void {
  if (jamRole() === "host") {
    usePlayerStore.getState().setPosition(ms);
    seekPlayback(ms).catch(() => {});
    return;
  }
  if (!guestMayControl("seek")) return;
  usePlayerStore.getState().setPosition(ms);
  send({ endpoint: "seek_to", value: Math.max(0, Math.floor(ms)) }, "seek").catch(() => {});
}

export function jamToggleShuffle(): void {
  if (jamRole() !== "host") {
    toast("Only the host can shuffle a Jam");
    return;
  }
  const on = !useJamStore.getState().connect?.shuffle;
  connectSetShuffle(on).catch(fail("Couldn't change shuffle"));
}

export function jamCycleRepeat(): void {
  if (jamRole() !== "host") {
    toast("Only the host can change repeat in a Jam");
    return;
  }
  const c = useJamStore.getState().connect;
  // off -> context -> track -> off, like everywhere else
  const [context, track] = c?.repeat_track ? [false, false] : c?.repeat_context ? [false, true] : [true, false];
  connectSetRepeat(context, track).catch(fail("Couldn't change repeat"));
}

export function jamEnqueue(track: TrackItem): void {
  const done = () => toast(`Added to the Jam: ${track.name}`);
  if (jamRole() === "host") {
    connectAddToQueue(track.id).then(done).catch(fail("Couldn't add to the Jam"));
    return;
  }
  send({ endpoint: "add_to_queue", track: { uri: idToUri(track.id) } }, "add to the Jam").then(done).catch(() => {});
}

// a queue row clicked: jump ahead to it
export function jamSkipTo(id: string): void {
  if (jamRole() === "host") {
    connectSkipTo(id).catch(fail("Couldn't play that"));
    return;
  }
  if (!guestMayControl("change the song")) return;
  send({ endpoint: "skip_next", track: { uri: idToUri(id) } }, "play that").catch(() => {});
}

/* something was picked to play (a track row, an album's play button, ...).
   the callers have already put it in the app's queue via playContext and
   shown it as now playing; `id` is the track to start. */
export function jamPlay(id: string): Promise<void> {
  const upcoming = useQueueStore.getState().queue.map((t) => t.id);
  const ids = [id, ...upcoming.filter((u) => u !== id)].slice(0, MAX_LOAD);
  useJamStore.getState().markLocalAction();

  if (jamRole() === "host") {
    // the host's music is the jam's: load the whole list into spirc so it is
    // what the jam hears next, not just this one song
    return connectLoadTracks(ids, 0, 0, true, false).catch((e) => {
      fail("Couldn't play that")(e);
      refreshConnectState(true);
    });
  }

  const undo = () => refreshConnectState(true);
  const track = usePlayerStore.getState().currentTrack;
  if (useJamStore.getState().session?.queue_only_mode) {
    // spotify: in a queue-only jam, picking a song adds it
    if (track?.id === id) jamEnqueue(track);
    else send({ endpoint: "add_to_queue", track: { uri: idToUri(id) } }, "add to the Jam").then(() => toast("Added to the Jam")).catch(() => {});
    return undo();
  }
  return send(
    {
      endpoint: "play",
      context: { pages: [{ tracks: ids.map((t) => ({ uri: idToUri(t) })) }] },
      options: { skip_to: { track_index: 0 } },
    },
    "play that in the Jam",
  ).catch(() => undo());
}

/* the host starting a jam: hand spirc the app's queue first so the jam starts
   with it (and guests see what's coming), keeping the song that's playing */
export async function seedJamFromLocal(): Promise<void> {
  const p = usePlayerStore.getState();
  if (!p.currentTrack) return;
  const queue = useQueueStore.getState().queue.map((t) => t.id);
  const ids = [p.currentTrack.id, ...queue.filter((id) => id !== p.currentTrack!.id)].slice(0, MAX_LOAD);
  await connectLoadTracks(ids, 0, p.positionMs, p.isPlaying, true);
}

// back to the app's own queue: a guest's music stops with the jam
export async function leftJam(wasHost: boolean): Promise<void> {
  const jam = useJamStore.getState();
  const held = jam.held;
  jam.setSession(null);
  if (held) await jamHold(false).catch(() => {});
  if (!wasHost) {
    usePlayerStore.getState().setPlaying(false);
    await pausePlayback().catch(() => {});
  }
}
