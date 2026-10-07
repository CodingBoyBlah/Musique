import { memo, useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import type { CSSProperties } from "react";
import type { LyricLine } from "../api/lyrics";
import { usePlayerStore } from "../store/player.store";
import { getOutputLatencyMs, startLatencyPolling } from "./outputLatency";
import { useWindowActive } from "../hooks/useWindowActive";


export { CONCURRENT_TOL_MS, buildRows, voiceLayout, type Row, type VoiceSide } from "./lyricsRows";

const cleanSpaces = (s: string) => (s || "").replace(/[\u00A0\u200B\u202F\uFEFF]/g, " ");

// word model

export type RenderWord = { text: string; startMs: number; endMs: number };

/* map a line's real word timings to render words */
export function mapWords(line: LyricLine): RenderWord[] {
  const ws = line.words;
  return ws.map((w, i) => ({
    text: cleanSpaces(w.text),
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
  const initial = usePlayerStore.getState();
  const baseRef = useRef({ pos: initial.positionMs, at: performance.now() });
  const playingRef = useRef(initial.isPlaying);
  const offsetRef = useRef(initial.lyricsOffsetMs);

  // ref-counted, shared with any other lyric surface; polls slowly in the
  // background and never re-renders us
  useEffect(() => startLatencyPolling(), []);

  useEffect(() => {
    const init = usePlayerStore.getState();
    baseRef.current = { pos: init.positionMs, at: performance.now() };
    playingRef.current = init.isPlaying;
    offsetRef.current = init.lyricsOffsetMs;

    return usePlayerStore.subscribe((state, prev) => {
      offsetRef.current = state.lyricsOffsetMs;

      if (state.positionMs !== prev.positionMs) {
        const now = performance.now();
        const predicted = playingRef.current
          ? baseRef.current.pos + (now - baseRef.current.at)
          : baseRef.current.pos;
        const drift = state.positionMs - predicted;
        if (Math.abs(drift) > 1200) {
          baseRef.current = { pos: state.positionMs, at: now }; // seek / track change
        } else {
          baseRef.current = { pos: predicted + drift * 0.2, at: now }; // gentle pull
        }
      }

      // Match the original effects: correct position using the previous play
      // state before rebasing a simultaneous pause/resume update.
      if (state.isPlaying !== prev.isPlaying) {
        const now = performance.now();
        baseRef.current = {
          pos: playingRef.current
            ? baseRef.current.pos + (now - baseRef.current.at)
            : baseRef.current.pos,
          at: now,
        };
        playingRef.current = state.isPlaying;
      }
    });
  }, []);

  const getClock = useCallback(() => {
    const c = playingRef.current
      ? baseRef.current.pos + (performance.now() - baseRef.current.at)
      : baseRef.current.pos;
    /* the reported position is where the DECODER is; the listener is hearing
       whatever left it a buffer ago. subtracting the measured output latency is
       what actually puts the highlight on the beat - the manual offset on top
       is now purely taste, not compensation. */
    return c - getOutputLatencyMs() + offsetRef.current;
  }, []);

  const resync = useCallback((ms: number) => {
    baseRef.current = { pos: ms, at: performance.now() };
  }, []);

  return { getClock, resync };
}

// active row

/* Wake exactly when the next line starts, not sixty times a second.
 *
 * This used to run a requestAnimationFrame loop that did a binary search every
 * 35ms to answer a question whose answer only changes a handful of times a
 * minute. We already know every row boundary and we already have an
 * interpolated clock, so the next boundary is just arithmetic: sleep until it,
 * re-check, sleep again. The 1s ceiling keeps the timer honest across the
 * clock's soft drift correction; the 16ms floor stops it spinning if a boundary
 * lands in the past. Playback pause and seek both re-arm it through the deps. */
export function useActiveRow(rowStarts: number[], getClock: () => number, synced: boolean) {
  const [active, setActive] = useState(-1);
  const activeRef = useRef(-1);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  useEffect(() => {
    if (!synced || !rowStarts.length) {
      activeRef.current = -1;
      setActive(-1);
      return;
    }

    let timer = 0;
    const run = () => {
      const t = getClock();
      let low = 0;
      let high = rowStarts.length - 1;
      let idx = -1;
      while (low <= high) {
        const mid = (low + high) >> 1;
        if (rowStarts[mid] <= t) {
          idx = mid;
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      if (idx !== activeRef.current) {
        activeRef.current = idx;
        setActive(idx);
      }
      if (!usePlayerStore.getState().isPlaying) return;
      const next = rowStarts[idx + 1];
      timer = window.setTimeout(run, next === undefined ? 1000 : Math.min(1000, Math.max(16, next - t)));
    };

    run();

    // Re-arm on every reported position, including small seeks while paused,
    // without making unchanged active rows render on each tick.
    const unsub = usePlayerStore.subscribe((state, prev) => {
      if (state.positionMs !== prev.positionMs || state.currentId !== prev.currentId) {
        window.clearTimeout(timer);
        run();
      }
    });

    return () => {
      window.clearTimeout(timer);
      unsub();
    };
  }, [synced, rowStarts, getClock, isPlaying]);

  return active;
}

// distance falloff

export interface Tone {
  /** CSS shadow blur radius. 0 means render the glyphs solid. */
  blur: number;
  alpha: number;
}

/* How far a line has fallen out of the present, as light rather than as size.
 *
 * Cider never scales a lyric - every line is set at the same size, and the only
 * thing that moves the eye is how much light each one carries. Scaling the
 * active row was also what pushed text past the right edge of the panel: a
 * transform scaled from `left center` grows the box to the right without the
 * layout knowing, so a row that fit at 1.0 clipped at 1.12. */
export function lyricTone(distance: number, past: boolean, solid = false): Tone {
  if (distance === 0) return { blur: 0, alpha: 1 };
  /* Increased contrast: no line dissolves into light. Distance is carried by
     opacity alone, and never so low a reader who asked for contrast loses it. */
  if (solid) {
    const d = distance - 1;
    return { blur: 0, alpha: Math.max(0.5, (past ? 0.62 : 0.72) - d * 0.06) };
  }
  /* The line either side of the active one stays sharp and plainly readable -
     in the reference it is the second line out that starts to soften. Falling
     off faster than this is what turns the panel into mush: the eye has nothing
     to read ahead to, so the whole column reads as one grey smear. */
  const d = distance - 1;
  return {
    blur: Math.min(6.5, d * 2.2),
    alpha: Math.max(0.1, (past ? 0.42 : 0.54) - d * 0.12),
  };
}

/* The OS "increase contrast" setting, live. Blurred distant lines are the one
   thing CSS alone cannot undo, since the blur is baked into a text-shadow. */
const CONTRAST_Q = "(prefers-contrast: more)";
function subscribeContrast(cb: () => void) {
  const mq = window.matchMedia?.(CONTRAST_Q);
  mq?.addEventListener?.("change", cb);
  return () => mq?.removeEventListener?.("change", cb);
}
export function useMoreContrast(): boolean {
  return useSyncExternalStore(
    subscribeContrast,
    () => !!window.matchMedia?.(CONTRAST_Q).matches,
    () => false,
  );
}

/* A blurred line, drawn as light with no body.
 *
 * `text-shadow`'s blur is a gaussian over the glyph coverage mask - the same
 * convolution `filter: blur()` performs, at twice the radius - so a transparent
 * fill plus a shadow is a real blur of the text. The difference is what it
 * costs: `filter` promotes the row to its own render surface the compositor has
 * to hold and re-blend, and the old view put one on every row in the list with
 * `will-change: transform` on top of it. A shadow is an ordinary paint op
 * inside the row's existing layer. Distant lines dissolving into pure light is
 * also, on its own terms, the look we want.
 *
 * The alpha stays at full here and the row carries the dimming instead. That is
 * what makes the handoff invisible: when a line stops being active its DOM
 * swaps from the sweep's gradient spans to a plain paragraph, and both are
 * white at that moment, so all the eye sees is the row's opacity easing down. */
export function toneText(tone: Tone, ink = "255, 255, 255"): CSSProperties {
  if (tone.blur <= 0) return { color: `rgba(${ink}, 0.97)`, textShadow: "none" };
  // a second, tighter pass fills the glyph interiors the wide one hollows out
  return {
    color: "transparent",
    textShadow:
      `0 0 ${tone.blur.toFixed(1)}px rgba(${ink}, 0.95), ` +
      `0 0 ${(tone.blur * 0.42).toFixed(1)}px rgba(${ink}, 0.82)`,
  };
}

/* Punctuation is not a word, and must not be allowed to become a line.
 *
 * Every token is rendered as an inline-block so the sweep gradient can clip to
 * it, and a break opportunity exists between any two inline-blocks. Word-level
 * sources tokenise however they like - Musixmatch often emits punctuation on
 * its own, ["I lied to you", ",", " I lied to you"], and sometimes hangs it off
 * the end of the word before, ["again   (", "Baby,"]. Either way a line could
 * wrap with a lone comma starting a row, or an opening bracket stranded at the
 * end of one.
 *
 * Rather than try to anticipate the tokenisation, the line is rebuilt from it.
 * The words are flattened back into text, carrying a timing per character, and
 * re-split on whitespace only - so a break can land where a space is and
 * nowhere else, which is the rule we actually wanted. Punctuation ends up
 * attached to its neighbour because nothing separates them. Runs of whitespace
 * collapse to one space on the way through, which also fixes the visible gap a
 * separate whitespace token used to render under `white-space: pre-wrap`.
 *
 * A second pass then handles the case the first cannot: a source that puts a
 * real space between a bracket and its word, "( Baby,". Those weld onto the
 * neighbour they belong to.
 *
 * Timings are absorbed with the text. A comma is not sung, so the word it joins
 * simply holds the sweep until the comma's end.
 */

const SPACE = /\s/;

// closing marks, and the dashes and apostrophes that behave like them
const TRAILING = /^[,.!?;:…)\]}»”’"'\-–—、。！？」』]+$/u;

// opening marks
const LEADING = /^[([{«“‘¿¡「『]+$/u;

export function weldPunctuation(words: RenderWord[]): RenderWord[] {
  if (!words.length) return [];

  /* Flatten, keeping a start and end for every character.
   *
   * The times are interpolated across the word rather than copied from it. A
   * source is free to hand back a whole phrase as one token, and re-splitting
   * that on whitespace would otherwise give every word in it the same span -
   * the sweep would cross the first word over the phrase's entire duration and
   * then jump the rest. Spreading the span over the characters means a split
   * token gets exactly the slice of time its text occupies, and a token that
   * was already a single word keeps its original start and end untouched. */
  let text = "";
  const cStart: number[] = [];
  const cEnd: number[] = [];
  for (const w of words) {
    const len = w.text.length;
    const span = w.endMs - w.startMs;
    for (let i = 0; i < len; i++) {
      cStart.push(w.startMs + (span * i) / len);
      cEnd.push(w.startMs + (span * (i + 1)) / len);
    }
    text += w.text;
  }

  // re-split on whitespace, and on nothing else
  const split: RenderWord[] = [];
  let i = 0;
  while (i < text.length) {
    if (SPACE.test(text[i])) {
      const from = i;
      while (i < text.length && SPACE.test(text[i])) i++;
      const prev = split[split.length - 1];
      if (prev) {
        // one space, on the token it follows - never its own token. Its slice
        // of time goes with it, or the sweep finishes the word early and then
        // stalls in the gap before the next one starts.
        prev.text += " ";
        prev.endMs = Math.max(prev.endMs, cEnd[i - 1] ?? cEnd[from]);
      }
      continue;
    }
    const from = i;
    while (i < text.length && !SPACE.test(text[i])) i++;
    split.push({ text: text.slice(from, i), startMs: cStart[from], endMs: cEnd[i - 1] });
  }

  // "( Baby," - a bracket the source separated from its word with a real space
  const out: RenderWord[] = [];
  let lead = "";
  let leadStart = 0;
  for (const w of split) {
    const bare = w.text.trim();
    if (!bare) continue;

    if (LEADING.test(bare)) {
      if (!lead) leadStart = w.startMs;
      lead += bare;
      continue;
    }

    const prev = out[out.length - 1];
    if (TRAILING.test(bare) && prev && !lead) {
      // drop the space the previous token is carrying, so it reads "you," not "you ,"
      prev.text = prev.text.replace(/\s+$/, "") + bare + (/\s$/.test(w.text) ? " " : "");
      prev.endMs = Math.max(prev.endMs, w.endMs);
      continue;
    }

    out.push({
      text: lead + w.text,
      startMs: lead ? Math.min(leadStart, w.startMs) : w.startMs,
      endMs: w.endMs,
    });
    lead = "";
  }

  if (lead) {
    const prev = out[out.length - 1];
    if (prev) prev.text = prev.text.replace(/\s+$/, "") + lead;
    else out.push({ text: lead, startMs: leadStart, endMs: leadStart });
  }

  // the sweep walks these in order, so they have to be monotonic even if a
  // source hands back overlapping or out-of-order timings
  for (let k = 0; k < out.length; k++) {
    if (k) out[k].startMs = Math.max(out[k].startMs, out[k - 1].endMs);
    out[k].endMs = Math.max(out[k].endMs, out[k].startMs);
  }

  return out;
}


/* The tokens a line is drawn from.
 *
 * Real per-word timings, or nothing. There used to be a third option: when a
 * source only gave line-level timings we spread the line's duration across its
 * words by character count and swept the light along that. It looked like
 * word-by-word and it was not - it was a guess about where in the line the
 * singer was, and it drifted against the vocal on every line long enough to
 * notice. That is what "word by word is often unsynced" actually was: not a bad
 * provider, an invented timing.
 *
 * So a line-level line now lights as a whole line, the instant it starts, which
 * is what Apple Music does with line-synced lyrics and what the sync-over-
 * richness rule demands. A single token starting and ending at the line's onset
 * makes `charPos` jump straight to fully-lit. Word-by-word is now only ever
 * shown when a provider actually measured the words. */
export function lyricWords(line: LyricLine, startMs: number, endMs: number): RenderWord[] {
  const effectiveStart = line.time_ms >= 0 ? line.time_ms : startMs;
  if (line.words.length) {
    return weldPunctuation(mapWords(line));
  }
  void endMs; // a line-level line has no interior timing to sweep across
  return [{ text: cleanSpaces(line.text), startMs: effectiveStart, endMs: effectiveStart }];
}

// one line, in every state

const PLACEHOLDER: RenderWord[] = [{ text: "♪", startMs: 0, endMs: 0 }];

/* How far the light has travelled, measured in characters rather than in words.
 *
 * The sweep used to run each word as its own little animation: progress 0 to 1
 * across that word, eased with a smoothstep, then hand over to the next. Two
 * things came out of that and both were visible. The easing slowed the light at
 * the start and end of every single word, so instead of gliding it pulsed once
 * per word - the jitter. And because each word got the same 0-to-1 regardless
 * of how long it lasted, a word sung in 90ms swept at ten times the speed of
 * one sung in 900ms; short words did not sweep so much as teleport.
 *
 * Measuring in characters fixes the second: the light covers ground in
 * proportion to how much text there is, so its speed tracks the vocal instead
 * of the tokenisation. */
function charPos(t: number, toks: RenderWord[], starts: number[], lens: number[], total: number): number {
  if (!toks.length) return 0;
  if (t <= toks[0].startMs) return 0;
  for (let i = 0; i < toks.length; i++) {
    const w = toks[i];
    if (t < w.startMs) return starts[i]; // a gap between words - hold at the edge
    if (t <= w.endMs) {
      const span = Math.max(w.endMs - w.startMs, 1);
      return starts[i] + lens[i] * ((t - w.startMs) / span);
    }
  }
  return total;
}

/* Time constant for the filter that chases it. The light is pulled toward the
   true position rather than snapped to it, so a word with a very short or badly
   quantised duration becomes a quick glide instead of a jump. It costs about
   this much lag against the vocal, which is small enough to read as the light
   having weight. */
const CHASE_MS = 80;

/* A lyric line, lit or not.
 *
 * Both states render exactly the same elements, and that is the entire point.
 * They used to differ: an inactive line was a paragraph of plain text, and an
 * active one was a row of inline-block spans carrying `text-wrap: balance`.
 * Inline-blocks and balanced text do not break in the same places as ordinary
 * text, so a line could wrap to a different number of rows the instant it lit
 * up - the text visibly re-flowing under the enlarge. Scaling was never what
 * moved it; swapping what it was made of was.
 *
 * With one set of spans for both states, the box the browser measures never
 * changes. The line wraps once, when it is laid out, and every state after that
 * is colour. It also makes the handoff a style change rather than a remount, so
 * the row's transitions carry across it instead of cutting. */
export const LyricRowText = memo(function LyricRowText({
  words,
  active,
  getClock,
  tone,
  size = 21,
  weight = 800,
  dim = 0.42,
  glowRgb = "255, 255, 255",
  inkRgb = "255, 255, 255",
  align = "left",
  tracking = "-0.022em",
}: {
  words: RenderWord[];
  active: boolean;
  getClock: () => number;
  tone: Tone;
  size?: number | string;
  weight?: number;
  dim?: number;
  /** the cover's own light, so the line reads as lit by the record */
  glowRgb?: string;
  /** near-white carrying the cover's hue - see lib/ambient */
  inkRgb?: string;
  align?: "left" | "right" | "center";
  /** letter-spacing. tight for lead lines; smaller secondary voices open up */
  tracking?: string;
}) {
  const spans = useRef<(HTMLSpanElement | null)[]>([]);
  const lastProps = useRef<{ a: string; b: string; g: string }[]>([]);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  /* The sweep is the only 60fps loop left on the page. Chromium throttles rAF
     for a hidden document but not for a merely unfocused window, so without
     this it kept running - and repainting - behind whatever the user switched
     to. The clock is read absolutely rather than accumulated, so it snaps back
     into place on the first frame after they return. */
  const awake = useWindowActive();

  const toks = words.length ? words : PLACEHOLDER;

  // where each token sits along the line, in characters
  const geom = useMemo(() => {
    const starts: number[] = [];
    const lens: number[] = [];
    let at = 0;
    for (const w of toks) {
      const n = Math.max(w.text.trim().length, 1);
      starts.push(at);
      lens.push(n);
      at += n;
    }
    return { starts, lens, total: Math.max(at, 1) };
  }, [toks]);

  useEffect(() => {
    if (!active) return;
    const { starts, lens, total } = geom;

    let raf = 0;
    let pos = charPos(getClock(), toks, starts, lens, total); // start where we are, not at zero
    let last = performance.now();
    lastProps.current = [];

    const tick = (now: number) => {
      const t = getClock();
      const dt = Math.min(now - last, 64); // a dropped frame must not overshoot
      last = now;

      // critically-damped chase, frame-rate independent
      pos += (charPos(t, toks, starts, lens, total) - pos) * (1 - Math.exp(-dt / CHASE_MS));

      for (let i = 0; i < toks.length; i++) {
        const el = spans.current[i];
        if (!el) continue;

        const n = lens[i];
        const local = (pos - starts[i]) / n;
        const fill = local < 0 ? 0 : local > 1 ? 1 : local;

        /* The soft edge is about a character and a half wide whatever the word
           is, rather than a fixed share of it - otherwise the leading edge is a
           hard line across "I" and a long smear across "definitely". */
        const band = Math.max(9, Math.min(46, (150 / n)));
        const a = Math.round((fill * (100 + band) - band) * 2) / 2;
        const aVal = `${a}%`;
        const bVal = `${a + band}%`;

        /* The bloom rides the light: it rises as the word fills and fades over
           the few characters after it, so the brightness travels through the
           line rather than the whole line switching on at once. */
        const over = pos - (starts[i] + n);
        const decay = over <= 0 ? 1 : Math.max(0, 1 - over / 5);
        const gVal = (Math.round(fill * decay * 20) / 20).toFixed(2);

        const prev = lastProps.current[i];
        if (!prev || prev.a !== aVal || prev.b !== bVal || prev.g !== gVal) {
          lastProps.current[i] = { a: aVal, b: bVal, g: gVal };
          el.style.setProperty("--a", aVal);
          el.style.setProperty("--b", bVal);
          el.style.setProperty("--g", gVal);
        }
      }

      // keep going until the light has run off the end and the last bloom died
      if (pos < total + 6 && isPlaying && awake) {
        raf = requestAnimationFrame(tick);
      }
    };

    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [toks, geom, getClock, isPlaying, awake, active]);

  const unlit = toneText(tone, inkRgb);

  return (
    <p
      className="lyr-line"
      style={{
        margin: 0,
        fontSize: size,
        lineHeight: 1.26,
        letterSpacing: tracking,
        fontWeight: weight,
        textAlign: align,
        ["--glow" as string]: glowRgb,
      }}
    >
      {toks.map((w, i) => (
        <span
          key={i}
          className={active ? "lyr-word lyr-word-lit" : "lyr-word"}
          ref={(el) => {
            spans.current[i] = el;
          }}
          style={
            active
              ? { backgroundImage: `linear-gradient(90deg, rgba(${inkRgb}, 0.99) var(--a,-50%), rgba(${inkRgb}, ${dim}) var(--b,-10%))` }
              : unlit
          }
        >
          {w.text}
        </span>
      ))}
    </p>
  );
});
