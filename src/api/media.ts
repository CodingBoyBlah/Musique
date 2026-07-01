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

export function requestNotificationPermission(): void {
  if ("Notification" in window && Notification.permission === "default") {
    Notification.requestPermission().catch(() => {});
  }
}

export function showTrackNotification(track: TrackItem): void {
  if (!("Notification" in window) || Notification.permission !== "granted") return;
  const opts: NotificationOptions = {
    body:   track.artists.map((a) => a.name).join(", "),
    silent: true,
  };
  if (track.album?.image_url) opts.icon = track.album.image_url;
  new Notification(track.name, opts);
}
