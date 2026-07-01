import type { LyricLine } from "../api/lyrics";

/* a display row = one lead line plus an optional background (backing vocal)
line sung simultaneously with it. each keeps its own timing/words so the
renderer can animate them independently at the same tim */
export type LyricRow = {
  startMs: number;
  lead: LyricLine;
  bg: LyricLine | null;
};


const SAME_START_TOL = 200;
/* for word-level sources: the next line has to overlap the leads sung window
by at least this much to count as a concurrent backing vocal rather than just
the next lead line following on. legato lines overlap 100-300ms - those must
NOT merge or the sequential rhythm falls apart */
const MIN_OVERLAP = 400;

// absolute ms the line stops being sung. word-level -> last word end.
function lineEnd(line: LyricLine, fallbackNextStart: number): number {
  if (line.words.length) {
    let max = line.words[0].end_ms;
    for (const w of line.words) if (w.end_ms > max) max = w.end_ms;
    return max;
  }
  return fallbackNextStart; // line-level (LRCLIB): no word ends -> next start
}


export function buildRows(lines: LyricLine[]): LyricRow[] {

  if (lines.some((l) => l.bg)) {
    return lines.map((l) => ({ startMs: l.time_ms, lead: l, bg: l.bg ?? null }));
  }

  // otherwise infer concurrency from timing (same-ts duets / overlapping lines)
  const rows: LyricRow[] = [];
  let i = 0;
  while (i < lines.length) {
    const lead = lines[i];
    const next = lines[i + 1];

    if (
      next &&
      lead.time_ms >= 0 &&
      next.time_ms >= 0 &&
      isConcurrent(lead, next)
    ) {
      rows.push({ startMs: lead.time_ms, lead, bg: next });
      i += 2; // eat the backing line so it isn't also rendered as a lead row
      continue;
    }

    rows.push({ startMs: lead.time_ms, lead, bg: null });
    i += 1;
  }
  return rows;
}

function isConcurrent(lead: LyricLine, next: LyricLine): boolean {
  const sameStart = Math.abs(next.time_ms - lead.time_ms) <= SAME_START_TOL;
  const end = lineEnd(lead, next.time_ms);
  const overlaps = next.time_ms < end - MIN_OVERLAP;
  return sameStart || overlaps;
}
