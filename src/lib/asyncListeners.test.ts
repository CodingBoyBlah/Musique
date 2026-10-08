import { expect, it, vi } from "vitest";
import { cleanupAsyncListeners } from "./asyncListeners";

it("releases successful registrations when a sibling registration fails", async () => {
  const first = vi.fn();
  const last = vi.fn();
  const cleanup = cleanupAsyncListeners([
    Promise.resolve(first), Promise.reject(new Error("registration failed")), Promise.resolve(last),
  ]);
  await Promise.resolve();
  cleanup();
  cleanup();
  expect(first).toHaveBeenCalledTimes(1);
  expect(last).toHaveBeenCalledTimes(1);
});

it("releases a listener that finishes registering after unmount", async () => {
  let resolve!: (unlisten: () => void) => void;
  const pending = new Promise<() => void>((done) => { resolve = done; });
  const unlisten = vi.fn();
  const cleanup = cleanupAsyncListeners([pending]);
  cleanup();
  resolve(unlisten);
  await pending;
  cleanup();
  expect(unlisten).toHaveBeenCalledTimes(1);
});
