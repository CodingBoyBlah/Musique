import { useEffect, useRef, type RefObject } from "react";
import { isTopKeyLayer, popKeyLayer, pushKeyLayer } from "../lib/keyLayers";

const FOCUSABLE =
  'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';

/* The three things every modal owes a keyboard user:
   - Escape closes it (the way out, always)
   - Tab stays inside it while it's open
   - focus goes back to whatever opened it once it closes

   `initialFocus` picks the element to focus on open - the safe default action
   for a confirmation, the text field for a form. Falls back to the first
   focusable element in the dialog.

   Open modals form a stack (an update prompt can be up when the quit dialog
   opens over it). Only the topmost one answers Escape and traps Tab - every
   modal listens on window, and stopPropagation can't stop sibling listeners
   on the same target, so without the stack one Escape closed both and the
   lower dialog's trap pulled focus out of the upper one. See lib/keyLayers. */

export function useModalA11y(
  ref: RefObject<HTMLElement | null>,
  open: boolean,
  onClose: () => void,
  initialFocus?: RefObject<HTMLElement | null>,
) {
  const closeRef = useRef(onClose);
  closeRef.current = onClose;

  useEffect(() => {
    if (!open) return;
    const returnTo = document.activeElement as HTMLElement | null;
    const id = pushKeyLayer();

    const raf = requestAnimationFrame(() => {
      const target =
        initialFocus?.current ?? ref.current?.querySelector<HTMLElement>(FOCUSABLE) ?? ref.current;
      target?.focus({ preventScroll: true });
    });

    const onKey = (e: KeyboardEvent) => {
      if (!isTopKeyLayer(id)) return;
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        closeRef.current();
        return;
      }
      if (e.key !== "Tab" || !ref.current) return;
      const els = Array.from(ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
        (el) => el.offsetParent !== null || el === document.activeElement,
      );
      if (!els.length) {
        e.preventDefault();
        return;
      }
      const first = els[0];
      const last = els[els.length - 1];
      const active = document.activeElement as HTMLElement | null;
      if (e.shiftKey && (active === first || !ref.current.contains(active))) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && (active === last || !ref.current.contains(active))) {
        e.preventDefault();
        first.focus();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("keydown", onKey, true);
      popKeyLayer(id);
      if (returnTo && document.contains(returnTo)) returnTo.focus?.({ preventScroll: true });
    };
    // initialFocus is a ref; its identity is stable
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, ref]);
}
