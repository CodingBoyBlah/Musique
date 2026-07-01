import { invoke } from "@tauri-apps/api/core";

export const getSetting = (key: string): Promise<string | null> =>
  invoke<string | null>("get_setting", { key });

export const setSetting = (key: string, value: string): Promise<void> =>
  invoke<void>("set_setting", { key, value });

export const getAllSettings = (): Promise<Record<string, string>> =>
  invoke<Record<string, string>>("get_all_settings");
