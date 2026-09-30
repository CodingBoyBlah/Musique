import { invoke } from "@tauri-apps/api/core";

/* Real output latency, measured rather than guessed.
 *
 * `positionMs` from librespot is the DECODER's position. The audio the listener
 * is actually hearing left the decoder some milliseconds ago and is still
 * sitting in our sink queue and then the device buffer. So what's heard is
 * `reported - queued`, and lyrics keyed to the reported position run AHEAD of
 * the sound.
 *
 * This used to be papered over with a hardcoded `lyricsOffsetMs: -250` default,
 * which is wrong on any machine whose buffer differs and wrong again the moment
 * the user changes the buffer size. The backend now reports the real figure
 * (queued frames / sample rate + the device buffer cpal actually granted), so
 * the manual offset goes back to being what it should always have been: a
 * personal preference, defaulting to zero.
 *
 * Since the playout queue (src-tauri/src/playout.rs) keeps seconds of audio
 * ready ahead of the speaker, both backends report positions as HEARD - taken
 * where the queue meets the device - so the figure here is only the device
 * buffer past that point, not the queue.
 *
 * Deliberately NOT a react hook or a store value. The clock reads it inside
 * `getClock()`, so a latency change must not re-render anything - this codebase
 * has an explicit no-re-render-cascade mandate. One module-level number, one
 * slow timer, zero subscriptions. */

/* Used until the first successful reading lands. Reading 0 too early would
 * leave lyrics ~350ms early on the first track, which is exactly the bug this
 * file exists to remove - so we start from the old default's assumption and let
 * the real measurement replace it a moment later. */
const ASSUMED_MS = 250;

/* Anything outside this is a bad reading, not a real buffer, and is ignored
 * rather than allowed to yank the clock around. */
const MAX_PLAUSIBLE_MS = 2000;

let latencyMs = ASSUMED_MS;
let timer: ReturnType<typeof setInterval> | null = null;
let consumers = 0;

async function poll(): Promise<void> {
  try {
    const v = await invoke<number>("get_output_latency_ms");
    // 0 means "no sink" (not initialised, or the silent fallback) - keep the
    // last good reading rather than snapping the clock to no correction at all
    if (typeof v === "number" && v > 0 && v <= MAX_PLAUSIBLE_MS) {
      latencyMs = v;
    }
  } catch {
    // playback not up yet, or the command is unavailable - keep what we have
  }
}

/** Current output latency in ms. Safe to call every frame; it's a field read. */
export function getOutputLatencyMs(): number {
  return latencyMs;
}

/* The device and its buffer only change when the user switches output or
 * changes the buffer size, so this is a slow background check, not a poll loop.
 * Ref-counted so several lyric surfaces (panel + immersive) share one timer. */
export function startLatencyPolling(): () => void {
  consumers += 1;
  if (!timer) {
    void poll();
    timer = setInterval(() => void poll(), 10_000);
  }
  return () => {
    consumers -= 1;
    if (consumers <= 0 && timer) {
      clearInterval(timer);
      timer = null;
      consumers = 0;
    }
  };
}
