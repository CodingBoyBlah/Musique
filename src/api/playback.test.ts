import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { playTrack, resumeOrPlay } from "./playback";
import { transportPause, transportPlay } from "../hooks/usePlayerControls";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { toast } from "../store/toast.store";
import type { TrackItem } from "../types/spotify";

vi.hoisted(() => {
  const storage = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", { configurable: true, value: {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
    removeItem: (key: string) => storage.delete(key),
  } });
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../lib/jam", () => ({
  jamPlay: vi.fn(), jamSetPlaying: vi.fn(), jamNext: vi.fn(), jamPrev: vi.fn(), jamSeek: vi.fn(),
}));
vi.mock("../utils/radio", () => ({ replenishQueue: vi.fn() }));
vi.mock("../utils/listenTracker", () => ({ trackSkipped: vi.fn() }));

const TRACK: TrackItem = {
  id: "song-a", name: "Song A", duration_ms: 180_000, explicit: false, artists: [], album: null,
};

beforeEach(() => {
  vi.restoreAllMocks();
  vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
  usePlayerStore.getState().clear();
  usePlayerStore.setState({ currentTrack: TRACK, currentId: TRACK.id, positionMs: 30_000, sessionReady: true });
  useQueueStore.setState({ contextUri: null, contextTracks: [] });
});

it("dispatches pause while resume is still waiting for a download", async () => {
  let finish!: () => void;
  const pending = new Promise<void>((resolve) => { finish = resolve; });
  vi.mocked(invoke).mockImplementation((command) => command === "resume_or_play" ? pending : Promise.resolve(undefined));

  transportPlay();
  expect(invoke).toHaveBeenCalledWith("resume_or_play", expect.objectContaining({ positionMs: 30_000 }));
  transportPause();
  expect(invoke).toHaveBeenCalledWith("pause_playback");
  expect(usePlayerStore.getState().isPlaying).toBe(false);
  finish();
  await pending;
});

it("a superseded failed load cannot pause a newer song or show its error", async () => {
  let fail!: (error: Error) => void;
  vi.mocked(invoke).mockReturnValueOnce(new Promise((_, reject) => { fail = reject; }));
  const first = playTrack("song-a").catch((error) => error);
  await playTrack("song-b");
  const errorToast = vi.spyOn(toast, "error");
  fail(new Error("old download failed"));
  await first;
  expect(usePlayerStore.getState().isPlaying).toBe(true);
  expect(errorToast).not.toHaveBeenCalled();
});

it("a current playback failure still clears optimism and reports the error", async () => {
  vi.mocked(invoke).mockRejectedValueOnce(new Error("download failed"));
  const errorToast = vi.spyOn(toast, "error");
  await expect(playTrack("song-a")).rejects.toThrow("download failed");
  expect(usePlayerStore.getState().isPlaying).toBe(false);
  expect(errorToast).toHaveBeenCalled();
});

it("keeps an official Spotify device selected when choosing another song", async () => {
  usePlayerStore.setState({ isRemotePlayback: true, activeDevice: {
    id: "official-desktop", name: "Spotify desktop", type: "Computer", is_active: true,
    is_private_session: false, is_restricted: false, supports_volume: true, volume_percent: 50,
  } });
  await playTrack("song-b");
  expect(invoke).toHaveBeenCalledExactlyOnceWith("remote_play_track", {
    deviceId: "official-desktop", trackId: "song-b", positionMs: 0,
  });
  expect(usePlayerStore.getState().isRemotePlayback).toBe(true);

  await resumeOrPlay("spotify:episode:episode-a", 45_000);
  expect(invoke).toHaveBeenLastCalledWith("remote_play_track", {
    deviceId: "official-desktop", trackId: "spotify:episode:episode-a", positionMs: 45_000,
  });
});
