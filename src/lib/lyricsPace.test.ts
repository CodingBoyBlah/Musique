import { describe, it, expect } from "vitest";
import { paceWords } from "./lyrics";

/* paceWords gives a line that only has a start and an end a sweep across its
   own words. Most tracks resolve to LRCLIB, which is line-level only, so
   without this the active lyric just switches on and sits there. The estimate
   will not match the vocal the way real word data does - what these pin down is
   that it stays inside the line's own bounds and never runs backwards. */

const text = (ws: ReturnType<typeof paceWords>) => ws.map((w) => w.text).join("");

describe("paceWords", () => {
  it("reconstructs the line exactly, spacing included", () => {
    const src = "Are we still friends?";
    expect(text(paceWords(src, 0, 4000))).toBe(src);
  });

  it("starts on the line and finishes exactly on its end", () => {
    const ws = paceWords("I heard you say that you need someone", 12_000, 16_400);
    expect(ws[0].startMs).toBe(12_000);
    expect(ws[ws.length - 1].endMs).toBe(16_400);
  });

  it("never runs backwards and leaves no gaps", () => {
    const ws = paceWords("Sleeping with the moon and the stars", 500, 5_000);
    for (let i = 0; i < ws.length; i++) {
      expect(ws[i].endMs).toBeGreaterThan(ws[i].startMs);
      if (i) expect(ws[i].startMs).toBeCloseTo(ws[i - 1].endMs, 5);
    }
  });

  it("gives a long word more of the line than a short one", () => {
    const [i, extraordinary] = paceWords("I extraordinary", 0, 4000);
    expect(extraordinary.endMs - extraordinary.startMs).toBeGreaterThan(i.endMs - i.startMs);
  });

  it("still paces a one-word line", () => {
    const ws = paceWords("(Yeah)", 1_000, 2_000);
    expect(ws).toHaveLength(1);
    expect(ws[0]).toMatchObject({ startMs: 1_000, endMs: 2_000 });
  });

  it("survives a line that is empty or only whitespace", () => {
    expect(paceWords("", 0, 1000)).toEqual([]);
    expect(paceWords("   ", 0, 1000)).toEqual([]);
  });

  it("does not divide by zero when a line has no duration", () => {
    const ws = paceWords("hold on", 8_000, 8_000);
    expect(ws.every((w) => Number.isFinite(w.startMs) && Number.isFinite(w.endMs))).toBe(true);
    expect(ws[ws.length - 1].endMs).toBe(8_000);
  });

  it("keeps short words from flashing past on a long line", () => {
    // a fixed cost per word means "a" still gets a readable slice
    const ws = paceWords("a b c d e f g h", 0, 8_000);
    expect(Math.min(...ws.map((w) => w.endMs - w.startMs))).toBeGreaterThan(100);
  });
});
