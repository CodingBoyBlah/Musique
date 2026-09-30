import { describe, it, expect } from "vitest";
import { listenTime, heatLevel, heatMax, peakSlot, hourLabel } from "./stats";

describe("stats formatting", () => {
  it("formats listening time", () => {
    expect(listenTime(0)).toBe("0 min");
    expect(listenTime(42 * 60000)).toBe("42 min");
    expect(listenTime(125 * 60000)).toBe("2 hr 5 min");
    expect(listenTime(120 * 60000)).toBe("2 hr");
  });

  it("scales heat with a visible floor", () => {
    expect(heatLevel(0, 10)).toBe(0);
    expect(heatLevel(1, 10)).toBeGreaterThan(0.18);
    expect(heatLevel(10, 10)).toBe(1);
  });

  it("finds the peak slot", () => {
    const grid = Array.from({ length: 7 }, () => Array(24).fill(0));
    expect(peakSlot(grid)).toBeNull();
    grid[2][21] = 5;
    grid[5][9] = 3;
    expect(heatMax(grid)).toBe(5);
    expect(peakSlot(grid)).toEqual({ day: 2, hour: 21, plays: 5 });
  });

  it("labels hours", () => {
    expect(hourLabel(0)).toBe("12am");
    expect(hourLabel(13)).toBe("1pm");
    expect(hourLabel(12)).toBe("12pm");
  });
});
