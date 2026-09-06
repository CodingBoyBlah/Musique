import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  startLogin,
  logout as apiLogout,
  getAuthStatus,
} from "../api/auth";
import { warmupPlayback } from "../api/playback";
import { syncLibrary } from "../api/library";
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
  });

  const { mutate: logout, isPending: loggingOut } = useMutation({
    mutationFn: apiLogout,
    onSuccess: () => {
      store.clear();
      usePlayerStore.getState().clear();
      useQueueStore.getState().clearAll();
      usePinsStore.getState().clear();
      useSpeedDialStore.getState().clear();
      qc.clear();
      qc.setQueryData(["auth-status"], LOGGED_OUT);
      try {
        localStorage.clear();
      } catch (err) {
        console.error("[auth] failed to clear localStorage:", err);
      }
    },
  });

  return {
    ...store,
    isLoading,
    login,
    logout,
    loggingIn,
    loggingOut,
  };
}
