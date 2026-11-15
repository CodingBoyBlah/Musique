import { useEffect, useRef, useState } from "react";

const HOLD = "[data-idle-hold]";

/* Video-player controls: shown while the mouse moves, gone a moment after it
 * stops.
 *
 * Returns `idle`, true once nothing has happened for `ms`. Controls opt in to
 * holding it off by carrying `data-idle-hold`. Any real pointer
 * movement, press, wheel or key wakes the controls and restarts the clock.
 * They are held up, not just woken, while the pointer rests on them or holds a
 * press (a scrub in progress), and while keyboard focus is visibly inside them -
 * vanishing from under a resting cursor or a tabbed-to button is the thing that
 * makes this pattern feel hostile.
 *
 * Windows WebView2 sends the odd synthetic pointermove at the same coordinates
 * with the mouse untouched, which would keep the controls up forever, so a move
 * only counts if the pointer actually went somewhere - by any amount at all,
 * so the slightest nudge brings the controls back. */

export function useIdleControls(active: boolean, ms = 1800): boolean {
  const [idle, setIdle] = useState(false);
  const hovering = useRef(false);
  const pressed = useRef(false);

  useEffect(() => {
    if (!active) {
      setIdle(false);
      return;
    }
    let timer = 0;
    let lastX = -1, lastY = -1;

    const held = () => {
      const f = document.activeElement;
      const focused = !!f?.closest(HOLD) && f.matches(":focus-visible");
      return hovering.current || pressed.current || focused;
    };
    const arm = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        if (held()) arm();
        else setIdle(true);
      }, ms);
    };
    const wake = () => {
      setIdle(false);
      arm();
    };
    const onMove = (e: PointerEvent) => {
      if (e.clientX === lastX && e.clientY === lastY) return;
      lastX = e.clientX;
      lastY = e.clientY;
      hovering.current = !!(e.target as Element | null)?.closest?.(HOLD);
      wake();
    };
    const onDown = () => { pressed.current = true; wake(); };
    const onUp = () => { pressed.current = false; arm(); };
    // the pointer leaving the window can't be resting on the controls
    const onLeave = () => { hovering.current = false; };

    /* Capture phase throughout: a control that stops propagation (a slider
       eating its pointerdown, a key handler eating its keydown) must still
       count as activity, or the dock stays hidden while you use it. */
    const cap = { capture: true, passive: true } as const;
    window.addEventListener("pointermove", onMove, cap);
    window.addEventListener("pointerdown", onDown, cap);
    window.addEventListener("pointerup", onUp, cap);
    window.addEventListener("pointercancel", onUp, cap);
    window.addEventListener("wheel", wake, cap);
    window.addEventListener("keydown", wake, cap);
    document.documentElement.addEventListener("pointerleave", onLeave);
    arm();
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("pointermove", onMove, cap);
      window.removeEventListener("pointerdown", onDown, cap);
      window.removeEventListener("pointerup", onUp, cap);
      window.removeEventListener("pointercancel", onUp, cap);
      window.removeEventListener("wheel", wake, cap);
      window.removeEventListener("keydown", wake, cap);
      document.documentElement.removeEventListener("pointerleave", onLeave);
      hovering.current = false;
      pressed.current = false;
    };
  }, [active, ms]);

  return idle;
}
