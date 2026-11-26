import { Channel, invoke } from "@tauri-apps/api/core";
import type { Accounts, AuthStatus, Provider } from "../types/ipc";
import type { Profile } from "../types/spotify";

export interface LoginLinks { ncspot: string; desktop: string }

export const startLogin = (onLinks: (links: LoginLinks) => void, openBrowser = true): Promise<AuthStatus> => {
  const channel = new Channel<LoginLinks>();
  channel.onmessage = onLinks;
  return invoke("start_login", { onLinks: channel, openBrowser });
};
/** Signs out of one service (the shown one by default). The other, if signed in, takes over. */
export const logout        = (provider?: Provider): Promise<AuthStatus> => invoke("logout", { provider: provider ?? null });
export const getAccounts   = (): Promise<Accounts>   => invoke("get_accounts");
/** Shows the other signed-in service; both stay signed in. */
export const switchProvider = (provider: Provider): Promise<AuthStatus> => invoke("switch_provider", { provider });
export const getAuthStatus = (): Promise<AuthStatus> => invoke("get_auth_status");
export const getProfile    = (): Promise<Profile>    => invoke("get_profile");
