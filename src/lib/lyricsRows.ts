import type { LyricLine, Lyrics } from "../api/lyrics";

export const CONCURRENT_TOL_MS = 60;

/* Row model: a display row contains one primary lead voice plus any concurrent
voices (such as backing vocals or simultaneous duets). All voices in a row share
the row's active/inactive state while animating on their own timings. */
export type Row = {
  startMs: number;
  endMs: number;
  voices: LyricLine[];
};

const cleanSpaces = (s: string) => (s || "").replace(/[\u00A0\u200B\u202F\uFEFF]/g, " ");

const tidy = (line: LyricLine): LyricLine => ({
  ...line,
  text: cleanSpaces(line.text),
  words: (line.words || []).map((w) => ({ ...w, text: cleanSpaces(w.text) })),
  bg: line.bg ? tidy(line.bg) : line.bg,
});

/* absolute ms the lead line stops being sung. word-level -> last word end.
   line-level -> fallback to next row's start or startMs + 4000 */
function leadLineEnd(lead: LyricLine, fallbackNextStart: number): number {
  if (lead.words && lead.words.length > 0) {
    let max = lead.words[0].end_ms;
    for (const w of lead.words) {
      if (w.end_ms > max) max = w.end_ms;
    }
    if (max > lead.time_ms) return max;
  }
  return fallbackNextStart;
}

export function buildRows(data: Lyrics | LyricLine[] | undefined): Row[] {
  if (!data) return [];

  let lines: LyricLine[] = [];
  let plain: string | null = null;

  if (Array.isArray(data)) {
    lines = data;
  } else {
    if (data.lines && data.lines.length > 0) {
      lines = data.lines;
    } else if (data.plain) {
      plain = data.plain;
    }
  }

  if (plain) {
    return plain.split(/\r?\n/).map((t) => ({
      startMs: -1,
      endMs: -1,
      voices: [{ time_ms: -1, text: cleanSpaces(t), words: [] }],
    }));
  }

  if (!lines.length) return [];

  const out: Row[] = [];

  for (const rawLine of lines) {
    const l = tidy(rawLine);
    const last = out[out.length - 1];

    // A line arriving with role: "bg" that is NOT nested must be folded into the
    // current row as a concurrent voice rather than starting a new row, even if
    // it falls outside CONCURRENT_TOL_MS.
    if (l.role === "bg" && last) {
      last.voices.push(l);
      if (rawLine.bg) last.voices.push(tidy(rawLine.bg));
      continue;
    }

    // Keep CONCURRENT_TOL_MS for genuine same-instant duets / concurrent lines
    if (
      last &&
      last.startMs >= 0 &&
      l.time_ms >= 0 &&
      Math.abs(l.time_ms - last.startMs) <= CONCURRENT_TOL_MS
    ) {
      last.voices.push(l);
      if (rawLine.bg) last.voices.push(tidy(rawLine.bg));
      continue;
    }

    // Otherwise starts a new row
    const newRow: Row = {
      startMs: l.time_ms,
      endMs: 0,
      voices: [l],
    };
    if (rawLine.bg) {
      newRow.voices.push(tidy(rawLine.bg));
    }
    out.push(newRow);
  }

  // Derive endMs from LEAD lines only. A bg/duet voice inside a row must never
  // define the end of the previous row. Prefer a lead line's real end (last word's
  // end_ms) over "next row's start" when the source carries word timings.
  for (let i = 0; i < out.length; i++) {
    const lead = out[i].voices[0];
    const fallbackNext = i + 1 < out.length ? out[i + 1].startMs : out[i].startMs + 4000;
    out[i].endMs = out[i].startMs >= 0 ? leadLineEnd(lead, fallbackNext) : -1;
  }

  return out;
}

export type VoiceSide = "left" | "right";

/* Where each voice in a row sits, Apple Music style: the lead singer on the
left, the duet singer on the right. A backing vocal (any non-duet voice after
the first) is sung UNDER someone, so it takes that singer's side - the voice
before it that isn't itself a backing vocal. It used to be pinned left
regardless, so a duet line's backing vocal sat on the wrong side of the
screen from the line it belongs to. */
export function voiceLayout(row: Row): { side: VoiceSide; secondary: boolean }[] {
  let owner: VoiceSide = "left";
  return row.voices.map((v, i) => {
    const secondary = i > 0 && v.role !== "duet";
    if (secondary) return { side: owner, secondary };
    owner = v.role === "duet" ? "right" : "left";
    return { side: owner, secondary };
  });
}
