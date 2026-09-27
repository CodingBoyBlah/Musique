import { useEffect, useRef, type RefObject } from "react";
import { isTopKeyLayer, popKeyLayer, pushKeyLayer } from "../lib/keyLayers";

/* Keyboard for an open menu: arrows / Home / End move between its
   [role="menuitem"]s, Escape and Tab close it. Focus lands on the first item
   (or the checked one) when the menu opens and goes back to whatever opened it
   when it closes - so a keyboard user is never left stranded on <body>.

   Programmatic focus only shows a :focus-visible ring when the last input was
   the keyboard, so a mouse-opened menu doesn't light up its first row. */
export function useMenuKeyboard(
  ref: RefObject<HTMLElement | null>,
  open: boolean,
  onClose: () => void,
) {
  // through a ref: an inline onClose would otherwise re-run the effect on
  // every render and yank focus back to the first item
  const closeRef = useRef(onClose);
  closeRef.current = onClose;

  useEffect(() => {
    if (!open) return;
    const layer = pushKeyLayer();
    const returnTo = document.activeElement as HTMLElement | null;
    const items = () =>
      Array.from(ref.current?.querySelectorAll<HTMLElement>('[role="menuitem"],[role="menuitemradio"]') ?? []);

    // after paint, so the menu exists
    const raf = requestAnimationFrame(() => {
      const list = items();
      const checked = list.find((el) => el.getAttribute("aria-checked") === "true");
      (checked ?? list[0])?.focus({ preventScroll: true });
    });

    const onKey = (e: KeyboardEvent) => {
      if (!isTopKeyLayer(layer)) return;
      const list = items();
      if (!list.length) return;
      const i = list.indexOf(document.activeElement as HTMLElement);
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          list[(i + 1 + list.length) % list.length]?.focus();
          break;
        case "ArrowUp":
          e.preventDefault();
          list[(i - 1 + list.length) % list.length]?.focus();
          break;
        case "Home":
          e.preventDefault();
          list[0]?.focus();
          break;
        case "End":
          e.preventDefault();
          list[list.length - 1]?.focus();
          break;
        case "Escape":
          e.preventDefault();
          e.stopPropagation();
          closeRef.current();
          break;
        case "Tab":
          closeRef.current();
          break;
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("keydown", onKey, true);
      popKeyLayer(layer);
      // only hand focus back if it was inside the menu (a click elsewhere
      // already moved it somewhere the user chose)
      const active = document.activeElement;
      if (returnTo && (!active || active === document.body || ref.current?.contains(active))) {
        returnTo.focus?.({ preventScroll: true });
      }
    };
  }, [open, ref]);
}
