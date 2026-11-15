import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { usePlayerStore } from "../store/player.store";
import { playheadMs } from "./playClock";

let t = 0;

beforeEach(() => {
  t = 1_000_000;
  vi.spyOn(performance, "now").mockImplementation(() => t);
  usePlayerStore.setState({ isPlaying: false, positionMs: 0, durationMs: 300_000, playbackRate: 1, currentId: "a" });
});

afterEach(() => vi.restoreAllMocks());

describe("playhead clock", () => {
  it("runs on real time between reports, however late the ticker fires", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 10_000 });
    // the window was in the background: the 1s interval fired once in 7s
    t += 7_000;
    usePlayerStore.getState().incrementPos();
    expect(usePlayerStore.getState().positionMs).toBe(17_000);
    expect(playheadMs()).toBe(17_000);
    // and the ticker's own write didn't re-anchor anything
    t += 500;
    expect(playheadMs()).toBe(17_500);
  });

  it("re-anchors on every report, so a player that fell behind pulls lyrics back", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 0 });
    t += 10_000;
    // a rebuffer: the player is only at 9.7s
    usePlayerStore.getState().onEvent({ type: "position_changed", track_id: "a", position_ms: 9_700 });
    expect(playheadMs()).toBe(9_700);
    t += 1_000;
    expect(playheadMs()).toBe(10_700);
  });

  it("smooths report jitter instead of twitching the sweep", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 0 });
    t += 1_000;
    usePlayerStore.getState().onEvent({ type: "position_changed", track_id: "a", position_ms: 1_040 });
    expect(playheadMs()).toBe(1_020);
  });

  it("freezes on pause where it was", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 5_000 });
    t += 2_000;
    usePlayerStore.setState({ isPlaying: false });
    t += 10_000;
    expect(playheadMs()).toBe(7_000);
  });
});
