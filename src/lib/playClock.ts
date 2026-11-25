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

/* A seek, a track start or a resume anchors the clock where playback was asked
   to be, the moment it was asked. The audio of it only reaches the speaker once
   the player has reopened the stream and queued some of it (tens of ms for a
   seek, more for a track start), and its position stays put until then. So the
   first reports after an anchor are the truth about that gap, not jitter: they
   are taken as they are. Slewing them in by halves left a seek's gap (~30-60ms
   on Apple Music) in the lyrics for seconds after every click. */
const TRUSTED_AFTER_ANCHOR = 2;
let trusted = 0;

function setAnchor(posMs: number, playing: boolean, rate: number): void {
  anchorPos = posMs;
  anchorAt = now();
  anchorPlaying = playing;
  anchorRate = rate > 0 ? rate : 1;
}

/** Re-anchor on a seek / play-state change / track change. */
export function anchorPlayhead(posMs: number, playing: boolean, rate = 1): void {
  setAnchor(posMs, playing, rate);
  trusted = TRUSTED_AFTER_ANCHOR;
}

/* Reports land about once a second with a few ms of IPC and quantisation
   jitter. Snapping to each one would twitch the word sweep by that much, so a
   small disagreement is closed halfway per report (a constant error is gone
   within a few seconds) while anything larger - a seek, a stall, a rebuffer -
   is taken at once. */
const SLEW_LIMIT_MS = 120;

/** Take in a position report from the player. */
export function reportPlayhead(posMs: number, playing: boolean, rate = 1): void {
  if (playing && anchorPlaying && trusted > 0) {
    trusted -= 1;
    setAnchor(posMs, playing, rate);
    return;
  }
  if (playing && anchorPlaying) {
    const predicted = playheadMs();
    const drift = posMs - predicted;
    if (Math.abs(drift) < SLEW_LIMIT_MS) {
      setAnchor(predicted + drift * 0.5, playing, rate);
      return;
    }
  }
  // a stall or a jump nobody asked for: a discontinuity like any other
  anchorPlayhead(posMs, playing, rate);
}

/** Where playback is right now, in ms. A field read and a subtraction. */
export function playheadMs(): number {
  return anchorPlaying ? anchorPos + (now() - anchorAt) * anchorRate : anchorPos;
}

export function playheadPlaying(): boolean {
  return anchorPlaying;
}
