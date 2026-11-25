import { useAuthStore } from "../store/auth.store";

/**
 * Whether the signed-in account is Apple Music rather than Spotify.
 *
 * Spotify-only surfaces (Connect devices, Jams, friend activity, podcasts,
 * canvases, Spotify credits) hide themselves on this, since Apple Music has no
 * counterpart for them.
 */
export const useIsApple = (): boolean => useAuthStore((s) => s.provider === "apple");

/** Non-hook form, for event handlers and plain modules. */
export const isApple = (): boolean => useAuthStore.getState().provider === "apple";
