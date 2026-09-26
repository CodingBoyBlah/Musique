import { useEffect, useRef, useState, type RefObject } from "react";

/* A scrollbar that takes no room. The native bar is hidden on the scroller
(give it the `ovs-hide` class) and this thin pill floats over its right edge
instead, inside a positioned `ovs-host` wrapper. It shows while you scroll or
hover the host and fades once you stop, like a macOS overlay scrollbar.

The thumb tracks the pointer 1:1 from wherever you grabbed it (pointer
capture, so it keeps following outside the strip), and a click on the empty
track pages toward the click. */

const MIN_THUMB = 28;
const IDLE_MS = 900;

export function OverlayScrollbar({ target }: { target: RefObject<HTMLElement | null> }) {
  const trackRef = useRef<HTMLDivElement>(null);
  const [m, setM] = useState({ size: 0, offset: 0, on: false });
  const [scrolling, setScrolling] = useState(false);
  const [dragging, setDragging] = useState(false);
  const drag = useRef<{ y: number; top: number } | null>(null);

  useEffect(() => {
    const el = target.current;
    if (!el) return;
    let raf = 0;
    let idle: ReturnType<typeof setTimeout> | undefined;

    const update = () => {
      raf = 0;
      const track = trackRef.current?.clientHeight ?? el.clientHeight;
      const { scrollHeight: sh, clientHeight: ch, scrollTop: st } = el;
      if (sh <= ch + 1) {
        setM((p) => (p.on ? { size: 0, offset: 0, on: false } : p));
        return;
      }
      const size = Math.max(MIN_THUMB, (ch / sh) * track);
      const offset = (st / (sh - ch)) * (track - size);
      setM((p) => (p.on && p.size === size && p.offset === offset ? p : { size, offset, on: true }));
    };
    // at most one measure per frame, however many events arrive
    const schedule = () => {
      if (!raf) raf = requestAnimationFrame(update);
    };
    const onScroll = () => {
      schedule();
      setScrolling(true);
      clearTimeout(idle);
      idle = setTimeout(() => setScrolling(false), IDLE_MS);
    };

    update();
    el.addEventListener("scroll", onScroll, { passive: true });
    // the box resizing (window, zoom) and the content changing height (rows
    // added, a section folding) both move the thumb
    const ro = new ResizeObserver(schedule);
    ro.observe(el);
    const mo = new MutationObserver(schedule);
    mo.observe(el, { childList: true, subtree: true, attributes: true, attributeFilter: ["style"] });
    return () => {
      el.removeEventListener("scroll", onScroll);
      ro.disconnect();
      mo.disconnect();
      cancelAnimationFrame(raf);
      clearTimeout(idle);
    };
  }, [target]);

  function onThumbDown(e: React.PointerEvent<HTMLDivElement>) {
    const el = target.current;
    if (!el || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { y: e.clientY, top: el.scrollTop };
    setDragging(true);
  }

  function onThumbMove(e: React.PointerEvent<HTMLDivElement>) {
    const el = target.current;
    const track = trackRef.current;
    if (!drag.current || !el || !track) return;
    // content pixels per thumb pixel, so the thumb stays under the pointer
    const ratio = (el.scrollHeight - el.clientHeight) / Math.max(1, track.clientHeight - m.size);
    el.scrollTop = drag.current.top + (e.clientY - drag.current.y) * ratio;
  }

  function endDrag() {
    drag.current = null;
    setDragging(false);
  }

  function onTrackDown(e: React.PointerEvent<HTMLDivElement>) {
    const el = target.current;
    const track = trackRef.current;
    if (!el || !track || e.target !== track || e.button !== 0) return;
    const y = e.clientY - track.getBoundingClientRect().top;
    const dir = y < m.offset ? -1 : 1;
    const reduce = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    el.scrollBy({ top: dir * el.clientHeight * 0.9, behavior: reduce ? "auto" : "smooth" });
  }

  return (
    <div
      ref={trackRef}
      aria-hidden
      className="ovs-track"
      data-on={m.on || undefined}
      data-visible={scrolling || dragging || undefined}
      data-dragging={dragging || undefined}
      onPointerDown={onTrackDown}
    >
      {m.on && (
        <div
          className="ovs-thumb"
          onPointerDown={onThumbDown}
          onPointerMove={onThumbMove}
          onPointerUp={endDrag}
          onPointerCancel={endDrag}
          style={{ height: m.size, transform: `translateY(${m.offset}px)` }}
        />
      )}
    </div>
  );
}
