import { describe, it, expect } from "vitest";
import { buildRows, voiceLayout, CONCURRENT_TOL_MS, type Row } from "./lyricsRows";
import type { LyricLine } from "../api/lyrics";

// line-level helper (no word timings, like LRCLIB)
const L = (time_ms: number, text: string, role?: LyricLine["role"]): LyricLine => ({
  time_ms,
  text,
  words: [],
  role,
});

// word-level helper: one word spanning [time_ms, time_ms+dur]
const W = (
  time_ms: number,
  text: string,
  dur: number,
  role?: LyricLine["role"],
): LyricLine => ({
  time_ms,
  text,
  words: [{ time_ms, end_ms: time_ms + dur, text }],
  role,
});

const lead = (r: Row) => r.voices[0];
const bg = (r: Row) => r.voices[1] ?? null;

describe("buildRows", () => {
  it("keeps normal sequential lines one-per-row (rhythm intact)", () => {
    const rows = buildRows([L(0, "a"), L(2000, "b"), L(4000, "c")]);
    expect(rows.map((r) => lead(r).text)).toEqual(["a", "b", "c"]);
    expect(rows.every((r) => r.voices.length === 1)).toBe(true);
    expect(rows[0].startMs).toBe(0);
    expect(rows[0].endMs).toBe(2000);
    expect(rows[1].startMs).toBe(2000);
    expect(rows[1].endMs).toBe(4000);
    expect(rows[2].startMs).toBe(4000);
    expect(rows[2].endMs).toBe(8000);
  });

  it("pairs same-timestamp lines within CONCURRENT_TOL_MS as concurrent voices in one row", () => {
    const rows = buildRows([
      L(1000, "Ok Ok Ok"),
      L(1050, "La La La"),
      L(3000, "next"),
    ]);
    expect(rows).toHaveLength(2);
    expect(lead(rows[0]).text).toBe("Ok Ok Ok");
    expect(bg(rows[0])?.text).toBe("La La La");
    expect(lead(rows[1]).text).toBe("next");
    expect(bg(rows[1])).toBeNull();
  });

  it("trusts source-provided nested background (Apple Music TTML)", () => {
    const lead1: LyricLine = {
      time_ms: 1000,
      text: "Ok Ok Ok",
      words: [],
      bg: { time_ms: 1100, text: "La La La", words: [] },
    };
    const lead2 = L(3000, "plain next");
    const rows = buildRows([lead1, lead2]);
    expect(rows).toHaveLength(2);
    expect(lead(rows[0]).text).toBe("Ok Ok Ok");
    expect(bg(rows[0])?.text).toBe("La La La");
    expect(lead(rows[1]).text).toBe("plain next");
    expect(bg(rows[1])).toBeNull();
  });

  it("folds an unnested line with role: 'bg' into the current row even outside CONCURRENT_TOL_MS", () => {
    const leadLine: LyricLine = {
      time_ms: 1000,
      text: "lead vocal",
      words: [{ time_ms: 1000, end_ms: 4000, text: "lead vocal" }],
      role: "main",
    };
    // Backing vocal starts 400ms later (well beyond CONCURRENT_TOL_MS = 60ms)
    const bgLine: LyricLine = {
      time_ms: 1400,
      text: "(backing vocal)",
      words: [{ time_ms: 1400, end_ms: 3500, text: "(backing vocal)" }],
      role: "bg",
    };
    const nextLine = L(6000, "after");

    const rows = buildRows([leadLine, bgLine, nextLine]);
    expect(rows).toHaveLength(2);
    expect(lead(rows[0]).text).toBe("lead vocal");
    expect(bg(rows[0])?.text).toBe("(backing vocal)");
    // folding affects ROW membership, while preserving the voice's own timing
    expect(rows[0].startMs).toBe(1000);
    expect(bg(rows[0])?.time_ms).toBe(1400);
    expect(lead(rows[1]).text).toBe("after");
  });

  it("derives row endMs from LEAD lines only and never from bg voice", () => {
    // Lead line is sung from 1000..3000ms.
    // Backing vocal starts at 1500ms and extends until 5000ms.
    const leadLine = W(1000, "lead line", 2000, "main"); // ends at 3000ms
    const bgLine = W(1500, "backing", 3500, "bg"); // ends at 5000ms
    const nextLead = W(8000, "next lead", 2000, "main");

    const rows = buildRows([leadLine, bgLine, nextLead]);
    expect(rows).toHaveLength(2);
    // Row 0 endMs MUST be 3000ms (from leadLine), NOT 5000ms from bgLine
    expect(rows[0].endMs).toBe(3000);
    expect(rows[0].voices).toHaveLength(2);
    expect(rows[1].startMs).toBe(8000);
    expect(rows[1].endMs).toBe(10000);
  });

  it("prefers lead line's real word-end over 'next row start' during instrumental gaps", () => {
    // Lead line sung 1000..4000ms. There is an instrumental gap until 20000ms.
    const leadLine = W(1000, "sing before solo", 3000); // ends at 4000ms
    const nextLine = W(20000, "sing after solo", 3000); // starts at 20000ms

    const rows = buildRows([leadLine, nextLine]);
    expect(rows).toHaveLength(2);
    // Real end (4000ms) preferred over next line start (20000ms) so highlight does not linger
    expect(rows[0].endMs).toBe(4000);
    expect(rows[1].endMs).toBe(23000);
  });

  it("falls back to next row's startMs for line-level sources without word timings", () => {
    const a = L(1000, "line level 1");
    const b = L(5000, "line level 2");

    const rows = buildRows([a, b]);
    expect(rows).toHaveLength(2);
    expect(rows[0].endMs).toBe(5000);
    expect(rows[1].endMs).toBe(9000); // fallback +4000
  });

  it("merges genuine same-instant duets within CONCURRENT_TOL_MS into one row", () => {
    const a: LyricLine = { time_ms: 1000, text: "singer 1", words: [], role: "main" };
    const b: LyricLine = { time_ms: 1040, text: "singer 2", words: [], role: "duet" };
    expect(Math.abs(b.time_ms - a.time_ms)).toBeLessThanOrEqual(CONCURRENT_TOL_MS);

    const rows = buildRows([a, b]);
    expect(rows).toHaveLength(1);
    expect(rows[0].voices).toHaveLength(2);
    expect(rows[0].voices[0].text).toBe("singer 1");
    expect(rows[0].voices[1].text).toBe("singer 2");
    expect(rows[0].voices[1].role).toBe("duet");
  });

  it("keeps duet lines at separate timestamps as their own rows", () => {
    const a: LyricLine = { time_ms: 1000, text: "her line", words: [], role: "main" };
    const b: LyricLine = { time_ms: 4000, text: "his line", words: [], role: "duet" };

    const rows = buildRows([a, b]);
    expect(rows).toHaveLength(2);
    expect(rows.map((r) => lead(r).text)).toEqual(["her line", "his line"]);
    expect(rows[0].voices).toHaveLength(1);
    expect(rows[1].voices).toHaveLength(1);
    expect(rows[1].voices[0].role).toBe("duet");
  });

  it("ignores unsynced placeholder lines (time_ms < 0)", () => {
    const rows = buildRows([L(-1, "x"), L(-1, "y")]);
    expect(rows).toHaveLength(2);
    expect(rows.every((r) => r.voices.length === 1)).toBe(true);
    expect(rows[0].startMs).toBe(-1);
    expect(rows[0].endMs).toBe(-1);
  });
});

describe("voiceLayout", () => {
  const row = (...voices: LyricLine[]): Row => ({ startMs: 0, endMs: 1000, voices });

  it("puts a lead line left and a duet line right", () => {
    expect(voiceLayout(row(L(0, "a")))).toEqual([{ side: "left", secondary: false }]);
    expect(voiceLayout(row(L(0, "a", "duet")))).toEqual([{ side: "right", secondary: false }]);
  });

  it("gives a backing vocal the side of the line it is sung under", () => {
    expect(voiceLayout(row(L(0, "a"), L(0, "(a)", "bg")))).toEqual([
      { side: "left", secondary: false },
      { side: "left", secondary: true },
    ]);
    expect(voiceLayout(row(L(0, "a", "duet"), L(0, "(a)", "bg")))).toEqual([
      { side: "right", secondary: false },
      { side: "right", secondary: true },
    ]);
  });

  it("follows the nearest owner when a lead and a duet share a row", () => {
    expect(
      voiceLayout(row(L(0, "a"), L(0, "(a)", "bg"), L(0, "b", "duet"), L(0, "(b)", "bg"))).map((v) => v.side),
    ).toEqual(["left", "left", "right", "right"]);
  });

  it("works from the row buildRows makes for a duet line with a nested backing vocal", () => {
    const duet: LyricLine = { ...L(0, "Could you be mine?", "duet"), bg: L(0, "(my love)", "bg") };
    const [r] = buildRows([duet]);
    expect(voiceLayout(r).map((v) => v.side)).toEqual(["right", "right"]);
  });
});
