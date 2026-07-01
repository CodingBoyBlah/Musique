import { useCallback, useEffect, useRef, useState } from "react";
import type { Lyrics, LyricLine } from "../api/lyrics";
import { usePlayerStore } from "../store/player.store";


export const CONCURRENT_TOL_MS = 60;

// row model

export type Row = { startMs: number; endMs: number; voices: LyricLine[] };

/* group near-simultaneous synced lines into rows of concurrent voices, or one
row per plain (untimed) line. same logic for every consumer. */
export function buildRows(data: Lyrics | undefined): Row[] {
  if (data?.lines.length) {
    const out: Row[] = [];
    for (const l of data.lines) {
      const last = out[out.length - 1];
      if (last && Math.abs(l.time_ms - last.startMs) <= CONCURRENT_TOL_MS) {
        last.voices.push(l);
      } else {
        out.push({ startMs: l.time_ms, endMs: 0, voices: [l] });
      }
    }
    for (let i = 0; i < out.length; i++) {
      out[i].endMs = i + 1 < out.length ? out[i + 1].startMs : out[i].startMs + 4000;
    }
    return out;
  }
  if (data?.plain) {
    return data.plain.split(/\r?\n/).map((t) => ({
      startMs: -1,
      endMs: -1,
      voices: [{ time_ms: -1, text: t, words: [] }],
    }));
  }
  return [];
}

// word model

export type RenderWord = { text: string; startMs: number; endMs: number };

/* map a line's real word timings to render words. no estimation - a line with
no word data just yields nothing word-by-word. */
export function mapWords(line: LyricLine): RenderWord[] {
  const ws = line.words;
  return ws.map((w, i) => ({
    text: w.text,
    startMs: w.time_ms,
    endMs: w.end_ms > w.time_ms ? w.end_ms : (ws[i + 1]?.time_ms ?? w.time_ms + 400),
  }));
}

// interpolated clock

/* a smooth monotonic clock the lyric offset rides on top of. the player only
reports whole-second positions, so we run a free interpolated clock and only
soft-correct it toward the reported position (hard resync on a real seek /
track jump). resync lets a consumer snap it after a click-to-seek. */
export function useLyricClock() {
  const positionMs = usePlayerStore((s) => s.positionMs);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const offset = usePlayerStore((s) => s.lyricsOffsetMs);

  const baseRef = useRef({ pos: positionMs, at: performance.now() });
  const playingRef = useRef(isPlaying);
  const offsetRef = useRef(offset);
  useEffect(() => {
    offsetRef.current = offset;
  }, [offset]);

  useEffect(() => {
    const now = performance.now();
    const predicted = playingRef.current
      ? baseRef.current.pos + (now - baseRef.current.at)
      : baseRef.current.pos;
    const drift = positionMs - predicted;
    if (Math.abs(drift) > 1200) {
      baseRef.current = { pos: positionMs, at: now }; // seek / track change
    } else {
      baseRef.current = { pos: predicted + drift * 0.2, at: now }; // gentle pull
    }
  }, [positionMs]);

  useEffect(() => {
    const now = performance.now();
    baseRef.current = {
      pos: playingRef.current
        ? baseRef.current.pos + (now - baseRef.current.at)
        : baseRef.current.pos,
      at: now,
    };
    playingRef.current = isPlaying;
  }, [isPlaying]);

  const getClock = useCallback(() => {
    const c = playingRef.current
      ? baseRef.current.pos + (performance.now() - baseRef.current.at)
      : baseRef.current.pos;
    return c + offsetRef.current;
  }, []);

  const resync = useCallback((ms: number) => {
    baseRef.current = { pos: ms, at: performance.now() };
  }, []);

  return { getClock, resync };
}

// track the active row index by polling the interpolated clock each frame
export function useActiveRow(rowStarts: number[], getClock: () => number, synced: boolean) {
  const [active, setActive] = useState(-1);
  useEffect(() => {
    if (!synced) {
      setActive(-1);
      return;
    }
    let raf = 0;
    const tick = () => {
      const t = getClock();
      let idx = -1;
      for (let i = 0; i < rowStarts.length; i++) {
        if (rowStarts[i] <= t) idx = i;
        else break;
      }
      setActive((prev) => (prev === idx ? prev : idx));
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [synced, rowStarts, getClock]);
  return active;
}

// word-by-word active line - apple-style brightness sweep


export function ActiveLine({
  words,
  getClock,
  size = 21,
  weight = 800,
  dim = 0.34,
  halo = 0.14,
}: {
  words: RenderWord[];
  getClock: () => number;
  size?: number;
  weight?: number;
  dim?: number;
  halo?: number;
}) {
  const spans = useRef<(HTMLSpanElement | null)[]>([]);

  useEffect(() => {
    let raf = 0;
    const tick = () => {
      const t = getClock();
      for (let i = 0; i < words.length; i++) {
        const el = spans.current[i];
        if (!el) continue;
        const w = words[i];
        const raw = (t - w.startMs) / Math.max(w.endMs - w.startMs, 1);
        const local = raw < 0 ? 0 : raw > 1 ? 1 : raw;
        const fill = local * local * (3 - 2 * local); // smoothstep
        const p = fill * 112 - 6;
        el.style.setProperty("--a", `${p.toFixed(2)}%`);
        el.style.setProperty("--b", `${(p + 13).toFixed(2)}%`);
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [words, getClock]);

  return (
    <p
      style={{
        margin: 0,
        fontSize: size,
        lineHeight: 1.3,
        letterSpacing: "-0.01em",
        fontWeight: weight,
        textShadow: `0 0 18px rgba(255,255,255,${halo})`,
      }}
    >
      {words.map((w, i) => (
        <span
          key={i}
          ref={(el) => {
            spans.current[i] = el;
          }}
          style={{
            display: "inline-block",
            whiteSpace: "pre",
            color: "transparent",
            backgroundImage: `linear-gradient(90deg, rgba(255,255,255,0.98) var(--a,-30%), rgba(255,255,255,${dim}) var(--b,-16%))`,
            WebkitBackgroundClip: "text",
            backgroundClip: "text",
          }}
        >
          {w.text}
        </span>
      ))}
    </p>
  );
}
