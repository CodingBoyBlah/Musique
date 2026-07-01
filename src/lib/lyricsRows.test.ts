import { describe, it, expect } from "vitest";
import { buildRows } from "./lyricsRows";
import type { LyricLine } from "../api/lyrics";

// line-level helper (no word timings, like LRCLIB)
const L = (time_ms: number, text: string): LyricLine => ({
  time_ms,
  text,
  words: [],
});

// word-level helper: one word spanning [time_ms, time_ms+dur]
const W = (time_ms: number, text: string, dur: number): LyricLine => ({
  time_ms,
  text,
  words: [{ time_ms, end_ms: time_ms + dur, text }],
});

describe("buildRows", () => {
  it("keeps normal sequential lines one-per-row (rhythm intact)", () => {
    const rows = buildRows([L(0, "a"), L(2000, "b"), L(4000, "c")]);
    expect(rows.map((r) => r.lead.text)).toEqual(["a", "b", "c"]);
    expect(rows.every((r) => r.bg === null)).toBe(true);
  });

  it("pairs same-timestamp lines as lead + background", () => {
    const rows = buildRows([L(1000, "Ok Ok Ok"), L(1050, "La La La"), L(3000, "next")]);
    expect(rows).toHaveLength(2);
    expect(rows[0].lead.text).toBe("Ok Ok Ok");
    expect(rows[0].bg?.text).toBe("La La La");
    expect(rows[1].lead.text).toBe("next");
    expect(rows[1].bg).toBeNull();
  });

  it("treats a time-overlapping word-level line as background", () => {
    // lead sung 1000..5000; backing starts at 2000 (overlaps by 3000ms)
    const rows = buildRows([W(1000, "lead", 4000), W(2000, "backing", 2000)]);
    expect(rows).toHaveLength(1);
    expect(rows[0].lead.text).toBe("lead");
    expect(rows[0].bg?.text).toBe("backing");
  });

  it("does NOT merge legato lines that only touch at the seam", () => {
    
    const rows = buildRows([W(1000, "lead", 1100), W(2000, "next", 1000)]);
    expect(rows).toHaveLength(2);
    expect(rows.every((r) => r.bg === null)).toBe(true);
  });

  it("trusts source-provided background (Apple Music TTML), one row per line", () => {
    const lead1: LyricLine = {
      time_ms: 1000,
      text: "Ok Ok Ok",
      words: [],
      bg: { time_ms: 1100, text: "La La La", words: [] },
    };
    const lead2 = L(3000, "plain next");
    const rows = buildRows([lead1, lead2]);
    expect(rows).toHaveLength(2);
    expect(rows[0].lead.text).toBe("Ok Ok Ok");
    expect(rows[0].bg?.text).toBe("La La La");
    expect(rows[1].lead.text).toBe("plain next");
    expect(rows[1].bg).toBeNull();
  });

  it("ignores unsynced placeholder lines (time_ms < 0)", () => {
    const rows = buildRows([L(-1, "x"), L(-1, "y")]);
    expect(rows).toHaveLength(2);
    expect(rows.every((r) => r.bg === null)).toBe(true);
  });
});
