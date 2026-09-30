import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

export interface SpotifyDevice {
  id: string | null;
  is_active: boolean;
  is_private_session: boolean;
  is_restricted: boolean;
  name: string;
  type: string;
  volume_percent: number | null;
  supports_volume: boolean;
}

export interface DevicesPayload {
  devices: SpotifyDevice[];
  musique_device_id: string;
}

export interface RemotePlaybackState {
  device: SpotifyDevice;
  is_playing: boolean;
  progress_ms: number | null;
  timestamp?: number | null;
  shuffle_state: boolean;
  repeat_state: string;
  track: TrackItem | null;
}

export const getDevices = (): Promise<DevicesPayload> =>
  invoke("get_devices");

export const getPlaybackState = (): Promise<RemotePlaybackState | null> =>
  invoke("get_playback_state");

export const transferPlayback = (deviceId: string, play = true): Promise<void> =>
  invoke("transfer_playback", { deviceId, play });

export const remotePlayTrack = (
  deviceId: string,
  trackId?: string | null,
  positionMs?: number
): Promise<void> =>
  invoke("remote_play_track", {
    deviceId,
    trackId: trackId ?? undefined,
    positionMs: positionMs != null ? Math.round(positionMs) : undefined,
  });

export const remotePlay = (deviceId?: string): Promise<void> =>
  invoke("remote_play", { deviceId });

export const remotePause = (deviceId?: string): Promise<void> =>
  invoke("remote_pause", { deviceId });

export const remoteNext = (deviceId?: string): Promise<void> =>
  invoke("remote_next", { deviceId });

export const remotePrevious = (deviceId?: string): Promise<void> =>
  invoke("remote_previous", { deviceId });

export const remoteSeek = (positionMs: number): Promise<void> =>
  invoke("remote_seek", { positionMs: Math.round(positionMs) });

export const remoteSetVolume = (volumePercent: number): Promise<void> =>
  invoke("remote_set_volume", { volumePercent: Math.round(volumePercent) });

export const getMusiqueDeviceId = (): Promise<string> =>
  invoke("get_musique_device_id");

export interface RemoteQueue {
  currently_playing: TrackItem | null;
  queue: TrackItem[];
}

// spotify's own queue on whichever device is playing right now
export const getRemoteQueue = (): Promise<RemoteQueue> =>
  invoke("get_remote_queue");

export const remoteSetShuffle = (state: boolean): Promise<void> =>
  invoke("remote_set_shuffle", { state });

export type RemoteRepeat = "off" | "context" | "track";

export const remoteSetRepeat = (state: RemoteRepeat): Promise<void> =>
  invoke("remote_set_repeat", { state });

// a bare track id, or a full uri (episodes)
export const remoteAddToQueue = (id: string): Promise<void> =>
  invoke("remote_add_to_queue", { id });

export const remotePlayContext = (opts: {
  contextUri: string;
  deviceId?: string | null;
  offsetUri?: string | null;
  offsetPosition?: number | null;
  positionMs?: number | null;
}): Promise<void> =>
  invoke("remote_play_context", {
    deviceId:       opts.deviceId ?? null,
    contextUri:     opts.contextUri,
    offsetUri:      opts.offsetUri ?? null,
    offsetPosition: opts.offsetPosition ?? null,
    positionMs:     opts.positionMs != null ? Math.round(opts.positionMs) : null,
  });
