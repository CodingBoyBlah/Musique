/* A critically damped spring on an element's scrollTop.
 *
 * Both lyric surfaces follow the singer with this. It replaces two different
 * motions: the immersive view's fixed 560ms ease-out-quart, which restarted at
 * the curve's peak speed on every new line whatever the column was already
 * doing (on a fast verse, lines land less than 560ms apart, so the scroll
 * lurched each time), and the side panel's native smooth scroll, whose curve
 * and length the browser picks for itself.
 *
 * A spring has no duration to restart. A new line only moves the target; the
 * position and velocity carry straight through, so back-to-back lines blend
 * into one continuous glide rather than a series of kicks. It always starts
 * from the live scrollTop, so it can be interrupted and picked back up
 * anywhere without a jump.
 *
 * Critically damped (zeta = 1): it settles as fast as a spring can without
 * overshooting - a lyric column bouncing past its line would read as a
 * mistake. `response` is Apple's term: roughly how long it takes to get there.
 *
 * Integrated in closed form each frame rather than stepped, so it is exact at
 * any frame rate and a dropped frame cannot make it overshoot. */

export interface ScrollSpring {
  /** move toward `target`, keeping whatever velocity is already in flight */
  to: (target: number) => void;
  /** place instantly, no motion */
  jump: (target: number) => void;
  /** halt where it is (the user took over) */
  stop: () => void;
  readonly running: boolean;
}

export const LYRIC_SCROLL_RESPONSE = 0.45;

export function createScrollSpring(
  el: HTMLElement,
  response = LYRIC_SCROLL_RESPONSE,
): ScrollSpring {
  const omega = (2 * Math.PI) / response;

  let raf = 0;
  let pos = 0;      // our own float position; scrollTop may round
  let vel = 0;      // px/s
  let target = 0;
  let last = 0;

  const maxScroll = () => Math.max(0, el.scrollHeight - el.clientHeight);
  const clampT = (t: number) => Math.min(Math.max(t, 0), maxScroll());

  const halt = () => {
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
    vel = 0;
  };

  const step = (now: number) => {
    const dt = Math.min((now - last) / 1000, 0.064);
    last = now;

    // x(t) = (C1 + C2 t) e^(-wt), with x measured from the target
    const d0 = pos - target;
    const c2 = vel + omega * d0;
    const decay = Math.exp(-omega * dt);
    const d = (d0 + c2 * dt) * decay;
    vel = (c2 - omega * (d0 + c2 * dt)) * decay;
    pos = target + d;

    if (Math.abs(d) < 0.5 && Math.abs(vel) < 8) {
      pos = target;
      el.scrollTop = target;
      halt();
      return;
    }
    el.scrollTop = pos;
    raf = requestAnimationFrame(step);
  };

  return {
    to(t) {
      target = clampT(t);
      if (!raf) {
        // start from what is on screen, not from where we last left it
        pos = el.scrollTop;
        vel = 0;
        if (Math.abs(pos - target) < 1) return;
        last = performance.now();
        raf = requestAnimationFrame(step);
      }
    },
    jump(t) {
      halt();
      target = clampT(t);
      pos = target;
      el.scrollTop = target;
    },
    stop: halt,
    get running() {
      return raf !== 0;
    },
  };
}
