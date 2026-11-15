import type { LyricLine, Lyrics } from "../api/lyrics";

export const CONCURRENT_TOL_MS = 60;

/* Row model: a display row contains one primary lead voice plus any concurrent
voices (such as backing vocals or simultaneous duets). Timed backing vocals can
remain active after the lead advances to another row. */
export type Row = {
  startMs: number;
  endMs: number;
  voices: LyricLine[];
  /** an instrumental stretch, drawn as three breathing dots - see withInterludes */
  interlude?: boolean;
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
  if (lead.end_ms != null && lead.end_ms > lead.time_ms) return lead.end_ms;
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
      if (rawLine.bg) last.voices.push({ ...tidy(rawLine.bg), role: "bg" });
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
      if (rawLine.bg) last.voices.push({ ...tidy(rawLine.bg), role: "bg" });
      continue;
    }

    // Otherwise starts a new row
    const newRow: Row = {
      startMs: l.time_ms,
      endMs: 0,
      voices: [l],
    };
    if (rawLine.bg) {
      newRow.voices.push({ ...tidy(rawLine.bg), role: "bg" });
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
    const primary = i === 0 || v.role === "duet" || v.role === "main" || !!v.agent;
    const secondary = v.role === "bg" || !primary;
    if (secondary) return { side: owner, secondary };
    owner = v.role === "duet" ? "right" : "left";
    return { side: owner, secondary };
  });
}

export type OverlappingVoice = { row: number; voice: number; startMs: number; endMs: number };

/** Every measured vocal interval stays active across later rows, including leads. */
export function timedVoices(rows: Row[]): OverlappingVoice[] {
  return rows.flatMap((row, ri) => {
    return row.voices.flatMap((voice, vi) => {
      if (voice.time_ms < 0 || (!voice.words.length && voice.end_ms == null)) return [];
      const endMs = voice.words.length ? Math.max(...voice.words.map((w, wi) =>
        w.end_ms > w.time_ms ? w.end_ms : (voice.words[wi + 1]?.time_ms ?? w.time_ms + 400),
      )) : voice.end_ms!;
      return [{ row: ri, voice: vi, startMs: voice.time_ms, endMs }];
    });
  });
}

export function timedSecondaryVoices(rows: Row[]): OverlappingVoice[] {
  return timedVoices(rows).filter((v) => voiceLayout(rows[v.row])[v.voice].secondary);
}

export function overlappingVoices(voices: OverlappingVoice[], active: number, timeMs: number) {
  return voices.filter((v) => v.row < active && v.startMs <= timeMs && timeMs < v.endMs);
}

/* Instrumental stretches, the way Apple Music draws them.
 *
 * Sources mark a gap in the vocal three ways, and none of them is a line anyone
 * wants to read: Spotify sends a "♪" line, LRCLIB a blank one, and a word-timed
 * source just stops having words for a while. All three become one interlude
 * row, which the views draw as three dots that fill across the gap and fold
 * away when the voice comes back.
 *
 * - a run of marker/blank lines collapses to one interlude
 * - a long silence inside word-timed lyrics gets one, starting where the
 *   singing actually stopped rather than where the next line starts
 * - a long intro before the first line gets one at 0:00
 * - an interlude too short to read as a pause, or one with nothing after it
 *   (the outro), is dropped: dots racing to fill in a second look like a glitch
 *
 * Kept apart from buildRows so the grouping stays testable on its own. */
export const INTERLUDE_MIN_MS = 2500;
export const INTRO_MIN_MS = 3000;
export const SILENCE_MIN_MS = 7000;

const MARKER_ONLY = /^[\s♪♫♬\u{1F3B5}\u{1F3B6}]*$/u; // blank, ♪ ♫ ♬ 🎵 🎶

function isMarkerRow(row: Row): boolean {
  return row.voices.every(
    (v) => MARKER_ONLY.test(v.text) && v.words.every((w) => MARKER_ONLY.test(w.text)),
  );
}

/** when a row's singing really stops, or null if it has no word timings to say */
function sungEnd(row: Row): number | null {
  let end: number | null = null;
  for (const v of row.voices) {
    for (const w of v.words) end = Math.max(end ?? w.time_ms, w.time_ms, w.end_ms);
  }
  return end;
}

const gapRow = (startMs: number): Row => ({
  startMs,
  endMs: startMs,
  voices: [{ time_ms: startMs, text: "", words: [] }],
  interlude: true,
});

export function withInterludes(rows: Row[]): Row[] {
  if (!rows.length || rows[0].startMs < 0) return rows; // unsynced: nothing to time

  const out: Row[] = [];
  for (const row of rows) {
    const last = out[out.length - 1];
    if (isMarkerRow(row)) {
      if (!last?.interlude) out.push(gapRow(row.startMs));
      continue;
    }
    if (last && !last.interlude) {
      const end = sungEnd(last);
      if (end != null && row.startMs - end >= SILENCE_MIN_MS) out.push(gapRow(end));
    }
    out.push(row);
  }

  if (out.length) {
    if (out[0].interlude) out[0] = gapRow(0);
    else if (out[0].startMs >= INTRO_MIN_MS) out.unshift(gapRow(0));
  }

  return out.filter((row, i) => {
    if (!row.interlude) return true;
    const next = out[i + 1];
    if (!next) return false;
    row.endMs = next.startMs;
    return row.endMs - row.startMs >= INTERLUDE_MIN_MS;
  });
}

/* How far each row sits from the active one, counting only rows on screen.
   A folded interlude takes no space, so it must not push the line after it a
   step further out of focus than it looks. */
export function rowDistances(rows: Row[], active: number): number[] {
  const pos: number[] = [];
  let p = 0;
  for (let i = 0; i < rows.length; i++) {
    pos.push(p);
    if (!rows[i].interlude || i === active) p++;
  }
  const at = active < 0 ? -1 : pos[active];
  return pos.map((x) => Math.abs(x - at));
}
