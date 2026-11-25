import { invoke } from "@tauri-apps/api/core";

/** Where Spotify's Soloist player is (`soloist::Status` in Rust). */
export type SoloistStatus =
  | { state: "off" }
  | { state: "unsupported" }
  | { state: "needs_key" }
  | { state: "downloading"; received: number; total: number }
  | { state: "starting" }
  | { state: "pairing"; name: string; version: string }
  | { state: "ready"; name: string; version: string; expires: number }
  | { state: "failed"; message: string };

/** The streaming quality asked of Soloist. */
export type SoloistQuality = "high" | "very_high" | "lossless";

/** What Soloist is decoding, as its own log reports it. */
export interface SoloistFormat {
  codec: string;
  bitrate: number | null;
  sample_rate: number | null;
}

export interface SoloistState {
  supported: boolean;
  has_key: boolean;
  /** Soloist is the audio source. */
  chosen: boolean;
  status: SoloistStatus;
  quality: SoloistQuality;
  format: SoloistFormat | null;
}

export const soloistStatus = () => invoke<SoloistState>("soloist_status");
/** Save `key` (or use the saved one) and play through Soloist. */
export const soloistEnable = (key?: string) =>
  invoke<SoloistState>("soloist_enable", { key: key ?? null });
export const soloistDisable = () => invoke<SoloistState>("soloist_disable");
export const soloistForget = () => invoke<SoloistState>("soloist_forget");
export const soloistSetQuality = (quality: SoloistQuality) =>
  invoke<SoloistState>("soloist_set_quality", { quality });

/** "Ogg Vorbis · 320 kbps · 44.1 kHz", or "FLAC · 44.1 kHz". */
export function describeFormat(f: SoloistFormat): string {
  const codec =
    f.codec === "vorbis" ? "Ogg Vorbis"
    : f.codec === "flac" ? "FLAC"
    : f.codec === "aac" ? "AAC"
    : f.codec === "mp3" ? "MP3"
    : f.codec.toUpperCase();
  const parts = [codec];
  // a lossless bitrate varies with the music, so it says nothing useful
  if (f.bitrate && f.codec !== "flac") parts.push(`${Math.round(f.bitrate / 1000)} kbps`);
  if (f.sample_rate) parts.push(`${+(f.sample_rate / 1000).toFixed(1)} kHz`);
  return parts.join(" · ");
}
