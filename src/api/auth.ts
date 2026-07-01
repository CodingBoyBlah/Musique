import { invoke } from "@tauri-apps/api/core";
import type { AuthStatus } from "../types/ipc";
import type { Profile } from "../types/spotify";

export const startLogin    = (): Promise<AuthStatus> => invoke("start_login");
export const logout        = (): Promise<void>       => invoke("logout");
export const getAuthStatus = (): Promise<AuthStatus> => invoke("get_auth_status");
export const getProfile    = (): Promise<Profile>    => invoke("get_profile");
