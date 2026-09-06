import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

export function updateNowPlaying(track: TrackItem, positionMs = 0): void {
  invoke("update_now_playing", {
    title:      track.name,
    artist:     track.artists.map((a) => a.name).join(", "),
    album:      track.album?.name ?? "",
    coverUrl:   track.album?.image_url ?? null,
    trackUrl:   track.id ? `https://open.spotify.com/track/${track.id}` : null,
    durationMs: track.duration_ms,
    positionMs,
  }).catch(() => {});
}

export function setDiscordEnabled(enabled: boolean): void {
  invoke("set_discord_enabled", { enabled }).catch(() => {});
}

import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";

export async function requestNotificationPermission(): Promise<boolean> {
  try {
    let granted = await isPermissionGranted();
    if (!granted) {
      const permission = await requestPermission();
      granted = permission === "granted";
    }
    return granted;
  } catch (e) {
    console.warn("[notification] permission check/request failed:", e);
    return false;
  }
}

export async function showTrackNotification(track: TrackItem): Promise<void> {
  const title = track.name;
  const artist = track.artists.map((a) => a.name).join(", ");
  const album = track.album?.name;
  const body = album ? `${artist} • ${album}` : artist;
  const icon = track.album?.image_url ?? null;

  try {
    const hasPermission = await requestNotificationPermission();
    if (!hasPermission) return;
  } catch (err) {
    console.warn("[notification] permission check failed:", err);
  }

  try {
    await invoke("show_playback_notification", {
      title,
      body,
      icon,
    });
  } catch (err) {
    console.error("[notification] backend notification error:", err);
  }
}
