import type { QueryClient } from "@tanstack/react-query";
import { startLogin, logout as apiLogout, switchProvider } from "../api/auth";
import { appleSignIn } from "../api/apple";
import { syncLibrary } from "../api/library";
import { warmupPlayback } from "../api/playback";
import { useAuthStore } from "../store/auth.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import { useJamStore } from "../store/jam.store";
import type { Accounts, AuthStatus, Provider } from "../types/ipc";

export const LOGGED_OUT: AuthStatus = {
  logged_in: false, user_id: null, display_name: null,
  email: null, product: null, image_url: null,
};

export const ACCOUNTS_KEY = ["accounts"] as const;

export function currentAuthRequest(generation: number) {
  const s = useAuthStore.getState();
  return s.generation === generation && s.phase === "idle";
}

/* What each account keeps in this browser: its queue, pins and speed dial
   whole, and what the player bar last showed. Volume and lyric settings in
   the player store belong to the device and stay put. */
const ACCOUNT_STORAGE = ["spotify-queue", "spotify-pins", "musique-speed-dial-v2"];
const stashKey = (provider: Provider) => `musique-account-${provider}`;

interface LocalStash {
  storage: Record<string, string>;
  player: { currentTrack: unknown; currentId: string | null; durationMs: number };
}

function snapshotLocal(): LocalStash {
  const storage: Record<string, string> = {};
  for (const key of ACCOUNT_STORAGE) {
    try {
      const value = localStorage.getItem(key);
      if (value !== null) storage[key] = value;
    } catch { /* storage unavailable: nothing to keep */ }
  }
  const { currentTrack, currentId, durationMs } = usePlayerStore.getState();
  return { storage, player: { currentTrack, currentId, durationMs } };
}

function keepLocal(provider: Provider, stash: LocalStash) {
  try { localStorage.setItem(stashKey(provider), JSON.stringify(stash)); }
  catch (err) { console.error(`[auth] could not keep ${provider}'s local state:`, err); }
}

function dropLocal(provider: Provider) {
  try { localStorage.removeItem(stashKey(provider)); } catch { /* nothing kept */ }
}

function emptyLocal() {
  usePlayerStore.getState().clear();
  useQueueStore.getState().clearAll();
  usePinsStore.getState().clear();
  useSpeedDialStore.getState().clear();
  useJamStore.setState({ session: null, connect: null, meta: {}, held: false, localActionAt: 0 });
  for (const key of ["spotify-player", ...ACCOUNT_STORAGE]) {
    try { localStorage.removeItem(key); }
    catch (err) { console.error(`[auth] failed to clear ${key}:`, err); }
  }
}

/* Empties the outgoing account's local state and brings back what `provider`
   left when it was last shown. */
function restoreLocal(provider: Provider) {
  usePlayerStore.getState().clear();
  useQueueStore.getState().clearAll();
  usePinsStore.getState().clear();
  useSpeedDialStore.getState().clear();
  useJamStore.setState({ session: null, connect: null, meta: {}, held: false, localActionAt: 0 });
  let stash: LocalStash | null = null;
  try {
    const raw = localStorage.getItem(stashKey(provider));
    localStorage.removeItem(stashKey(provider));
    stash = raw ? (JSON.parse(raw) as LocalStash) : null;
  } catch { stash = null; }
  for (const key of ACCOUNT_STORAGE) {
    try {
      const value = stash?.storage?.[key];
      if (value != null) localStorage.setItem(key, value);
      else localStorage.removeItem(key);
    } catch { /* storage unavailable */ }
  }
  void useQueueStore.persist.rehydrate();
  void usePinsStore.persist.rehydrate();
  void useSpeedDialStore.persist.rehydrate();
  const player = stash?.player;
  if (player?.currentId && player.currentTrack) {
    usePlayerStore.setState({
      currentTrack: player.currentTrack as ReturnType<typeof usePlayerStore.getState>["currentTrack"],
      currentId: player.currentId,
      durationMs: player.durationMs ?? 0,
      positionMs: 0,
    });
  }
}

function clearLocalAccountState() {
  useAuthStore.getState().clear();
  emptyLocal();
  dropLocal("spotify");
  dropLocal("apple");
}

/* Everything cached for the outgoing service goes; what is on screen refetches
   from the incoming one's library straight away. */
function resetCache(qc: QueryClient, status: AuthStatus) {
  qc.removeQueries({ type: "inactive" });
  qc.setQueryData(["auth-status"], status);
  void qc.resetQueries({ type: "active", predicate: (q) => q.queryKey[0] !== "auth-status" });
}

/* The library sync tops up what the switch brought back; then every view
   reads the result. */
function afterSwitch(qc: QueryClient, generation: number | null) {
  warmupPlayback().catch(() => {});
  void qc.invalidateQueries({ queryKey: ACCOUNTS_KEY });
  syncLibrary().catch((err) => console.error("[auth] post-switch sync:", err))
    .finally(() => { if (generation === null || currentAuthRequest(generation)) void qc.invalidateQueries(); });
}

export async function loginSession(qc: QueryClient, openBrowser = true) {
  return signIn(qc, (generation) => startLogin((links) => {
    useAuthStore.getState().setLoginLinks(generation, links);
  }, openBrowser));
}

/** Signs in to Apple Music through its own login page, in a private window of the app. */
export async function appleLoginSession(qc: QueryClient) {
  return signIn(qc, () => appleSignIn());
}

async function signIn(qc: QueryClient, start: (generation: number) => Promise<AuthStatus>) {
  const generation = useAuthStore.getState().begin("login");
  if (generation === null) return;
  const before = useAuthStore.getState();
  const previousId = before.userId;
  const previousProvider = before.provider;
  const wasSignedIn = before.loggedIn;
  // taken before the sign-in starts: the backend stops the old service's audio when it switches
  const outgoing = snapshotLocal();
  try {
    await qc.cancelQueries();
    const data = await start(generation);
    if (useAuthStore.getState().generation !== generation) return;
    const provider = data.provider ?? "spotify";
    if (wasSignedIn && provider !== previousProvider) {
      // the other service joined: the one that was shown keeps its state for later
      keepLocal(previousProvider, outgoing);
      restoreLocal(provider);
    } else if (previousId !== data.user_id) {
      clearLocalAccountState();
    }
    qc.clear();
    useAuthStore.getState().setFromStatus(data);
    qc.setQueryData(["auth-status"], data);
    useAuthStore.getState().finish(generation);
    if (wasSignedIn && provider !== previousProvider) warmupPlayback().catch(() => {});
    syncLibrary().catch((err) => console.error("[auth] post-login sync:", err))
      .finally(() => { if (currentAuthRequest(generation)) void qc.invalidateQueries(); });
  } catch (err) {
    if (useAuthStore.getState().generation !== generation) return;
    useAuthStore.getState().finish(generation);
    void qc.invalidateQueries({ queryKey: ["auth-status"] });
    void qc.invalidateQueries({ queryKey: ACCOUNTS_KEY });
    throw err;
  }
}

/** Shows the other signed-in service. Both stay signed in; library, home,
 *  recommendations, queue and player all become that service's. */
export async function switchProviderSession(qc: QueryClient, target: Provider) {
  const from = useAuthStore.getState().provider;
  if (from === target) return;
  const generation = useAuthStore.getState().begin("login");
  if (generation === null) return;
  const outgoing = snapshotLocal();
  try {
    await qc.cancelQueries();
    const status = await switchProvider(target);
    if (useAuthStore.getState().generation !== generation) return;
    keepLocal(from, outgoing);
    restoreLocal(status.provider ?? target);
    useAuthStore.getState().setFromStatus(status);
    resetCache(qc, status);
  } finally {
    useAuthStore.getState().finish(generation);
  }
  afterSwitch(qc, generation);
}

/** The backend moved to the other service by itself (the shown one lost its
 *  sign-in): show that one's state. The outgoing one has nothing to keep. */
export function adoptProvider(qc: QueryClient, status: AuthStatus) {
  const from = useAuthStore.getState().provider;
  const to = status.provider ?? "spotify";
  if (from === to) return;
  dropLocal(from);
  restoreLocal(to);
  useAuthStore.getState().setFromStatus(status);
  resetCache(qc, status);
  afterSwitch(qc, null);
}

/* Whether `provider` is signed in, from the last accounts answer the switcher
   fetched. Read synchronously so a sign-out clears the screen at once. */
function knownSignedIn(qc: QueryClient, provider: Provider): boolean {
  let newest: { at: number; data: Accounts } | null = null;
  for (const query of qc.getQueryCache().findAll({ queryKey: ACCOUNTS_KEY })) {
    const data = query.state.data as Accounts | undefined;
    if (data && (!newest || query.state.dataUpdatedAt > newest.at)) newest = { at: query.state.dataUpdatedAt, data };
  }
  return !!newest?.data[provider].logged_in;
}

/** Signs out of `provider` (the shown service by default). When the other
 *  service is signed in too, the app carries on with it. */
export async function logoutSession(qc: QueryClient, provider?: Provider) {
  const shown = useAuthStore.getState().provider;
  const target = provider ?? shown;
  const other: Provider = target === "spotify" ? "apple" : "spotify";

  const generation = useAuthStore.getState().begin("logout");
  if (generation === null) return;

  if (target !== shown || knownSignedIn(qc, other)) {
    // Someone stays signed in: no signed-out screen in between.
    try {
      await qc.cancelQueries();
      const status = await apiLogout(target);
      dropLocal(target);
      if (target === shown && useAuthStore.getState().generation === generation) {
        if (status.logged_in) {
          restoreLocal(status.provider ?? other);
          useAuthStore.getState().setFromStatus(status);
        } else {
          clearLocalAccountState();
        }
        resetCache(qc, status);
      }
    } finally {
      useAuthStore.getState().finish(generation);
      void qc.invalidateQueries({ queryKey: ACCOUNTS_KEY });
    }
    if (target === shown) afterSwitch(qc, generation);
    return;
  }

  clearLocalAccountState();
  const cancelled = qc.cancelQueries();
  qc.clear();
  qc.setQueryData(["auth-status"], LOGGED_OUT);
  let after: AuthStatus = LOGGED_OUT;
  try {
    const [status] = await Promise.all([apiLogout(target), cancelled]);
    // the other service turned out to be signed in after all: carry on with it
    if (status?.logged_in) {
      after = status;
      restoreLocal(status.provider ?? other);
      useAuthStore.getState().setFromStatus(status);
    }
  } finally {
    qc.clear();
    qc.setQueryData(["auth-status"], after);
    useAuthStore.getState().finish(generation);
    void qc.invalidateQueries({ queryKey: ["auth-status"] });
  }
  if (after.logged_in) afterSwitch(qc, generation);
}
