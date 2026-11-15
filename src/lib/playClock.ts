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
let anchorRate = 1;

const now = () => (typeof performance !== "undefined" ? performance.now() : Date.now());

/** Re-anchor on a real position report (or a seek / play-state change). */
export function anchorPlayhead(posMs: number, playing: boolean, rate = 1): void {
  anchorPos = posMs;
  anchorAt = now();
  anchorPlaying = playing;
  anchorRate = rate > 0 ? rate : 1;
}

/* Reports land about once a second with a few ms of IPC and quantisation
   jitter. Snapping to each one would twitch the word sweep by that much, so a
   small disagreement is closed halfway per report (a constant error is gone
   within a few seconds) while anything larger - a seek, a stall, a rebuffer -
   is taken at once. */
const SLEW_LIMIT_MS = 120;

/** Take in a position report from the player. */
export function reportPlayhead(posMs: number, playing: boolean, rate = 1): void {
  if (playing && anchorPlaying) {
    const predicted = playheadMs();
    const drift = posMs - predicted;
    if (Math.abs(drift) < SLEW_LIMIT_MS) {
      anchorPlayhead(predicted + drift * 0.5, playing, rate);
      return;
    }
  }
  anchorPlayhead(posMs, playing, rate);
}

/** Where playback is right now, in ms. A field read and a subtraction. */
export function playheadMs(): number {
  return anchorPlaying ? anchorPos + (now() - anchorAt) * anchorRate : anchorPos;
}

export function playheadPlaying(): boolean {
  return anchorPlaying;
}
