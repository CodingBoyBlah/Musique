import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { createScrollSpring, type ScrollSpring } from "../lib/scrollSpring";

/* Keep the sung line in view - until the reader takes the column.
 *
 * Shared by the side panel and the immersive view. Three rules:
 *
 * 1. The first placement is a jump. On open, on coming back to the Lyrics
 *    tab, and on a new track, the column starts at the top; tweening from there
 *    to the current line raced the whole song past the reader. Only line-to-line
 *    movement animates.
 *
 * 2. Line to line, one spring (lib/scrollSpring) that keeps its velocity, so
 *    fast verses glide instead of kicking on every line.
 *
 * 3. The reader always wins. A wheel, a touch, a scroll key or a grab of the
 *    scrollbar stops the spring on the spot and lets go of the column - the
 *    next line change used to haul anyone re-reading a verse straight back
 *    down. Following picks up again after a few quiet seconds, the moment the
 *    reader scrolls the current line back into view themselves, or when they
 *    ask for it (`recenter`, behind the "Back to current line" pill).
 */

const IDLE_RESUME_MS = 3000;
const SCROLL_KEYS = new Set(["ArrowUp", "ArrowDown", "PageUp", "PageDown", "Home", "End", " "]);

export function useLyricFollow({
  scrollRef,
  rowRefs,
  active,
  resetKey,
  trackKey,
  targetFor,
  reduceMotion,
}: {
  scrollRef: RefObject<HTMLDivElement | null>;
  rowRefs: RefObject<(HTMLDivElement | null)[]>;
  active: number;
  /** changes whenever the column's rows are rebuilt (loaded, upgraded) */
  resetKey: unknown;
  /** changes only when the song changes. a mid-song rebuild (word timings
      arriving) must not yank a reader who has scrolled away back to the line */
  trackKey: unknown;
  /** where the column should rest for this row */
  targetFor: (row: HTMLDivElement, cont: HTMLDivElement) => number;
  reduceMotion: boolean | null;
}) {
  const [detached, setDetached] = useState(false);
  const detachedRef = useRef(false);
  const placedRef = useRef(false);
  const springRef = useRef<{ el: HTMLElement; spring: ScrollSpring } | null>(null);
  const idleRef = useRef(0);
  const activeRef = useRef(active);
  activeRef.current = active;
  const targetForRef = useRef(targetFor);
  targetForRef.current = targetFor;

  const springFor = (cont: HTMLDivElement) => {
    if (springRef.current?.el !== cont) {
      springRef.current?.spring.stop();
      springRef.current = { el: cont, spring: createScrollSpring(cont) };
    }
    return springRef.current.spring;
  };

  const attach = useCallback(() => {
    window.clearTimeout(idleRef.current);
    detachedRef.current = false;
    setDetached(false);
  }, []);

  // new song: forget where we were, place without motion, follow again
  useEffect(() => {
    placedRef.current = false;
    attach();
  }, [trackKey, attach]);

  // rows rebuilt (the new song's lyrics landing, or an upgrade): re-place
  // without motion if we're following. if the reader has scrolled away,
  // leave them exactly where they are.
  useEffect(() => {
    if (!detachedRef.current) placedRef.current = false;
  }, [resetKey]);

  useEffect(() => {
    if (detached || active < 0) return;
    const el = rowRefs.current?.[active];
    const cont = scrollRef.current;
    if (!el || !cont) return;

    const spring = springFor(cont);
    const target = targetForRef.current(el, cont);
    if (!placedRef.current || reduceMotion) {
      spring.jump(target);
      placedRef.current = true;
      return;
    }
    spring.to(target);
    // the spring is left running on purpose: the next line re-targets it mid-flight
  }, [active, resetKey, reduceMotion, detached, rowRefs, scrollRef]);

  // the reader's hands on the column
  useEffect(() => {
    const cont = scrollRef.current;
    if (!cont) return;

    const detach = () => {
      springRef.current?.spring.stop();
      if (!detachedRef.current) {
        detachedRef.current = true;
        setDetached(true);
      }
      window.clearTimeout(idleRef.current);
      idleRef.current = window.setTimeout(attach, IDLE_RESUME_MS);
    };

    const onPointerDown = (e: PointerEvent) => {
      // middle-button autoscroll, or a press on the scrollbar itself (outside
      // the client box). a plain click on a line is a seek, not a scroll.
      const onBar = e.target === cont && (e.offsetX >= cont.clientWidth || e.offsetY >= cont.clientHeight);
      if (e.button === 1 || onBar) detach();
    };
    const onKey = (e: KeyboardEvent) => {
      if (SCROLL_KEYS.has(e.key)) detach();
    };
    // scrolled the current line back into the middle of the view by hand:
    // that is the reader handing the column back
    const onScroll = () => {
      if (!detachedRef.current) return;
      const el = rowRefs.current?.[activeRef.current];
      if (!el) return;
      const c = cont.getBoundingClientRect();
      const r = el.getBoundingClientRect();
      const mid = (r.top + r.bottom) / 2;
      if (mid > c.top + c.height * 0.25 && mid < c.bottom - c.height * 0.25) {
        window.clearTimeout(idleRef.current);
        idleRef.current = window.setTimeout(attach, 400);
      }
    };

    cont.addEventListener("wheel", detach, { passive: true });
    cont.addEventListener("touchstart", detach, { passive: true });
    cont.addEventListener("pointerdown", onPointerDown);
    cont.addEventListener("keydown", onKey);
    cont.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      cont.removeEventListener("wheel", detach);
      cont.removeEventListener("touchstart", detach);
      cont.removeEventListener("pointerdown", onPointerDown);
      cont.removeEventListener("keydown", onKey);
      cont.removeEventListener("scroll", onScroll);
    };
    // resetKey: the scroll container only exists once there are rows to show
  }, [resetKey, attach, rowRefs, scrollRef]);

  useEffect(
    () => () => {
      window.clearTimeout(idleRef.current);
      springRef.current?.spring.stop();
    },
    [],
  );

  return { detached, recenter: attach };
}
