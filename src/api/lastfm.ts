import { invoke } from "@tauri-apps/api/core";

export interface LastfmStatus {
  configured: boolean;       // api key + secret saved (no lsos)
  connected:  boolean;       // session key present ( then scrobbling active)
  username:   string | null;
}

export const lastfmStatus = (): Promise<LastfmStatus> =>
  invoke("lastfm_status");

export const lastfmSaveApi = (apiKey: string, apiSecret: string): Promise<void> =>
  invoke("lastfm_save_api", { apiKey, apiSecret });

// opens the authorzie page in the browser, returns the request token (SAVED)
export const lastfmStartAuth = (): Promise<string> =>
  invoke("lastfm_start_auth");

// polls auth.getSession until the user authorizes (or it times out) TODO - DONE
export const lastfmFinishAuth = (token: string): Promise<LastfmStatus> =>
  invoke("lastfm_finish_auth", { token });

export const lastfmDisconnect = (): Promise<void> =>
  invoke("lastfm_disconnect");

export const lastfmClear = (): Promise<void> =>
  invoke("lastfm_clear");

export function lastfmNowPlaying(artist: string, track: string, album: string): void {
  invoke("lastfm_now_playing", { artist, track, album }).catch(() => {});
}

export function lastfmScrobble(artist: string, track: string, album: string, timestamp: number): void {
  invoke("lastfm_scrobble", { artist, track, album, timestamp }).catch(() => {});
}
