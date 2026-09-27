import { invoke } from "@tauri-apps/api/core";
import type { AlbumItem, ArtistItem, TrackItem } from "../types/spotify";

export type StatsRange = "week" | "month" | "year" | "all";

export interface Ranked<T> {
  item: T;
  plays: number;
}

export interface ListeningStats {
  range: StatsRange;
  total_ms: number;
  plays: number;
  distinct_tracks: number;
  distinct_artists: number;
  current_streak: number;
  longest_streak: number;
  // [weekday 0=sun][hour], local time
  heatmap: number[][];
  top_tracks: Ranked<TrackItem>[];
  top_artists: Ranked<ArtistItem>[];
  top_albums: Ranked<AlbumItem>[];
  top_genres: { genre: string; share: number }[];
  on_repeat: Ranked<TrackItem>[];
  forgotten: Ranked<TrackItem>[];
  discoveries: Ranked<TrackItem>[];
  biggest_day: [string, number] | null;
}

// computed locally from what you've played in musique - no network
export const getListeningStats = (range: StatsRange): Promise<ListeningStats> =>
  invoke("get_listening_stats", { range });
