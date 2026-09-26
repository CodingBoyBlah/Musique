import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { ChevronLeft, ChevronRight } from "@/lib/icons";

/* one horizontal shelf for the whole app (Home rows, Album "More by").

- the arrows page by what's actually visible (the viewport minus one tile, so
  the last tile you could see becomes the first one you see), rather than a
  fixed 380px that fought mandatory snap and landed between tiles.
- the arrows disable at either end, so the edge is announced rather than a
  click that does nothing.
- the track fades at an edge only while there is more content past it: a
  scroll-edge effect in place of a hard clip, and a hint that it scrolls. */

function prefersReducedMotion() {
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
}

export interface CarouselState {
  ref: React.RefObject<HTMLDivElement | null>;
  canPrev: boolean;
  canNext: boolean;
  page: (dir: -1 | 1) => void;
}

export function useCarousel(deps: unknown[] = []): CarouselState {
  const ref = useRef<HTMLDivElement>(null);
  const [edges, setEdges] = useState({ canPrev: false, canNext: false });

  const update = useCallback(() => {
    const el = ref.current;
    if (!el) return;
    // is the first / last tile actually cut off? scrollLeft can't answer that:
    // the track's side padding makes the first snap point ~2px in, and at
    // fractional display scaling that read as "scrolled", fading the first
    // tile of a row that was sitting at its start. 1px of slack for rounding.
    const first = el.firstElementChild;
    const last = el.lastElementChild;
    if (!first || !last) return;
    const box = el.getBoundingClientRect();
    const canPrev = first.getBoundingClientRect().left < box.left - 1;
    const canNext = last.getBoundingClientRect().right > box.right + 1;
    setEdges((e) => (e.canPrev === canPrev && e.canNext === canNext ? e : { canPrev, canNext }));
  }, []);

  useLayoutEffect(() => {
    update();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.addEventListener("scroll", update, { passive: true });
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => {
      el.removeEventListener("scroll", update);
      ro.disconnect();
    };
  }, [update]);

  const page = useCallback((dir: -1 | 1) => {
    const el = ref.current;
    if (!el) return;
    const first = el.firstElementChild as HTMLElement | null;
    const gap = parseFloat(getComputedStyle(el).columnGap) || 0;
    const tile = first ? first.offsetWidth + gap : 0;
    const step = Math.max(tile, el.clientWidth - tile);
    el.scrollBy({ left: dir * step, behavior: prefersReducedMotion() ? "auto" : "smooth" });
  }, []);

  return { ref, canPrev: edges.canPrev, canNext: edges.canNext, page };
}

export function CarouselControls({ carousel, label }: { carousel: CarouselState; label: string }) {
  const { canPrev, canNext, page } = carousel;
  if (!canPrev && !canNext) return null;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <button
        type="button"
        className="btn-icon"
        onClick={() => canPrev && page(-1)}
        aria-disabled={!canPrev || undefined}
        aria-label={`Scroll ${label} back`}
        style={ARROW}
      >
        <ChevronLeft size={15} strokeWidth={2.4} />
      </button>
      <button
        type="button"
        className="btn-icon"
        onClick={() => canNext && page(1)}
        aria-disabled={!canNext || undefined}
        aria-label={`Scroll ${label} forward`}
        style={ARROW}
      >
        <ChevronRight size={15} strokeWidth={2.4} />
      </button>
    </div>
  );
}

const ARROW: React.CSSProperties = {
  width: 28,
  height: 28,
  borderRadius: "50%",
  border: "1px solid rgba(255, 255, 255, 0.08)",
  background: "rgba(255, 255, 255, 0.05)",
};

export function CarouselTrack<T>({
  carousel,
  items,
  getKey,
  renderItem,
  itemWidth,
  itemHeight,
  gap = 14,
  label,
}: {
  carousel: CarouselState;
  items: T[];
  getKey: (item: T) => string;
  renderItem: (item: T, index: number) => React.ReactNode;
  // any css length; every tile is exactly this wide
  itemWidth: number | string;
  itemHeight?: number | string;
  gap?: number;
  label: string;
}) {
  return (
    <div
      ref={carousel.ref}
      role="list"
      aria-label={label}
      className="edge-fade-x"
      data-fade-start={carousel.canPrev || undefined}
      data-fade-end={carousel.canNext || undefined}
      style={{
        display: "flex",
        gap,
        overflowX: "auto",
        overflowY: "hidden",
        scrollSnapType: "x mandatory",
        scrollbarWidth: "none",
        msOverflowStyle: "none",
        // room for hover shadows, taken back with the negative margin so the
        // row still lines up with the section title
        padding: "4px 2px 14px",
        margin: "-4px -2px -14px",
      }}
    >
      {items.map((item, i) => (
        <div
          key={getKey(item)}
          role="listitem"
          style={{
            flex: `0 0 ${typeof itemWidth === "number" ? `${itemWidth}px` : itemWidth}`,
            width: itemWidth,
            maxWidth: itemWidth,
            minWidth: 0,
            height: itemHeight,
            scrollSnapAlign: "start",
          }}
        >
          {renderItem(item, i)}
        </div>
      ))}
    </div>
  );
}
