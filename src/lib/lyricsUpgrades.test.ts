import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import type { Lyrics } from "../api/lyrics";

const native = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: native.listen }));

type Handler = (event: { payload: Lyrics }) => void;
let subscribe: typeof import("./lyricsUpgrades").subscribeLyricsUpgrades;
let handlers: Handler[];
let registrations: Array<{ resolve: (release: () => void) => void; reject: (error: Error) => void }>;
const settled = async () => { await Promise.resolve(); await Promise.resolve(); };
const upgrade = (track_id: string): Lyrics => ({
  track_id, lines: [{ time_ms: 1000, text: "Upgraded lyric", words: [] }],
  plain: null, synced: true, word_level: true, instrumental: false, source: "amll",
  found: true, offset_ms: 0, alternates: [], upgrading: false, has_translation: false, has_roman: false,
});

beforeEach(async () => {
  vi.resetModules();
  ({ subscribeLyricsUpgrades: subscribe } = await import("./lyricsUpgrades"));
  handlers = [];
  registrations = [];
  native.listen.mockReset();
  native.listen.mockImplementation((_event: string, handler: Handler) => {
    handlers.push(handler);
    return new Promise<() => void>((resolve, reject) => registrations.push({ resolve, reject }));
  });
});

describe("shared lyric upgrades", () => {
  it("updates one query cache once and releases only after its last consumer leaves", async () => {
    const client = new QueryClient();
    const writes = vi.spyOn(client, "setQueryData");
    const first = subscribe(client, () => "a");
    const second = subscribe(client, () => "a");
    expect(native.listen).toHaveBeenCalledTimes(1);
    const release = vi.fn();
    registrations[0].resolve(release);
    await settled();
    const payload = upgrade("a");
    handlers[0]({ payload });
    expect(writes).toHaveBeenCalledTimes(1);
    expect(client.getQueryData(["lyrics", "a"])).toEqual(payload);
    first();
    expect(release).not.toHaveBeenCalled();
    handlers[0]({ payload: upgrade("a") });
    expect(writes).toHaveBeenCalledTimes(2);
    second();
    expect(release).toHaveBeenCalledTimes(1);
    handlers[0]({ payload });
    expect(writes).toHaveBeenCalledTimes(2);
  });

  it("keeps clients separate and ignores upgrades after the selected track changes", async () => {
    const firstClient = new QueryClient();
    const secondClient = new QueryClient();
    const otherClient = new QueryClient();
    let selected = "a";
    const cleanups = [
      subscribe(firstClient, () => selected), subscribe(secondClient, () => selected),
      subscribe(otherClient, () => "other"),
    ];
    const release = vi.fn();
    registrations[0].resolve(release);
    await settled();
    const payload = upgrade("a");
    handlers[0]({ payload });
    expect(firstClient.getQueryData(["lyrics", "a"])).toEqual(payload);
    expect(secondClient.getQueryData(["lyrics", "a"])).toEqual(payload);
    expect(otherClient.getQueryData(["lyrics", "a"])).toBeUndefined();
    selected = "b";
    handlers[0]({ payload: { ...payload, source: "late" } });
    expect(firstClient.getQueryData(["lyrics", "a"])).toEqual(payload);
    expect(secondClient.getQueryData(["lyrics", "a"])).toEqual(payload);
    cleanups.forEach((cleanup) => cleanup());
    expect(release).toHaveBeenCalledTimes(1);
  });

  it("releases a registration that resolves after every consumer unmounts", async () => {
    const client = new QueryClient();
    const cleanup = subscribe(client, () => "a");
    cleanup();
    const release = vi.fn();
    registrations[0].resolve(release);
    await settled();
    expect(release).toHaveBeenCalledTimes(1);
    handlers[0]({ payload: upgrade("a") });
    expect(client.getQueryData(["lyrics", "a"])).toBeUndefined();
  });

  it("reuses a pending registration across unmount and remount", async () => {
    const client = new QueryClient();
    subscribe(client, () => "a")();
    const cleanup = subscribe(client, () => "a");
    expect(native.listen).toHaveBeenCalledTimes(1);
    const release = vi.fn();
    registrations[0].resolve(release);
    await settled();
    expect(release).not.toHaveBeenCalled();
    handlers[0]({ payload: upgrade("a") });
    expect(client.getQueryData(["lyrics", "a"])).toEqual(upgrade("a"));
    cleanup();
    expect(release).toHaveBeenCalledTimes(1);
  });

  it("can register again after a failed native subscription", async () => {
    const client = new QueryClient();
    const first = subscribe(client, () => "a");
    registrations[0].reject(new Error("native registration failed"));
    await settled();
    const second = subscribe(client, () => "a");
    expect(native.listen).toHaveBeenCalledTimes(2);
    const release = vi.fn();
    registrations[1].resolve(release);
    await settled();
    const writes = vi.spyOn(client, "setQueryData");
    handlers[1]({ payload: upgrade("a") });
    expect(writes).toHaveBeenCalledTimes(1);
    first(); second();
    expect(release).toHaveBeenCalledTimes(1);
  });
});
