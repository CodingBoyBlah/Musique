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

export interface SoloistState {
  enabled: boolean;
  supported: boolean;
  has_key: boolean;
  status: SoloistStatus;
}

export const soloistStatus = () => invoke<SoloistState>("soloist_status");
export const soloistEnable = (key?: string) =>
  invoke<SoloistState>("soloist_enable", { key: key ?? null });
export const soloistDisable = () => invoke<SoloistState>("soloist_disable");
export const soloistForget = () => invoke<SoloistState>("soloist_forget");
