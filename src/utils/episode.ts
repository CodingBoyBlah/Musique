import type { TrackItem } from "../types/spotify";
import type { EpisodeItem } from "../types/podcast";

/* podcast episodes travel through the player as TrackItems whose id is the
full "spotify:episode:<id>" uri - that's what tells the backend to load an
episode instead of a track, and what lets everything track-only (lyrics,
likes, scrobbling, the taste engine) step aside. */
export const EPISODE_PREFIX = "spotify:episode:";

export function isEpisodeId(id: string | null | undefined): boolean {
  return !!id && id.startsWith(EPISODE_PREFIX);
}

export function episodeToTrack(ep: EpisodeItem): TrackItem {
  const showId = ep.show_id ?? "";
  return {
    id: `${EPISODE_PREFIX}${ep.id}`,
    name: ep.name,
    duration_ms: ep.duration_ms,
    explicit: ep.explicit,
    artists: [{ id: showId, name: ep.publisher || ep.show_name || "Podcast", image_url: null }],
    album: {
      id: showId,
      name: ep.show_name ?? "Podcast",
      album_type: "show",
      image_url: ep.image_url,
      release_date: ep.release_date,
      artists: [],
    },
  };
}

// "1 hr 5 min" / "42 min" / "3 min left"
export function episodeLength(ms: number): string {
  const mins = Math.max(1, Math.round(ms / 60000));
  if (mins < 60) return `${mins} min`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h} hr ${m} min` : `${h} hr`;
}

export function episodeProgress(ep: EpisodeItem): { fraction: number; label: string } | null {
  if (ep.fully_played) return { fraction: 1, label: "Played" };
  const pos = ep.resume_position_ms ?? 0;
  if (pos <= 0 || ep.duration_ms <= 0) return null;
  return {
    fraction: Math.min(1, pos / ep.duration_ms),
    label: `${episodeLength(Math.max(0, ep.duration_ms - pos))} left`,
  };
}
