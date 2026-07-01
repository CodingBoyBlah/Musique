import { invoke } from "@tauri-apps/api/core";

// resolve a spotify URL to its odesli (song.link) universal page URL TODO  album link DONE
export const resolveOdesli = (url: string): Promise<string> =>
  invoke<string>("resolve_odesli", { url });
