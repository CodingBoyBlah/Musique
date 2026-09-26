import { useState, useRef, useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";
import { EASE_OUT } from "@/lib/motion";

/* Warm state, shared by every tooltip. The first tooltip waits a beat so a
   pointer passing over the toolbar doesn't spray labels; once one has been
   shown, moving to its neighbour opens the next one at once and without an
   animation - running along a row of controls should feel instant. */
const WARM_MS = 400;
let openCount = 0;
let lastClosedAt = 0;
const isWarm = () => openCount > 0 || performance.now() - lastClosedAt < WARM_MS;

interface Props {
  label:    ReactNode;
  children: ReactNode;
  side?:    "top" | "bottom" | "right";
  /* horizontal anchoring. "center" (default) centres over the control "end"
  pins the tooltip right edge to the control so it never spills off the
  right of the window (the corner queue button uses this) */
  align?:   "center" | "start" | "end";
}

/* small themed tooltip. wraps a control, shows a frosted label on hover.

stays visible while the pointers over the control, so clicking toggle
(shuffle / repeat) updates the label INPLACE without re-hovering
verticalAlign:middle + lineHeight:0 keep the wrapped button on the text
baseline so its scale animation doesnt make it jump. */
export function Tooltip({ label, children, side = "top", align = "center" }: Props) {
  const [open, setOpenState] = useState(false);
  // opened from the warm state: skip the animation too, not just the delay
  const [instant, setInstant] = useState(false);
  const openRef = useRef(false);
  const setOpen = (v: boolean) => {
    if (v === openRef.current) return;
    openRef.current = v;
    if (v) openCount++;
    else {
      openCount = Math.max(0, openCount - 1);
      lastClosedAt = performance.now();
    }
    setOpenState(v);
  };
  const [effectiveAlign, setEffectiveAlign] = useState(align);
  /* Trigger position in viewport coordinates, captured on hover. The tooltip
     is rendered in a portal (see below), so it can't be placed relative to the
     trigger by the normal box tree - it has to be positioned from this. */
  const [rect, setRect] = useState<DOMRect | null>(null);
  const triggerRef = useRef<HTMLSpanElement>(null);
  const timerRef = useRef<number | undefined>(undefined);
  const off = side === "top" ? 5 : -5;

  useEffect(() => {
    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current);
      if (openRef.current) {
        openRef.current = false;
        openCount = Math.max(0, openCount - 1);
        lastClosedAt = performance.now();
      }
    };
  }, []);

  /* The captured rect goes stale if the page moves under the pointer, and a
     fixed-position tooltip would visibly detach from its trigger. Hiding is
     the right response: the pointer has effectively left the control. */
  useEffect(() => {
    if (!open) return;
    const dismiss = () => setOpen(false);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("scroll", dismiss, true);
    window.addEventListener("resize", dismiss);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("scroll", dismiss, true);
      window.removeEventListener("resize", dismiss);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const handleMouseEnter = () => {
    if (triggerRef.current) {
      setRect(triggerRef.current.getBoundingClientRect());
    }
    if (triggerRef.current && align === "center") {
      const rect = triggerRef.current.getBoundingClientRect();
      if (rect.right > window.innerWidth - 85) {
        setEffectiveAlign("end");
      } else if (rect.left < 85) {
        setEffectiveAlign("start");
      } else {
        setEffectiveAlign("center");
      }
    } else {
      setEffectiveAlign(align);
    }
    if (timerRef.current) window.clearTimeout(timerRef.current);
    if (isWarm()) {
      setInstant(true);
      setOpen(true);
      return;
    }
    setInstant(false);
    timerRef.current = window.setTimeout(() => {
      setOpen(true);
    }, 140);
  };

  /* Keyboard focus shows the label too - a keyboard user tabbing onto an
     icon-only button otherwise has no way to learn what it does. Only for
     :focus-visible, so a mouse click doesn't pin the tooltip open. */
  const handleFocus = (e: React.FocusEvent) => {
    const t = e.target as HTMLElement;
    let visible = false;
    try {
      visible = t.matches(":focus-visible");
    } catch {
      visible = false;
    }
    if (visible) handleMouseEnter();
  };

  const handleMouseLeave = () => {
    if (timerRef.current) {
      window.clearTimeout(timerRef.current);
      timerRef.current = undefined;
    }
    setOpen(false);
  };

  const isRight = side === "right";

  /* Viewport-space anchoring, equivalent to the old percentage offsets but
     resolved against the trigger's rect instead of its containing block.
     Each case anchors the edge nearest the trigger (`bottom` for a tooltip
     above it, `right` for an end-aligned one) so the remaining offset can stay
     a pure transform - that leaves the x/y animation props below untouched and
     keeps the motion identical to before. */
  const horiz: React.CSSProperties = !rect
    ? {}
    : isRight
      ? { left: rect.right + 10, top: rect.top + rect.height / 2 }
      : {
          ...(effectiveAlign === "center"
            ? { left: rect.left + rect.width / 2 }
            : effectiveAlign === "end"
              ? { right: window.innerWidth - rect.right }
              : { left: rect.left }),
          ...(side === "top"
            ? { bottom: window.innerHeight - rect.top + 9 }
            : { top: rect.bottom + 9 }),
        };
  const tx = effectiveAlign === "center" ? "-50%" : "0%";
  /* grow out of the trigger, not from the label's own centre */
  const originX = effectiveAlign === "center" ? "center" : effectiveAlign === "end" ? "right" : "left";
  const transformOrigin = isRight
    ? "left center"
    : `${originX} ${side === "top" ? "bottom" : "top"}`;

  return (
    <span
      ref={triggerRef}
      style={{
        position: "relative",
        display: "inline-flex",
        verticalAlign: "middle",
      }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onFocus={handleFocus}
      onBlur={handleMouseLeave}
    >
      {children}
      {/* Rendered into <body> rather than next to the trigger.
          An ancestor with any non-visible overflow clips absolutely-positioned
          descendants - and note that setting overflow-x alone is enough, since
          CSS computes the other axis to `auto` rather than leaving it visible.
          The segmented control scrolls horizontally on narrow widths, which was
          quietly cutting off the tooltip on its first option. A portal escapes
          every such ancestor instead of special-casing each one. */}
      {createPortal(
        <AnimatePresence>
          {open && rect && (
          <motion.span
            role="tooltip"
            initial={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
            animate={isRight ? { opacity: 1, x: 0, y: "-50%", scale: 1 } : { opacity: 1, y: 0, scale: 1, x: tx }}
            exit={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
            transition={instant ? { duration: 0 } : { duration: 0.15, ease: EASE_OUT }}
            className="glass-solid-fallback t-caption"
            style={{
              transformOrigin,
              position:      "fixed",
              ...horiz,
              whiteSpace:    "nowrap",
              pointerEvents: "none",
              zIndex:        10000,
              padding:       "5px 9px",
              borderRadius:  7,
              lineHeight:    1.2,
              background:    "var(--color-popover)",
              backdropFilter:       "blur(20px)",
              WebkitBackdropFilter: "blur(20px)",
              border:        "1px solid var(--color-glass-border)",
              boxShadow:     "0 8px 22px rgba(0,0,0,0.45)",
              fontSize:      11.5,
              fontWeight:    600,
              color:         "var(--color-text-hi)",
            }}
          >
            {label}
          </motion.span>
          )}
        </AnimatePresence>,
        document.body,
      )}
    </span>
  );
}
