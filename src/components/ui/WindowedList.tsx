import { useEffect, useLayoutEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";

export interface WindowRange {
  start: number;
  end: number;
  topSpacerPx: number;
  bottomSpacerPx: number;
}

export function computeWindowRange({
  count,
  threshold,
  pitch,
  scrollTop,
  offsetTop,
  viewportHeight,
  overscan,
}: {
  count: number;
  threshold: number;
  pitch: number;
  scrollTop: number;
  offsetTop: number;
  viewportHeight: number;
  overscan: number;
}): WindowRange {
  if (count === 0) {
    return { start: 0, end: 0, topSpacerPx: 0, bottomSpacerPx: 0 };
  }
  if (count <= threshold || pitch <= 0 || viewportHeight <= 0) {
    return { start: 0, end: count, topSpacerPx: 0, bottomSpacerPx: 0 };
  }
  const relTop = Math.max(0, scrollTop - offsetTop);
  const firstVisible = Math.floor(relTop / pitch);
  const visibleCount = Math.ceil(viewportHeight / pitch) + 1;
  const start = Math.max(0, firstVisible - overscan);
  const end = Math.min(count, firstVisible + visibleCount + overscan);
  return {
    start,
    end,
    topSpacerPx: start * pitch,
    bottomSpacerPx: (count - end) * pitch,
  };
}

function findScrollParent(el: HTMLElement | null): HTMLElement | null {
  let cur = el?.parentElement ?? null;
  while (cur) {
    if (cur.tagName === "MAIN") return cur;
    const oy = getComputedStyle(cur).overflowY;
    if (oy === "auto" || oy === "scroll") return cur;
    cur = cur.parentElement;
  }
  return null;
}

export function WindowedList<T>({
  items,
  getKey,
  estimateHeight = 59,
  threshold = 60,
  overscan = 16,
  className,
  style,
  renderItem,
}: {
  items: T[];
  getKey: (item: T, index: number) => string;
  estimateHeight?: number;
  threshold?: number;
  overscan?: number;
  className?: string;
  style?: CSSProperties;
  renderItem: (item: T, index: number) => ReactNode;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const active = items.length > threshold;

  const [windowState, setWindowState] = useState<{ start: number; end: number; pitch: number }>(() => ({
    start: 0,
    end: active ? Math.min(items.length, threshold) : items.length,
    pitch: estimateHeight,
  }));
  const pitchRef = useRef(windowState.pitch);
  pitchRef.current = windowState.pitch;

  useLayoutEffect(() => {
    if (!active) {
      setWindowState((prev) =>
        prev.start === 0 && prev.end === items.length
          ? prev
          : { start: 0, end: items.length, pitch: prev.pitch },
      );
      return;
    }

    const root = containerRef.current;
    const scroller = findScrollParent(root);
    if (!root || !scroller) return;

    let raf = 0;
    const update = () => {
      raf = 0;
      const children = root.children;
      // Find first two rendered item nodes (excluding spacer divs marked data-wl-spacer)
      let firstEl: HTMLElement | null = null;
      let secondEl: HTMLElement | null = null;
      for (let i = 0; i < children.length; i++) {
        const ch = children[i] as HTMLElement;
        if (ch.dataset.wlSpacer) continue;
        if (!firstEl) firstEl = ch;
        else {
          secondEl = ch;
          break;
        }
      }
      let nextPitch = pitchRef.current;
      if (firstEl && secondEl) {
        const measured = secondEl.offsetTop - firstEl.offsetTop;
        if (measured > 10) nextPitch = measured;
      } else if (firstEl && firstEl.offsetHeight > 10) {
        nextPitch = firstEl.offsetHeight;
      }

      const rootRect = root.getBoundingClientRect();
      const scrollerRect = scroller.getBoundingClientRect();
      const offsetTop = rootRect.top - scrollerRect.top + scroller.scrollTop;
      const next = computeWindowRange({
        count: items.length,
        threshold,
        pitch: nextPitch,
        scrollTop: scroller.scrollTop,
        offsetTop,
        viewportHeight: scroller.clientHeight,
        overscan,
      });
      setWindowState((prev) =>
        prev.start === next.start && prev.end === next.end && prev.pitch === nextPitch
          ? prev
          : { start: next.start, end: next.end, pitch: nextPitch },
      );
    };

    const schedule = () => {
      if (!raf) raf = requestAnimationFrame(update);
    };

    update();
    scroller.addEventListener("scroll", schedule, { passive: true });
    const ro = typeof ResizeObserver !== "undefined" ? new ResizeObserver(schedule) : null;
    ro?.observe(scroller);
    ro?.observe(root);

    return () => {
      scroller.removeEventListener("scroll", schedule);
      ro?.disconnect();
      if (raf) cancelAnimationFrame(raf);
    };
  }, [active, items.length, threshold, overscan]);

  useEffect(() => {
    if (!active) return;
    setWindowState((prev) => {
      const clampedStart = Math.min(prev.start, Math.max(0, items.length - 1));
      const clampedEnd = Math.min(items.length, Math.max(clampedStart, prev.end));
      return clampedStart === prev.start && clampedEnd === prev.end
        ? prev
        : { start: clampedStart, end: clampedEnd, pitch: prev.pitch };
    });
  }, [active, items.length]);

  const start = active ? Math.min(windowState.start, items.length) : 0;
  const end = active ? Math.min(windowState.end, items.length) : items.length;
  const pitch = windowState.pitch;
  const topPx = active ? start * pitch : 0;
  const bottomPx = active ? Math.max(0, (items.length - end) * pitch) : 0;

  return (
    <div ref={containerRef} className={className} style={style}>
      {topPx > 0 && <div data-wl-spacer="top" aria-hidden style={{ height: topPx, flexShrink: 0 }} />}
      {items.slice(start, end).map((item, offset) => {
        const idx = start + offset;
        return (
          <div key={getKey(item, idx)} style={{ display: "flex", flexDirection: "column" }}>
            {renderItem(item, idx)}
          </div>
        );
      })}
      {bottomPx > 0 && <div data-wl-spacer="bottom" aria-hidden style={{ height: bottomPx, flexShrink: 0 }} />}
    </div>
  );
}
