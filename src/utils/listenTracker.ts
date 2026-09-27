import { recordListenEvent } from "../api/spotify";
import type { TrackItem } from "../types/spotify";
import { isEpisodeId } from "./episode";

// Central place for feeding the recommendation engine's behavioural signals.
// We only ever send play / complete / skip — never block playback on it.

let started: { id: string; at: number; durationMs: number } | null = null;
let lastPlayedId: string | null = null;

export function trackStarted(
  track: TrackItem | null,
  contextType: string | null = null,
  contextId: string | null = null,
): void {
  if (!track || lastPlayedId === track.id || isEpisodeId(track.id)) return;
  lastPlayedId = track.id;
  started = { id: track.id, at: Date.now(), durationMs: track.duration_ms };
  recordListenEvent(
    track.id,
    "play",
    0,
    track.duration_ms,
    contextType,
    contextId,
  ).catch(() => {});
}

export function trackCompleted(track: TrackItem | null): void {
  if (!track || isEpisodeId(track.id)) return;
  const duration = track.duration_ms || started?.durationMs || 0;
  recordListenEvent(track.id, "complete", duration, duration).catch(() => {});
  if (started?.id === track.id) started = null;
  lastPlayedId = null;
}

export function trackSkipped(
  track: TrackItem | null,
  positionMs: number,
  contextType: string | null = null,
  contextId: string | null = null,
): void {
  if (!track || isEpisodeId(track.id)) return;
  // if it was essentially over, treat it as a completion instead
  const duration = track.duration_ms || started?.durationMs || 0;
  const played =
    positionMs > 0
      ? positionMs
      : started?.id === track.id
        ? Date.now() - started.at
        : 0;

  if (duration > 0 && played >= duration * 0.9) {
    trackCompleted(track);
    return;
  }

  recordListenEvent(track.id, "skip", played, duration, contextType, contextId).catch(
    () => {},
  );
  if (started?.id === track.id) started = null;
  lastPlayedId = null;
}
