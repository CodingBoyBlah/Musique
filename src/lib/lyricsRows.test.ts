import { describe, it, expect } from "vitest";
import { buildRows, voiceLayout, timedVoices, timedSecondaryVoices, overlappingVoices, CONCURRENT_TOL_MS, type Row } from "./lyricsRows";
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

  it("keeps a majority singer primary when a guest starts the shared row", () => {
    const guest = { ...L(0, "hey", "duet"), agent: "guest" };
    const main = { ...L(40, "most of the lyrics", "main"), agent: "lead", bg: L(50, "oh", "bg") };
    const [r] = buildRows([guest, main]);
    expect(voiceLayout(r)).toEqual([
      { side: "right", secondary: false },
      { side: "left", secondary: false },
      { side: "left", secondary: true },
    ]);
    // Old cached documents have roles but no retained agent IDs.
    expect(voiceLayout({ ...r, voices: r.voices.map(({ agent, ...v }) => v) })).toEqual([
      { side: "right", secondary: false },
      { side: "left", secondary: false },
      { side: "left", secondary: true },
    ]);
  });

  it("works from the row buildRows makes for a duet line with a nested backing vocal", () => {
    const duet: LyricLine = { ...L(0, "Could you be mine?", "duet"), bg: L(0, "(my love)", "bg") };
    const [r] = buildRows([duet]);
    expect(voiceLayout(r).map((v) => v.side)).toEqual(["right", "right"]);
  });
});

describe("overlapping sublyrics", () => {
  const rows = buildRows([
    { ...W(1000, "first lead", 2000), bg: W(2000, "held backing", 5000) },
    W(4000, "second lead", 1000),
    W(6000, "third lead", 2000),
  ]);
  const voices = timedSecondaryVoices(rows);

  it("keeps a nested sublyric singing across multiple main lines without extending the lead", () => {
    expect(rows[0].endMs).toBe(3000);
    expect(overlappingVoices(voices, 1, 4000)).toEqual([
      { row: 0, voice: 1, startMs: 2000, endMs: 7000 },
    ]);
    expect(overlappingVoices(voices, 2, 6500)).toEqual(voices);
  });

  it.each([
    [0, 2500, 0], // still in its own row
    [1, 4000, 1], // next main starts
    [2, 6999, 1],
    [2, 7000, 0], // sublyric ends, with no new main line
    [2, 9000, 0], // seek past the overlap
    [1, 4500, 1], // seek back into it
    [-1, 0, 0], // seek before the song
  ])("at main row %i and %ims, retains %i old sublyrics", (active, timeMs, count) => {
    expect(overlappingVoices(voices, active, timeMs)).toHaveLength(count);
  });

  it("waits for a delayed sublyric to begin even after its main row has passed", () => {
    const timed = timedSecondaryVoices(buildRows([
      { ...W(1000, "lead", 1500), bg: W(5000, "late backing", 2000, "bg") },
      W(3000, "next", 5000),
    ]));
    expect(overlappingVoices(timed, 1, 4999)).toEqual([]);
    expect(overlappingVoices(timed, 1, 5000)).toEqual(timed);
  });

  it("supports unnested backing vocals and simultaneous overlaps from older rows", () => {
    const timed = timedSecondaryVoices(buildRows([
      W(1000, "a", 1000), W(1500, "backing a", 7000, "bg"),
      { ...W(3000, "b", 1000), bg: W(3500, "backing b", 5000) },
      W(5000, "c", 1000),
    ]));
    expect(overlappingVoices(timed, 2, 6000).map((v) => v.row)).toEqual([0, 1]);
  });

  it("leaves untimed sublyrics, unsynced lyrics, and primary duets on normal row activity", () => {
    const untimed = { ...L(1000, "a"), bg: L(1500, "backing") };
    const unsynced = { ...L(-1, "plain"), bg: W(-1, "plain backing", 1000, "bg") };
    const duet = { ...W(1000, "duet", 6000, "duet"), agent: "guest" };
    expect(timedSecondaryVoices(buildRows([untimed, duet, unsynced]))).toEqual([]);
  });
});

describe("overlapping main singers", () => {
  it("keeps both main phrases active across staggered starts without making either a sublyric", () => {
    const rows = buildRows([
      { ...W(1000, "first singer", 5000, "main"), agent: "v1" },
      { ...W(2000, "second singer", 6000, "duet"), agent: "v2" },
      W(5000, "next phrase", 1000, "main"),
    ]);
    const voices = timedVoices(rows);
    expect(rows).toHaveLength(3);
    expect(rows.map((r) => voiceLayout(r)[0].secondary)).toEqual([false, false, false]);
    expect(overlappingVoices(voices, 1, 2500).map((v) => v.row)).toEqual([0]);
    expect(overlappingVoices(voices, 2, 5500).map((v) => v.row)).toEqual([0, 1]);
    expect(overlappingVoices(voices, 2, 6500).map((v) => v.row)).toEqual([1]);
    expect(overlappingVoices(voices, 2, 8000)).toEqual([]);
    expect(overlappingVoices(voices, 1, 2500).map((v) => v.row)).toEqual([0]);
  });

  it("uses explicit TTML ends for overlapping line-synced main vocals", () => {
    const rows = buildRows([
      { ...L(1000, "first", "main"), end_ms: 6000 },
      { ...L(2000, "second", "duet"), end_ms: 7000 },
    ]);
    expect(rows[0].endMs).toBe(6000);
    expect(overlappingVoices(timedVoices(rows), 1, 3000)).toEqual([
      { row: 0, voice: 0, startMs: 1000, endMs: 6000 },
    ]);
  });

  it("keeps simultaneous Apple main lines primary even without agent identifiers", () => {
    const [row] = buildRows([W(1000, "one", 1000, "main"), W(1040, "two", 3000, "main")]);
    expect(voiceLayout(row).every((v) => !v.secondary)).toBe(true);
    expect(timedVoices([row]).map((v) => v.endMs)).toEqual([2000, 4040]);
  });
});
