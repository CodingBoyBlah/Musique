import { describe, it, expect } from "vitest";
import { toTurns, activeLine } from "./TranscriptView";
import type { TranscriptLine } from "../../types/podcast";

const l = (start_ms: number, text: string, speaker: string | null = null, heading = false): TranscriptLine => ({ start_ms, text, speaker, heading });

describe("transcript grouping", () => {
  it("starts a paragraph on each speaker change", () => {
    const turns = toTurns([
      l(0, "Intro", null, true),
      l(1000, "Hi.", "A"),
      l(2000, "Welcome."),
      l(3000, "Thanks.", "B"),
      l(4000, "Glad to be here."),
      l(5000, "So.", "A"),
    ]);
    expect(turns.map((t) => (t.heading ? "#" : t.speaker))).toEqual(["#", "A", "B", "A"]);
    expect(turns[1].lines.map((x) => x.line.text)).toEqual(["Hi.", "Welcome."]);
  });

  it("finds the sentence being spoken", () => {
    const t = { language: null, synced: true, lines: [l(0, "a"), l(1000, "b"), l(5000, "c")] };
    expect(activeLine(t, 4999)).toBe(1);
    expect(activeLine(t, 5000)).toBe(2);
  });
});
