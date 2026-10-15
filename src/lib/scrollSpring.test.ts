import { afterEach, expect, it, vi } from "vitest";
import { createScrollSpring } from "./scrollSpring";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

it("glides through changing layout targets, settles, and yields immediately to the reader", () => {
  let now = 0;
  let id = 0;
  const frames = new Map<number, FrameRequestCallback>();
  vi.spyOn(performance, "now").mockImplementation(() => now);
  vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
    frames.set(++id, cb);
    return id;
  });
  vi.stubGlobal("cancelAnimationFrame", (key: number) => frames.delete(key));
  const el = { scrollTop: 0, scrollHeight: 2000, clientHeight: 400 } as HTMLElement;
  const spring = createScrollSpring(el);
  const frame = () => {
    now += 1000 / 60;
    const pending = [...frames.values()];
    frames.clear();
    pending.forEach((cb) => cb(now));
  };

  spring.to(160);
  let previous = el.scrollTop;
  for (let i = 0; i < 60; i++) {
    // A revealing sublyric moves the resting position while scrolling.
    const target = 160 + Math.min(i, 24) * 2;
    spring.to(target);
    expect(el.scrollTop).toBe(previous);
    frame();
    expect(el.scrollTop).toBeGreaterThanOrEqual(previous);
    expect(el.scrollTop).toBeLessThanOrEqual(target);
    expect(el.scrollTop - previous).toBeLessThan(16);
    previous = el.scrollTop;
  }
  for (let i = 0; i < 60; i++) frame();
  expect(el.scrollTop).toBe(208);
  expect(spring.running).toBe(false);

  spring.to(600);
  frame();
  spring.stop();
  const stopped = el.scrollTop;
  frame();
  expect(el.scrollTop).toBe(stopped);
  expect(frames.size).toBe(0);

  // Reduced-motion placement and initial opening never animate.
  spring.jump(5000);
  expect(el.scrollTop).toBe(1600);
  expect(spring.running).toBe(false);
});
