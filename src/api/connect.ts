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
