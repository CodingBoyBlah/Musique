import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { usePlayerStore } from "../store/player.store";
import { anchorPlayhead, playheadMs } from "./playClock";

let t = 0;

beforeEach(() => {
  t = 1_000_000;
  vi.spyOn(performance, "now").mockImplementation(() => t);
  usePlayerStore.setState({ isPlaying: false, positionMs: 0, durationMs: 300_000, playbackRate: 1, currentId: "a" });
});

afterEach(() => vi.restoreAllMocks());

const report = (ms: number) =>
  usePlayerStore.getState().onEvent({ type: "position_changed", track_id: "a", position_ms: ms });

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

  it("takes a big disagreement at once, so a player that fell behind pulls lyrics back", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 0 });
    t += 10_000;
    // a rebuffer: the player is only at 9.6s
    report(9_600);
    expect(playheadMs()).toBe(9_600);
    t += 1_000;
    expect(playheadMs()).toBe(10_600);
  });

  it("closes a small disagreement over a second without ever going back", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 0 });
    t += 3_000;
    // the player is 40ms behind the clock
    report(2_960);
    // no jump back at the report...
    expect(playheadMs()).toBe(3_000);
    let last = playheadMs();
    for (let i = 0; i < 10; i++) {
      t += 100;
      expect(playheadMs()).toBeGreaterThan(last);
      last = playheadMs();
    }
    // ...and on the player's clock a second later
    expect(playheadMs()).toBeCloseTo(3_960, 6);
    t += 1_000;
    expect(playheadMs()).toBeCloseTo(4_960, 6);
  });

  it("absorbs a seek's start-up gap smoothly instead of jumping the lyrics back", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 10_000 });
    t += 1_000;
    // a lyric click: the clock jumps to the line at once
    usePlayerStore.getState().setPosition(50_000);
    anchorPlayhead(50_000, true);
    // the seek's audio took 60ms to start sounding; the next report says so
    t += 250;
    report(50_190);
    expect(playheadMs()).toBe(50_250);
    t += 1_000;
    expect(playheadMs()).toBeCloseTo(51_190, 6);
  });

  it("ignores a report the player sent before it acted on a seek", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 10_000 });
    t += 1_000;
    usePlayerStore.getState().setPosition(50_000);
    anchorPlayhead(50_000, true);
    t += 30;
    // a straggler from before the seek
    report(11_020);
    expect(playheadMs()).toBe(50_030);
    t += 200;
    report(50_200);
    expect(playheadMs()).toBe(50_230);
  });

  it("believes two far-off reports in a row", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 10_000 });
    usePlayerStore.getState().setPosition(50_000);
    anchorPlayhead(50_000, true);
    t += 100;
    report(20_100);
    t += 100;
    report(20_200);
    expect(playheadMs()).toBe(20_200);
  });

  it("freezes on pause where it was", () => {
    usePlayerStore.getState().onEvent({ type: "playing", track_id: "a", position_ms: 5_000 });
    t += 2_000;
    usePlayerStore.setState({ isPlaying: false });
    t += 10_000;
    expect(playheadMs()).toBe(7_000);
  });
});
