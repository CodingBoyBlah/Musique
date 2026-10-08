import type { QueryClient } from "@tanstack/react-query";
import { startLogin, logout as apiLogout } from "../api/auth";
import { syncLibrary } from "../api/library";
import { useAuthStore } from "../store/auth.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import { useJamStore } from "../store/jam.store";
import type { AuthStatus } from "../types/ipc";

export const LOGGED_OUT: AuthStatus = {
  logged_in: false, user_id: null, display_name: null,
  email: null, product: null, image_url: null,
};

export function currentAuthRequest(generation: number) {
  const s = useAuthStore.getState();
  return s.generation === generation && s.phase === "idle";
}

function clearLocalAccountState() {
  useAuthStore.getState().clear();
  usePlayerStore.getState().clear();
  useQueueStore.getState().clearAll();
  usePinsStore.getState().clear();
  useSpeedDialStore.getState().clear();
  useJamStore.setState({ session: null, connect: null, meta: {}, held: false, localActionAt: 0 });
  for (const key of ["spotify-player", "spotify-queue", "spotify-pins", "musique-speed-dial-v2"]) {
    try { localStorage.removeItem(key); }
    catch (err) { console.error(`[auth] failed to clear ${key}:`, err); }
  }
}

export async function loginSession(qc: QueryClient) {
  const generation = useAuthStore.getState().begin("login");
  if (generation === null) return;
  const previousId = useAuthStore.getState().userId;
  try {
    await qc.cancelQueries();
    const data = await startLogin();
    if (useAuthStore.getState().generation !== generation) return;
    if (previousId !== data.user_id) clearLocalAccountState();
    qc.clear();
    useAuthStore.getState().setFromStatus(data);
    qc.setQueryData(["auth-status"], data);
    useAuthStore.getState().finish(generation);
    syncLibrary().catch((err) => console.error("[auth] post-login sync:", err))
      .finally(() => { if (currentAuthRequest(generation)) void qc.invalidateQueries(); });
  } catch (err) {
    if (useAuthStore.getState().generation !== generation) return;
    useAuthStore.getState().finish(generation);
    void qc.invalidateQueries({ queryKey: ["auth-status"] });
    throw err;
  }
}

export async function logoutSession(qc: QueryClient) {
  const generation = useAuthStore.getState().begin("logout");
  if (generation === null) return;
  clearLocalAccountState();
  const cancelled = qc.cancelQueries();
  qc.clear();
  qc.setQueryData(["auth-status"], LOGGED_OUT);
  try {
    await Promise.all([apiLogout(), cancelled]);
  } finally {
    qc.clear();
    qc.setQueryData(["auth-status"], LOGGED_OUT);
    useAuthStore.getState().finish(generation);
    void qc.invalidateQueries({ queryKey: ["auth-status"] });
  }
}
