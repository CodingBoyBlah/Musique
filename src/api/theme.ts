import { invoke } from "@tauri-apps/api/core";

export const getWallpaperDataUrl = (): Promise<string | null> =>
  invoke("get_wallpaper_data_url");


export const getSystemAccent = (): Promise<string | null> =>
  invoke("get_system_accent");
