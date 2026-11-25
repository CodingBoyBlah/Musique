/* The one playhead clock.
 *
 * Position reports from the player are the only truth. Between them the
 * playhead is the last report plus elapsed performance.now() - which keeps
 * running while the window is unfocused or hidden, unlike the 1s interval that
 * used to advance it by +1000 per tick. That interval is throttled in the
 * background, so the playhead fell behind the music while the app sat behind
 * another window, and every lyric surface that mounted afterwards (reopening
 * the panel, entering immersive) started from that stale figure.
 *
 * Module-level on purpose: it outlives every component, so mounting or
 * unmounting a view can never reset it. The player store feeds it (see the
 * subscription at the bottom of store/player.store.ts); nothing here imports
 * the store, which keeps the module graph acyclic. */

let anchorPos = 0;
let anchorAt = 0;
let anchorPlaying = false;
/** The playback rate (podcast speed) the player reports. */
let anchorRate = 1;
/** How much faster or slower than `anchorRate` the clock runs while it closes a small
 *  disagreement with the player (see `reportPlayhead`). */
let correction = 1;
/** When the clock last jumped: a seek, a track start, a resume. */
let jumpedAt = -Infinity;
/** Far-off reports in a row since the last jump. */
let strays = 0;

const now = () => (typeof performance !== "undefined" ? performance.now() : Date.now());

function setAnchor(posMs: number, playing: boolean, rate: number): void {
  anchorPos = posMs;
  anchorAt = now();
  anchorPlaying = playing;
  anchorRate = rate > 0 ? rate : 1;
}

/** Re-anchor on a seek / play-state change / track change. */
export function anchorPlayhead(posMs: number, playing: boolean, rate = 1): void {
  setAnchor(posMs, playing, rate);
  correction = 1;
  jumpedAt = now();
  strays = 0;
}

/* Reports land about once a second with a few ms of IPC and quantisation
   jitter, and right after a seek or a track start they also carry the moment
   the player took to reopen the stream (tens of ms). Jumping to each report
   moved the playhead backwards by that much, and the line highlight and word
   sweep visibly stuttered on every lyric click. So a small disagreement is
   never jumped: the clock runs a little faster or slower until the next report
   (about a second), arriving where the player is without ever going back.
   Anything larger - a stall, a rebuffer, a jump nobody asked for - is taken at
   once. */
const SLEW_LIMIT_MS = 250;
/** How long a small disagreement takes to close. */
const CORRECT_OVER_MS = 1_000;
/** The furthest the clock strays from real time while it corrects: 10% closes a seek's
 *  start-up gap (up to 100 ms) within the second, and is not something the eye can follow. */
const MAX_CORRECTION = 0.1;
/* Just after a jump, a far-off report is most likely one the player sent
   before it acted on the seek. One such report is ignored; two in a row are
   the truth. */
const STRAY_WINDOW_MS = 1_000;

/** Take in a position report from the player. */
export function reportPlayhead(posMs: number, playing: boolean, rate = 1): void {
  if (playing && anchorPlaying) {
    const predicted = playheadMs();
    const drift = posMs - predicted;
    if (Math.abs(drift) < SLEW_LIMIT_MS) {
      strays = 0;
      setAnchor(predicted, playing, rate);
      const wanted = 1 + drift / (CORRECT_OVER_MS * anchorRate);
      correction = Math.min(1 + MAX_CORRECTION, Math.max(1 - MAX_CORRECTION, wanted));
      return;
    }
    if (now() - jumpedAt < STRAY_WINDOW_MS && strays === 0) {
      strays = 1;
      return;
    }
  }
  // a stall or a jump nobody asked for: a discontinuity like any other
  anchorPlayhead(posMs, playing, rate);
}

/** Where playback is right now, in ms. A field read and a subtraction. */
export function playheadMs(): number {
  if (!anchorPlaying) return anchorPos;
  const elapsed = now() - anchorAt;
  // the correction only runs for its window: a report that never comes must not leave
  // the clock running fast or slow
  return anchorPos + elapsed * anchorRate + Math.min(elapsed, CORRECT_OVER_MS) * anchorRate * (correction - 1);
}

export function playheadPlaying(): boolean {
  return anchorPlaying;
}
