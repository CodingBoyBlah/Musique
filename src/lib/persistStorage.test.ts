import { describe, it, expect, beforeEach, beforeAll, afterEach, vi } from "vitest";
import { createStore } from "zustand/vanilla";
import { persist } from "zustand/middleware";
import { dedupedStorage } from "./persistStorage";

describe("dedupedStorage", () => {
  const memoryStore = new Map<string, string>();

  beforeAll(() => {
    const mockStorage = {
      getItem: (key: string) => memoryStore.get(key) ?? null,
      setItem: (key: string, value: string) => memoryStore.set(key, value),
      removeItem: (key: string) => memoryStore.delete(key),
      clear: () => memoryStore.clear(),
      length: 0,
      key: () => null,
    };
    Object.defineProperty(globalThis, "localStorage", {
      value: mockStorage,
      writable: true,
      configurable: true,
    });
  });

  beforeEach(() => {
    memoryStore.clear();
  });

  afterEach(() => vi.restoreAllMocks());

  it("stores and retrieves state", () => {
    const storage = dedupedStorage<{ count: number }>();
    storage.setItem("test-key", { state: { count: 1 }, version: 1 });

    const retrieved = storage.getItem("test-key");
    expect(retrieved).toEqual({ state: { count: 1 }, version: 1 });
  });

  it("skips serialization when partialized state is shallowly equal", () => {
    const storage = dedupedStorage<{ trackId: string; volume: number }>();
    const stringifySpy = vi.spyOn(JSON, "stringify");

    storage.setItem("player-key", { state: { trackId: "t1", volume: 80 }, version: 1 });
    const initialCallCount = stringifySpy.mock.calls.length;

    // Simulate 10 position ticks where partialized state fields stay shallow-identical
    for (let i = 0; i < 10; i++) {
      storage.setItem("player-key", { state: { trackId: "t1", volume: 80 }, version: 1 });
    }

    // Zero additional JSON.stringify calls should occur
    expect(stringifySpy.mock.calls.length).toBe(initialCallCount);

    stringifySpy.mockRestore();
  });

  it("serializes and persists when a field actually changes", () => {
    const storage = dedupedStorage<{ trackId: string; volume: number }>();
    const stringifySpy = vi.spyOn(JSON, "stringify");

    storage.setItem("player-key", { state: { trackId: "t1", volume: 80 }, version: 1 });
    const count1 = stringifySpy.mock.calls.length;

    // Volume changed
    storage.setItem("player-key", { state: { trackId: "t1", volume: 85 }, version: 1 });
    expect(stringifySpy.mock.calls.length).toBe(count1 + 1);

    expect(storage.getItem("player-key")).toEqual({
      state: { trackId: "t1", volume: 85 },
      version: 1,
    });

    stringifySpy.mockRestore();
  });

  it("removes items cleanly", () => {
    const storage = dedupedStorage<{ a: number }>();
    storage.setItem("rem-key", { state: { a: 1 }, version: 1 });
    expect(storage.getItem("rem-key")).not.toBeNull();

    storage.removeItem("rem-key");
    expect(storage.getItem("rem-key")).toBeNull();
    expect(localStorage.getItem("rem-key")).toBeNull();
  });

  it("retries a write after localStorage rejects it", () => {
    const storage = dedupedStorage<{ volume: number }>();
    const value = { state: { volume: 80 }, version: 1 };
    const write = vi.spyOn(localStorage, "setItem").mockImplementationOnce(() => {
      throw new Error("quota");
    });
    expect(() => storage.setItem("retry", value)).toThrow("quota");
    storage.setItem("retry", value);
    expect(write).toHaveBeenCalledTimes(2);
    expect(storage.getItem("retry")).toEqual(value);
  });

  it("persists version changes and nested immutable replacements", () => {
    const storage = dedupedStorage<{ track: { id: string }; volume: number }>();
    storage.setItem("versions", { state: { track: { id: "t1" }, volume: 80 }, version: 1 });
    storage.setItem("versions", { state: { track: { id: "t2" }, volume: 80 }, version: 2 });
    expect(storage.getItem("versions")).toEqual({ state: { track: { id: "t2" }, volume: 80 }, version: 2 });
  });

  it("allows the same state to be saved after an item is removed", () => {
    const storage = dedupedStorage<{ volume: number }>();
    const value = { state: { volume: 80 }, version: 1 };
    storage.setItem("removed", value);
    storage.removeItem("removed");
    storage.setItem("removed", value);
    expect(storage.getItem("removed")).toEqual(value);
  });

  it("keeps Zustand hydration and migrations while skipping position tick serialization", () => {
    localStorage.setItem("middleware", JSON.stringify({ state: { volume: 50 }, version: 0 }));
    const makeStore = () => createStore<{ volume: number; positionMs: number }>()(persist(
      () => ({ volume: 80, positionMs: 0 }),
      {
        name: "middleware", version: 1,
        storage: dedupedStorage<{ volume: number }>(),
        partialize: (state) => ({ volume: state.volume }),
        migrate: (state) => ({ volume: (state as { volume: number }).volume + 1 }),
      },
    ));
    const store = makeStore();
    expect(store.persist.hasHydrated()).toBe(true);
    expect(store.getState().volume).toBe(51);
    const stringify = vi.spyOn(JSON, "stringify");
    const write = vi.spyOn(localStorage, "setItem");
    for (let i = 1; i <= 60; i++) store.setState({ positionMs: i * 1000 });
    expect(stringify).not.toHaveBeenCalled();
    expect(write).not.toHaveBeenCalled();
    store.setState({ volume: 70 });
    expect(write).toHaveBeenCalledTimes(1);
    const restored = makeStore();
    expect(restored.getState()).toEqual({ volume: 70, positionMs: 0 });
  });
});
