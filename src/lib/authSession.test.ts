import { beforeEach, expect, it, vi } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { startLogin, logout } from "../api/auth";
import { syncLibrary } from "../api/library";
import { currentAuthRequest, loginSession, logoutSession, LOGGED_OUT } from "./authSession";
import { useAuthStore } from "../store/auth.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { useJamStore } from "../store/jam.store";
import type { AuthStatus } from "../types/ipc";
import type { SyncResult } from "../types/library";
import type { TrackItem } from "../types/spotify";

vi.hoisted(() => {
  const storage = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", { configurable: true, value: {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
    removeItem: (key: string) => storage.delete(key), clear: () => storage.clear(),
  } });
});
vi.mock("../api/auth", () => ({ startLogin: vi.fn(), logout: vi.fn() }));
vi.mock("../api/library", () => ({ syncLibrary: vi.fn() }));

const ACCOUNT: AuthStatus = {
  logged_in: true, user_id: "a", display_name: "Alice", email: "a@example.com",
  product: "premium", image_url: "https://example.com/a",
};
const SYNC: SyncResult = {
  liked_count: 0, playlist_count: 0, artist_count: 0, album_count: 0,
  top_track_count: 0, top_artist_count: 0, recent_count: 0, new_release_count: 0,
};
const TRACK: TrackItem = { id: "alice-song", name: "Song", duration_ms: 1000, explicit: false, artists: [], album: null };
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
let qc: QueryClient;
beforeEach(() => {
  vi.resetAllMocks();
  localStorage.clear();
  qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  useAuthStore.setState({ phase: "idle", generation: 0 });
  useAuthStore.getState().setFromStatus(ACCOUNT);
  vi.mocked(syncLibrary).mockResolvedValue(SYNC);
});

it("clears the visible account and player before a slow backend logout finishes", async () => {
  const pending = deferred<void>();
  vi.mocked(logout).mockReturnValue(pending.promise);
  usePlayerStore.setState({ currentId: "old-song", isPlaying: true });
  useJamStore.setState({ meta: { stale: {} as never } });
  localStorage.setItem("theme", "dark");
  qc.setQueryData(["profile", "me"], { name: "Alice" });
  const task = logoutSession(qc);
  expect(useAuthStore.getState()).toMatchObject({ loggedIn: false, imageUrl: null, phase: "logout" });
  expect(usePlayerStore.getState()).toMatchObject({ currentId: null, isPlaying: false });
  expect(useJamStore.getState().meta).toEqual({});
  expect(qc.getQueryData(["profile", "me"])).toBeUndefined();
  expect(qc.getQueryData(["auth-status"])).toEqual(LOGGED_OUT);
  expect(localStorage.getItem("spotify-player")).toBeNull();
  expect(localStorage.getItem("theme")).toBe("dark");
  pending.resolve();
  await task;
  expect(useAuthStore.getState().phase).toBe("idle");
});

it("ignores a login response arriving after logout", async () => {
  const pending = deferred<AuthStatus>();
  vi.mocked(startLogin).mockReturnValue(pending.promise);
  vi.mocked(logout).mockResolvedValue(undefined);
  const loginTask = loginSession(qc);
  await vi.waitFor(() => expect(startLogin).toHaveBeenCalledOnce());
  await logoutSession(qc);
  pending.resolve(ACCOUNT);
  await loginTask;
  expect(useAuthStore.getState().loggedIn).toBe(false);
  expect(qc.getQueryData(["auth-status"])).toEqual(LOGGED_OUT);
  expect(syncLibrary).not.toHaveBeenCalled();
});

it("guards operations across different hook callers and rejects stale profile requests", async () => {
  const pending = deferred<void>();
  vi.mocked(logout).mockReturnValue(pending.promise);
  const generation = useAuthStore.getState().generation;
  const task = logoutSession(qc);
  await Promise.all([logoutSession(qc), loginSession(qc)]);
  expect(logout).toHaveBeenCalledOnce();
  expect(startLogin).not.toHaveBeenCalled();
  expect(currentAuthRequest(generation)).toBe(false);
  pending.resolve();
  await task;
  expect(currentAuthRequest(generation)).toBe(false);
});

it("clears persisted player and queue state when switching accounts", async () => {
  const other = { ...ACCOUNT, user_id: "b", display_name: "Bob", image_url: null };
  vi.mocked(startLogin).mockResolvedValue(other);
  usePlayerStore.setState({ currentId: "alice-song" });
  useQueueStore.setState({ queue: [TRACK] });
  await loginSession(qc);
  expect(useAuthStore.getState()).toMatchObject({ userId: "b", displayName: "Bob", imageUrl: null });
  expect(usePlayerStore.getState().currentId).toBeNull();
  expect(useQueueStore.getState().queue).toEqual([]);
  expect(qc.getQueryData(["auth-status"])).toEqual(other);
});

it("keeps local logout immediate when backend cleanup reports an error", async () => {
  vi.mocked(logout).mockRejectedValue(new Error("cleanup failed"));
  await expect(logoutSession(qc)).rejects.toThrow("cleanup failed");
  expect(useAuthStore.getState()).toMatchObject({ loggedIn: false, phase: "idle" });
  expect(qc.getQueryData(["auth-status"])).toEqual(LOGGED_OUT);
});

it("does not let a library sync finishing after logout invalidate the new session", async () => {
  const sync = deferred<SyncResult>();
  vi.mocked(syncLibrary).mockReturnValue(sync.promise);
  vi.mocked(startLogin).mockResolvedValue(ACCOUNT);
  vi.mocked(logout).mockResolvedValue(undefined);
  await loginSession(qc);
  await logoutSession(qc);
  const invalidate = vi.spyOn(qc, "invalidateQueries");
  sync.resolve(SYNC);
  await sync.promise;
  await Promise.resolve();
  await Promise.resolve();
  expect(invalidate).not.toHaveBeenCalled();
});
