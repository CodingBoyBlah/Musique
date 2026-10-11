import { Channel, invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "../types/ipc";
import type { Profile } from "../types/spotify";

export interface LoginLinks { ncspot: string; desktop: string }

export const startLogin = (onLinks: (links: LoginLinks) => void, openBrowser = true): Promise<AuthStatus> => {
  const channel = new Channel<LoginLinks>();
  channel.onmessage = onLinks;
  return invoke("start_login", { onLinks: channel, openBrowser });
};
export const logout        = (): Promise<void>       => invoke("logout");
export const getAuthStatus = (): Promise<AuthStatus> => invoke("get_auth_status");
export const getProfile    = (): Promise<Profile>    => invoke("get_profile");
