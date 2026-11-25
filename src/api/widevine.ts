import { Channel, invoke } from "@tauri-apps/api/core";

export interface WidevineStatus {
  installed: boolean;
  version:   string | null;
  path:      string | null;
  supported: boolean;
}

export interface WidevineProgress {
  done:  number;
  total: number | null;
}

export const widevineStatus = (): Promise<WidevineStatus> => invoke("widevine_status");

/** Google's terms for the Widevine module, shown before it is installed. */
export const widevineLicenseText = (): Promise<string> => invoke("widevine_license_text");

export const widevineInstall = (onProgress: (p: WidevineProgress) => void): Promise<WidevineStatus> => {
  const channel = new Channel<WidevineProgress>();
  channel.onmessage = onProgress;
  return invoke("widevine_install", { onProgress: channel });
};

export const widevineUninstall = (): Promise<void> => invoke("widevine_uninstall");
