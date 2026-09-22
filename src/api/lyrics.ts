import { invoke } from "@tauri-apps/api/core";
import type { TrackItem } from "../types/spotify";

/* which voice a line belongs to. providers that ship a structured document
(AMLL TTML, Apple Music) label them; line-level sources never do, so anything
without a role is a lead line. */
export type LineRole = "main" | "bg" | "duet";

export interface LyricWord {
  time_ms: number; // start ts
  end_ms:  number; // end
  text:    string;
}

export interface LyricLine {
  time_ms: number;
  text:    string;
  /* per-word timings. genuinely empty for line-level sources - the renderer
     paces those across the line itself rather than asking for them again. */
  words:   LyricWord[];
  translation?: string | null; // only when the source itself ships one
  roman?:       string | null; // provider romanization (pinyin / romaji)
  role?:        LineRole;      // absent means "main"
  bg?:          LyricLine | null; // backing vocal sung under this line
}

/* another cached source for the same track. already on disk, so switching to
one is instant and works offline. */
export interface Alternate {
  source:     string;
  word_level: boolean;
  synced:     boolean;
  lines:      number;
}

export interface Lyrics {
  track_id:     string;
  lines:        LyricLine[];   // synced lines (empty if none)
  plain:        string | null; // unsynced fallback
  synced:       boolean;
  word_level:   boolean;       // lines carry real word timings
  instrumental: boolean;
  source:       string;        // spotify/musixmatch/netease/amll/qq/kugou/lrclib/none
  found:        boolean;
  offset_ms:    number;        // shift applied to line up with the sync reference
  alternates:   Alternate[];   // other cached sources, for the switcher
  /* a word-by-word candidate may still be racing in the background and will
     arrive over `lyrics:upgraded`. informational - the swap is silent. */
  upgrading:       boolean;
  has_translation: boolean;
  has_roman:       boolean;
}

/* grab synced lyrics. cached in sqlite so repeat calls are instant and work
offline. returns the line-level result immediately - one request on the
critical path - and upgrades to word-by-word later over the event.
force=true skips the cache and refetches. */
export function getLyrics(track: TrackItem, force = false): Promise<Lyrics> {
  return invoke<Lyrics>("get_lyrics", {
    trackId:    track.id,
    name:       track.name,
    artist:     track.artists[0]?.name ?? "",
    album:      track.album?.name ?? null,
    durationMs: track.duration_ms,
    // the only identifier that matches across providers - without it matching
    // falls back to fuzzy title/artist, which is where wrong lyrics come from
    isrc:       track.external_ids?.isrc ?? null,
    force,
  });
}
