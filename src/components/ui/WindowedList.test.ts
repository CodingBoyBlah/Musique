import { describe, expect, it } from "vitest";
import { computeWindowRange } from "./WindowedList";

describe("computeWindowRange", () => {
  it("renders all rows without spacers at or below threshold", () => {
    expect(
      computeWindowRange({
        count: 60,
        threshold: 60,
        pitch: 59,
        scrollTop: 500,
        offsetTop: 300,
        viewportHeight: 800,
        overscan: 16,
      }),
    ).toEqual({ start: 0, end: 60, topSpacerPx: 0, bottomSpacerPx: 0 });
  });

  it("windows large lists around the scroll viewport with overscan", () => {
    const r = computeWindowRange({
      count: 1500,
      threshold: 60,
      pitch: 60,
      scrollTop: 3300,
      offsetTop: 300,
      viewportHeight: 600,
      overscan: 10,
    });
    // relTop = 3000 => firstVisible = 50, visibleCount = 11
    expect(r.start).toBe(40);
    expect(r.end).toBe(71);
    expect(r.topSpacerPx).toBe(40 * 60);
    expect(r.bottomSpacerPx).toBe((1500 - 71) * 60);
  });

  it("clamps cleanly at start and end boundaries", () => {
    const start = computeWindowRange({
      count: 1500,
      threshold: 60,
      pitch: 60,
      scrollTop: 0,
      offsetTop: 300,
      viewportHeight: 600,
      overscan: 10,
    });
    expect(start.start).toBe(0);
    expect(start.end).toBe(21);
    expect(start.topSpacerPx).toBe(0);

    const end = computeWindowRange({
      count: 100,
      threshold: 60,
      pitch: 60,
      scrollTop: 6000,
      offsetTop: 0,
      viewportHeight: 600,
      overscan: 10,
    });
    expect(end.end).toBe(100);
    expect(end.bottomSpacerPx).toBe(0);
  });
});
