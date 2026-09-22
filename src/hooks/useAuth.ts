import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  startLogin,
  logout as apiLogout,
  getAuthStatus,
} from "../api/auth";
import { warmupPlayback } from "../api/playback";
import { syncLibrary } from "../api/library";
import { errMsg } from "../lib/err";
import { toast } from "../store/toast.store";
import { useAuthStore } from "../store/auth.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import type { AuthStatus } from "../types/ipc";

const LOGGED_OUT: AuthStatus = {
  logged_in:    false,
  user_id:      null,
  display_name: null,
  email:        null,
  product:      null,
  image_url:    null,
};

/* Persisted state that belongs to the signed-in account, and nothing else.
   `localStorage.clear()` used to run here, which also wiped the theme, the
   sidebar layout and the audio prefs - those describe this machine, not the
   account, and losing them on every sign-out was its own small bug. */
const ACCOUNT_STORAGE_KEYS = [
  "spotify-player",
  "spotify-queue",
  "spotify-pins",
  "musique-speed-dial-v2",
];

/* Wipe every trace of the previous account from the renderer.
   Runs whether or not the backend reported success: a logout that half failed
   still means the user asked to be signed out, and leaving their name, avatar
   and playlists on screen is the worst possible answer. */
function clearLocalAccountState() {
  useAuthStore.getState().clear();
  usePlayerStore.getState().clear();
  useQueueStore.getState().clearAll();
  usePinsStore.getState().clear();
  useSpeedDialStore.getState().clear();

  // after the stores, so their own persist writes cannot put the keys back
  for (const key of ACCOUNT_STORAGE_KEYS) {
    try {
      localStorage.removeItem(key);
    } catch (err) {
      console.error(`[auth] failed to clear ${key}:`, err);
    }
  }
}

export function useAuth() {
  const store = useAuthStore();
  const qc    = useQueryClient();

  const { data: status, isLoading } = useQuery({
    queryKey:           ["auth-status"],
    queryFn:            getAuthStatus,
    staleTime:          5 * 60 * 1_000,
    refetchOnWindowFocus: false,
  });

  useEffect(() => {
    if (status !== undefined) store.setFromStatus(status);
  }, [status]); // eslint-disable-line react-hooks/exhaustive-deps

  const { mutate: login, isPending: loggingIn } = useMutation({
    mutationFn: startLogin,
    onMutate: () => {
      /* Whatever is cached belongs to whoever was signed in before. Dropping it
         up front means a sign-in that lands on a different account never has a
         moment where the old library is on screen under the new name. */
      qc.cancelQueries();
    },
    onSuccess: (data) => {
      qc.clear();
      store.setFromStatus(data);
      qc.setQueryData(["auth-status"], data);
      warmupPlayback().catch((err) => console.error("[auth] warmup playback error:", err));
      syncLibrary()
        .then(() => {
          qc.invalidateQueries();
        })
        .catch((err) => {
          console.error("[auth] post-login sync library error:", err);
          qc.invalidateQueries();
        });
    },
    onError: (err) => {
      console.error("[auth] login failed:", err);
      toast(errMsg(err));
      /* The backend may have got part way (tokens stored, profile call failed),
         so ask it what actually happened rather than assuming either outcome. */
      qc.invalidateQueries({ queryKey: ["auth-status"] });
    },
  });

  const { mutate: logout, isPending: loggingOut } = useMutation({
    mutationFn: async () => {
      /* Stop in-flight queries before the backend starts deleting rows, so
         nothing refetches the old account's data mid-purge. */
      await qc.cancelQueries();
      return apiLogout();
    },
    onError: (err) => {
      console.error("[auth] logout reported an error:", err);
      toast(`Signed out, but some data could not be cleared: ${errMsg(err)}`);
    },
    onSettled: () => {
      clearLocalAccountState();
      qc.clear();
      qc.setQueryData(["auth-status"], LOGGED_OUT);
      /* Confirm against the backend instead of trusting the local reset - if
         anything survived, the UI finds out immediately. */
      qc.invalidateQueries({ queryKey: ["auth-status"] });
    },
  });

  return {
    ...store,
    isLoading,
    /* Guarded so a double-click cannot start two browser authorizations; the
       second would race the first for the redirect port. */
    login:  () => { if (!loggingIn && !loggingOut) login(); },
    logout: () => { if (!loggingOut) logout(); },
    loggingIn,
    loggingOut,
  };
}
