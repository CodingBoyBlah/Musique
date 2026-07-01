import { invoke } from "@tauri-apps/api/core";

export interface VolumeState { level: number; muted: boolean; }

export const warmupPlayback  = (): Promise<void>                     => invoke("warmup_playback");
export const playTrack       = (id: string): Promise<void>           => invoke("play_track", { id });
export const retryPlayTrack  = (id: string): Promise<void>           => invoke("retry_play_track", { id });
export const pausePlayback   = (): Promise<void>                     => invoke("pause_playback");
export const resumePlayback  = (): Promise<void>                     => invoke("resume_playback");
export const resumeOrPlay    = (id: string, positionMs: number): Promise<void> =>
  invoke("resume_or_play", { id, positionMs });
export const stopPlayback    = (): Promise<void>                     => invoke("stop_playback");
export const seekPlayback    = (positionMs: number): Promise<void>   => invoke("seek_playback", { positionMs });
export const preloadTrack    = (id: string): Promise<void>           => invoke("preload_track", { id });
export const setVolume       = (level: number): Promise<void>        => invoke("set_volume", { level });
export const setMuted        = (muted: boolean): Promise<void>       => invoke("set_muted", { muted });
export const getVolume       = (): Promise<VolumeState>              => invoke("get_volume");
