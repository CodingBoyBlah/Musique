import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../store/player.store";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";

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
  toast(errMsg(e));
  throw e;
};

export const warmupPlayback  = (): Promise<void>                     => invoke("warmup_playback");
export const playTrack       = (id: string): Promise<void> => {
  const store = usePlayerStore.getState();
  store.setIsRemotePlayback(false);
  store.setTargetState("playing");
  store.setPlaying(true);
  store.setLastPlayingAt(Date.now());
  return invoke<void>("play_track", { id }).catch(failPlayback);
};
export const retryPlayTrack  = (id: string): Promise<void>           => invoke("retry_play_track", { id });
export const pausePlayback   = (): Promise<void>                     => invoke("pause_playback");
export const resumePlayback  = (): Promise<void>                     => invoke("resume_playback");
export const resumeOrPlay    = (id: string, positionMs: number): Promise<void> => {
  const store = usePlayerStore.getState();
  const safePos = Math.max(0, Math.floor(positionMs || 0));
  store.setIsRemotePlayback(false);
  store.setTargetState("playing");
  store.setPlaying(true);
  store.setPosition(safePos);
  store.setLastPlayingAt(Date.now());
  return invoke<void>("resume_or_play", { id, positionMs: safePos }).catch(failPlayback);
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
