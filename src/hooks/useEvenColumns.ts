import { useLayoutEffect, useState, type RefObject } from "react";
import { useRailPin } from "../lib/railFlip";

/* the most columns that fit AND divide the item count evenly, so a row of
tiles never leaves one orphan hanging on the next line (6 tiles -> 6, 3, 2 or
1 across as the width shrinks, never 5 + 1). */
export function evenColumns(width: number, count: number, minItem: number, gap: number): number {
  if (count <= 0 || width <= 0) return 1;
  const fit = Math.max(1, Math.floor((width + gap) / (minItem + gap)));
  for (let c = Math.min(fit, count); c > 1; c--) {
    if (count % c === 0) return c;
  }
  return 1;
}

export function useEvenColumns(ref: RefObject<HTMLElement | null>, count: number, minItem: number, gap: number): number {
  const [cols, setCols] = useState(1);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const measure = () => setCols(evenColumns(el.clientWidth, count, minItem, gap));
    measure();
    if (typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [ref, count, minItem, gap]);
  // a right-rail slide pins the grid: re-column now, before the FLIP reads it
  useRailPin(ref, () => {
    const el = ref.current;
    if (el) setCols(evenColumns(el.clientWidth, count, minItem, gap));
  });
  return cols;
}
