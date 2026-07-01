import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

export interface LyricWord {
  time_ms: number; // start ts
  end_ms:  number; // end
  text:    string;
}

export interface LyricLine {
  time_ms: number;
  text:    string;
  words:   LyricWord[]; // per word timings, empty for line-level sources {TODO only fetch if word_level=true
  bg?:     LyricLine | null; // background vocals line TODO- fix this 
}

export interface Lyrics {
  track_id:     string;
  lines:        LyricLine[];   // synced lines (empty if none)
  plain:        string | null; // unsynced fallback
  synced:       boolean;
  word_level:   boolean;       // lines carry real word timings
  instrumental: boolean;
  source:       string;        // netease/lrclib/none
  found:        boolean;
}

// grab synced lyrics. cached in sqlite so repeat calls are instant (cached queue seperagely) TODO- DONE
// force=true skips the cache and refetches
export function getLyrics(track: TrackItem, force = false): Promise<Lyrics> {
  return invoke<Lyrics>("get_lyrics", {
    trackId:    track.id,
    name:       track.name,
    artist:     track.artists[0]?.name ?? "",
    album:      track.album?.name ?? null,
    durationMs: track.duration_ms,
    force,
  });
}
