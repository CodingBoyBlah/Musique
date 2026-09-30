import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { jamRole } from "../store/jam.store";
import { jamPlay, jamSetPlaying } from "../lib/jam";

export interface VolumeState { level: number; muted: boolean; }

/**
 * Roll back the optimistic "playing" state and tell the user why.
 *
 * Both play entry points set `playing: true` before the IPC call so the UI
 * responds instantly. If the call then fails, that optimism has to be undone -
 * otherwise the transport sits there claiming to play a track that produced no
 * audio, which is indistinguishable from a bug.
 *
 * This matters most on the YouTube backend used by free accounts: "no
 * acceptable match" is a normal, expected outcome there, and the matcher
 * deliberately refuses rather than playing a near-miss. Refusing silently would
 * throw away the entire point of refusing.
 *
 * Every call site of these functions does `.catch(() => {})`, so centralising
 * the handling here is what actually makes the error reach a human.
 */
const failPlayback = (e: unknown): never => {
  const store = usePlayerStore.getState();
  store.setPlaying(false);
  store.setTargetState("paused");
  toast.error(errMsg(e));
  throw e;
};

/**
 * The Spotify context uri to report `id` under, so the play shows up as
 * "Playlist name · x tracks played" in Spotify's Recents instead of a loose
 * track. Only while `id` actually belongs to the loaded context - a track
 * queued by hand from elsewhere isn't part of that playlist.
 */
const contextUriFor = (id: string): string | null => {
  const { contextUri, contextTracks } = useQueueStore.getState();
  return contextUri && contextTracks.some((t) => t.id === id) ? contextUri : null;
};

export const warmupPlayback  = (): Promise<void>                     => invoke("warmup_playback");
export const playTrack       = (id: string): Promise<void> => {
  // in a jam, what plays is the jam's call (see lib/jam.ts)
  if (jamRole()) return jamPlay(id);
  const store = usePlayerStore.getState();
  store.setIsRemotePlayback(false);
  store.setTargetState("playing");
  store.setPlaying(true);
  store.setLastPlayingAt(Date.now());
  return invoke<void>("play_track", { id, contextUri: contextUriFor(id) }).catch(failPlayback);
};
export const retryPlayTrack  = (id: string): Promise<void>           =>
  invoke("retry_play_track", { id, contextUri: contextUriFor(id) });

// ── connect-driven playback (spotify jam) ──
// in a jam the queue is spotify's, held by this device's connect player
// (spirc). these drive it directly; the ui mirrors it from "connect:state".

export interface ConnectQueueEntry {
  uri: string;
  uid: string;
  provider: "context" | "queue" | "autoplay" | (string & {});
  // in a jam, the username of whoever added it
  queued_by: string | null;
}

export interface ConnectState {
  active: boolean;
  context_uri: string;
  track: ConnectQueueEntry | null;
  next: ConnectQueueEntry[];
  is_playing: boolean;
  is_paused: boolean;
  position_ms: number;
  timestamp: number;
  duration_ms: number;
  shuffle: boolean;
  repeat_context: boolean;
  repeat_track: boolean;
  jam_mode: boolean;
}

export const getConnectState   = (): Promise<ConnectState | null> => invoke("get_connect_state");
export const connectLoadTracks = (ids: string[], index: number, positionMs: number, startPlaying: boolean, keepStream: boolean): Promise<void> =>
  invoke("connect_load_tracks", { ids, index, positionMs: Math.max(0, Math.floor(positionMs)), startPlaying, keepStream });
export const connectAddToQueue = (id: string): Promise<void> => invoke("connect_add_to_queue", { id });
export const connectSkipTo     = (id: string): Promise<void> => invoke("connect_skip_to", { id });
export const connectNext       = (): Promise<void> => invoke("connect_next");
export const connectPrev       = (): Promise<void> => invoke("connect_prev");
export const connectSetShuffle = (on: boolean): Promise<void> => invoke("connect_set_shuffle", { on });
export const connectSetRepeat  = (context: boolean, track: boolean): Promise<void> => invoke("connect_set_repeat", { context, track });
// a jam guest's pause is theirs alone; releasing it rejoins the jam where it is
export const jamHold           = (hold: boolean): Promise<void> => invoke("jam_hold", { hold });
export const pausePlayback   = (): Promise<void>                     => invoke("pause_playback");
export const resumePlayback  = (): Promise<void>                     => invoke("resume_playback");
export const resumeOrPlay    = (id: string, positionMs: number): Promise<void> => {
  // in a jam: carry on the jam's song, or pick a new one through the jam
  if (jamRole()) {
    if (usePlayerStore.getState().currentTrack?.id === id) {
      jamSetPlaying(true);
      return Promise.resolve();
    }
    return jamPlay(id);
  }
  const store = usePlayerStore.getState();
  const safePos = Math.max(0, Math.floor(positionMs || 0));
  store.setIsRemotePlayback(false);
  store.setTargetState("playing");
  store.setPlaying(true);
  store.setPosition(safePos);
  store.setLastPlayingAt(Date.now());
  return invoke<void>("resume_or_play", { id, positionMs: safePos, contextUri: contextUriFor(id) }).catch(failPlayback);
};
export const stopPlayback    = (): Promise<void>                     => invoke("stop_playback");
export const seekPlayback    = (positionMs: number): Promise<void>   => invoke("seek_playback", { positionMs });
export const preloadTrack    = (id: string): Promise<void>           => invoke("preload_track", { id });
export const setVolume       = (level: number): Promise<void>        => invoke("set_volume", { level });
export const setMuted        = (muted: boolean): Promise<void>       => invoke("set_muted", { muted });
export const getVolume       = (): Promise<VolumeState>              => invoke("get_volume");

export type AudioQuality = "96" | "160" | "320";

export const getAudioQuality = (): Promise<AudioQuality> =>
  invoke("get_audio_quality");

export const setAudioQuality = (quality: AudioQuality): Promise<void> =>
  invoke("set_audio_quality", { quality });

// Audio backend
//
// Spotify only streams audio to Premium, so a free account can't use it at all
// and always plays through YouTube Music. Premium accounts choose between the
// two. Either way Spotify supplies all metadata, artwork and lyrics.
//
// `active` is both the backend in use and the selected value - there's no
// separate stored preference to disagree with it.

export type PlaybackBackend = "spotify" | "youtube";

export interface BackendState {
  active:            PlaybackBackend;
  product:           string | null;
  /** False on a free account, where the Spotify option is disabled. */
  spotify_available: boolean;
}

export const getPlaybackBackend = (): Promise<BackendState> =>
  invoke("get_playback_backend");

export const setPlaybackBackend = (mode: PlaybackBackend): Promise<void> =>
  invoke("set_playback_backend", { mode });

/** A cached Spotify-track -> YouTube-video mapping. */
export interface YtMatch {
  track_id:    string;
  /** null means "resolved, but nothing acceptable was found" - not "unresolved". */
  video_id:    string | null;
  score:       number | null;
  reason:      string | null;
  duration_ms: number | null;
  pinned:      boolean;
  checked_at:  number;
}

export interface YtCandidate {
  video_id:    string;
  title:       string;
  artists:     string[];
  album:       string | null;
  duration_ms: number | null;
  explicit:    boolean;
}

export const getYtMatch = (trackId: string): Promise<YtMatch | null> =>
  invoke("get_yt_match", { trackId });

/**
 * Raw, ungated search results for the manual-override picker.
 * Automatic playback never uses these - it only ever plays a gated match.
 */
export const searchYtCandidates = (trackId: string): Promise<YtCandidate[]> =>
  invoke("search_yt_candidates", { trackId });

/** Pin a user-chosen video. Survives re-resolution. */
export const pinYtMatch = (trackId: string, videoId: string): Promise<void> =>
  invoke("pin_yt_match", { trackId, videoId });

/** Forget a mapping so the next play re-resolves it. */
export const forgetYtMatch = (trackId: string): Promise<void> =>
  invoke("forget_yt_match", { trackId });

export const getAudioCacheLimit = (): Promise<number> =>
  invoke("get_audio_cache_limit");

export const setAudioCacheLimit = (limitMb: number): Promise<void> =>
  invoke("set_audio_cache_limit", { limitMb });

// pitch-preserving speed for whatever is playing (podcasts). 0.5 - 3
export const setPlaybackSpeed = (speed: number): Promise<void> =>
  invoke("set_playback_speed", { speed });
