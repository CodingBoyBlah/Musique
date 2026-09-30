import { useEffect, useRef, type RefObject } from "react";
import { flushSync } from "react-dom";
import { RAIL_EASE_CSS } from "./motion";

/* Tile grids through a right-rail slide.

A grid that simply followed the sliding edge would snap through every column
count on the way. So at the first frame of a slide each grid (data-rail-lock)
is pinned to the width it will have once the rail has landed, re-columns
exactly once, right then, and its cards are FLIPped from where they were to
where they now are: measure, pin, measure again, and play each card back with
the Web Animations API. A transform on the compositor - no React render and
nothing on the main thread per frame - on the same curve and clock as the
edge, so the cards move while the edge moves rather than after it. Rendering
every card through React for framer to measure is what used to stall the
frame a close landed on.

A CSS auto-fill grid re-columns the moment its width changes. A grid whose
column count comes from JS (EvenGrid, the quick actions, the stat tiles)
registers useRailPin, and is told synchronously, between the pin and the
second measure, so it has re-rendered its columns by the time it is read.

On a close the pinned width is wider than the page still is, for the length of
the slide. The page clips it, and the edge sliding away reveals cards already
sitting in their final cells. */

const RAIL_PIN = "railpin";

/** Pin every rail-locked grid under `root` to its width after the page has
 * grown by `grow` px (negative while a panel opens), FLIP its cards over `ms`,
 * and return a function that releases the pins. Must not be called during a
 * React render or commit - JS grids re-render synchronously from inside it. */
export function pinRailGrids(root: HTMLElement, grow: number, ms: number): () => void {
  const grids = Array.from(root.querySelectorAll<HTMLElement>("[data-rail-lock]"));
  if (grids.length === 0) return () => {};
  const view = root.getBoundingClientRect();

  // read everything, then write, then read again: one forced layout for the
  // whole page rather than one per card
  const widths = grids.map((g) => g.getBoundingClientRect().width);
  const before = new Map<Element, DOMRect>();
  for (const g of grids) for (const c of Array.from(g.children)) before.set(c, c.getBoundingClientRect());

  const prev = grids.map((g) => g.style.width);
  grids.forEach((g, i) => {
    g.style.width = `${Math.max(0, widths[i] + grow)}px`;
  });
  for (const g of grids) g.dispatchEvent(new Event(RAIL_PIN));

  const opts = { duration: ms, easing: RAIL_EASE_CSS };
  for (const g of grids) {
    const mode: FlipMode = g.dataset.railLock === "flip-size" ? "size" : "scale";
    for (const card of Array.from(g.children) as HTMLElement[]) {
      playFrom(card, before.get(card), mode, view, opts);
    }
  }

  return () => {
    grids.forEach((g, i) => {
      g.style.width = prev[i];
    });
  };
}

/* How a card travels from where it was.

"scale": one scale for both axes plus the move. Right for tiles - the artwork
is square and the text under it should grow with it, not stretch - and it is
pure compositor work.

"size": the move, and the card's real width animated from the old to the new.
For wide, short cards (the Home quick actions): a uniform scale shrank a 60px
card to 40px and its text with it, and a width-only scale would stretch the
text. Animating width re-lays out the card itself each frame, which for a
handful of small cards costs nothing. */
export type FlipMode = "scale" | "size";

function playFrom(card: HTMLElement, a: DOMRect | undefined, mode: FlipMode, view: DOMRect, opts: KeyframeAnimationOptions) {
  const b = card.getBoundingClientRect();
  if (!a) {
    // a JS grid showing one more tile than it did: let it arrive
    if (b.bottom >= view.top && b.top <= view.bottom) card.animate([{ opacity: 0 }, { opacity: 1 }], opts);
    return;
  }
  // off screen both before and after: nobody would see it move
  if ((a.bottom < view.top && b.bottom < view.top) || (a.top > view.bottom && b.top > view.bottom)) return;
  const dx = a.left - b.left;
  const dy = a.top - b.top;
  const s = b.width > 0 ? a.width / b.width : 1;
  if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5 && Math.abs(s - 1) < 0.004) return;
  if (mode === "size") {
    // width relative to the card's live grid track, not a pixel target: the
    // grid may still be resizing (a window drag, the sidebar folding), and a
    // fixed target would land off by however far it moved and then snap.
    // Ending on 100% hands the card straight back to its track.
    card.animate(
      [
        { transform: `translate(${dx}px, ${dy}px)`, width: `calc(100% + ${a.width - b.width}px)` },
        { transform: "none", width: "100%" },
      ],
      opts,
    );
    return;
  }
  card.animate(
    [
      { transformOrigin: "0 0", transform: `translate(${dx}px, ${dy}px) scale(${s})` },
      { transformOrigin: "0 0", transform: "none" },
    ],
    opts,
  );
}

/** Re-column a grid and move its cards to their new cells rather than letting
 * them jump: measure, run `change` (which must update the DOM synchronously,
 * e.g. a flushSync'd setState), and play each card from where it was. For a
 * JS-columned grid reacting to its own width outside a rail slide - a window
 * resize, the sidebar folding. */
export function flipChildren(grid: HTMLElement, change: () => void, mode: FlipMode, ms = 320): void {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    change();
    return;
  }
  const kids = Array.from(grid.children) as HTMLElement[];
  const before = new Map<Element, DOMRect>(kids.map((k) => [k, k.getBoundingClientRect()]));
  change();
  const opts = { duration: ms, easing: RAIL_EASE_CSS };
  // measure against the window, not the grid: a card may be moving into a
  // row that did not exist before
  const port = new DOMRect(0, 0, window.innerWidth, window.innerHeight);
  for (const card of Array.from(grid.children) as HTMLElement[]) playFrom(card, before.get(card), mode, port, opts);
}

/** For a grid that works out its own columns in JS: re-measure, synchronously,
 * when Layout pins it for a rail slide. `measure` reads the element's width
 * and sets the column state; it is flushed before the FLIP reads the cards. */
export function useRailPin(ref: RefObject<HTMLElement | null>, measure: () => void): void {
  const latest = useRef(measure);
  latest.current = measure;
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const onPin = () => flushSync(() => latest.current());
    el.addEventListener(RAIL_PIN, onPin);
    return () => el.removeEventListener(RAIL_PIN, onPin);
  }, [ref]);
}
