import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { getLyrics, type Lyrics } from "./lyrics";
import type { TrackItem } from "../types/spotify";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const track: TrackItem = {
  id: "track", name: "Song", duration_ms: 180000, explicit: false,
  artists: [{ id: "artist", name: "Artist", image_url: null }], album: null,
};
const result = { track_id: track.id } as Lyrics;
const backend = vi.mocked(invoke);
beforeEach(() => backend.mockReset());

it("coalesces identical pending lyric requests and releases settled results", async () => {
  let resolve!: (value: Lyrics) => void;
  backend.mockImplementationOnce(() => new Promise< Lyrics >((done) => { resolve = done; }));
  const first = getLyrics(track);
  const second = getLyrics({ ...track });
  expect(backend).toHaveBeenCalledTimes(1);
  resolve(result);
  expect(await first).toBe(result);
  expect(await second).toBe(result);
  backend.mockResolvedValueOnce(result);
  await getLyrics(track);
  expect(backend).toHaveBeenCalledTimes(2);
});

it("keeps force refresh and richer matching metadata independent", async () => {
  backend.mockResolvedValue(result);
  await Promise.all([
    getLyrics(track), getLyrics(track, true),
    getLyrics({ ...track, external_ids: { isrc: "NEW-ISRC" } }),
  ]);
  expect(backend).toHaveBeenCalledTimes(3);
  expect(backend).toHaveBeenCalledWith("get_lyrics", expect.objectContaining({ force: true }));
  expect(backend).toHaveBeenCalledWith("get_lyrics", expect.objectContaining({ isrc: "NEW-ISRC" }));
});

it("allows a failed request to be retried", async () => {
  backend.mockRejectedValueOnce(new Error("offline"));
  await expect(getLyrics(track)).rejects.toThrow("offline");
  backend.mockResolvedValueOnce(result);
  await expect(getLyrics(track)).resolves.toBe(result);
  expect(backend).toHaveBeenCalledTimes(2);
});
