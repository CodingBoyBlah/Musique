import { invoke } from "@tauri-apps/api/core";
import { usePlayerStore } from "../store/player.store";

export interface VolumeState { level: number; muted: boolean; }

export const warmupPlayback  = (): Promise<void>                     => invoke("warmup_playback");
export const playTrack       = (id: string): Promise<void> => {
  const store = usePlayerStore.getState();
  store.setIsRemotePlayback(false);
  store.setTargetState("playing");
  store.setPlaying(true);
  store.setLastPlayingAt(Date.now());
  return invoke("play_track", { id });
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
  return invoke("resume_or_play", { id, positionMs: safePos });
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

export const getAudioCacheLimit = (): Promise<number> =>
  invoke("get_audio_cache_limit");

export const setAudioCacheLimit = (limitMb: number): Promise<void> =>
  invoke("set_audio_cache_limit", { limitMb });
