import { useState, useRef, useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";

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
  const [open, setOpen] = useState(false);
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
    };
  }, []);

  /* The captured rect goes stale if the page moves under the pointer, and a
     fixed-position tooltip would visibly detach from its trigger. Hiding is
     the right response: the pointer has effectively left the control. */
  useEffect(() => {
    if (!open) return;
    const dismiss = () => setOpen(false);
    window.addEventListener("scroll", dismiss, true);
    window.addEventListener("resize", dismiss);
    return () => {
      window.removeEventListener("scroll", dismiss, true);
      window.removeEventListener("resize", dismiss);
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
    timerRef.current = window.setTimeout(() => {
      setOpen(true);
    }, 140);
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
            transition={{ duration: 0.15, ease: [0.23, 1, 0.32, 1] }}
            style={{
              position:      "fixed",
              ...horiz,
              whiteSpace:    "nowrap",
              pointerEvents: "none",
              zIndex:        10000,
              padding:       "5px 9px",
              borderRadius:  7,
              lineHeight:    1.2,
              background:    "rgba(26, 26, 30, 0.96)",
              backdropFilter:       "blur(20px)",
              WebkitBackdropFilter: "blur(20px)",
              border:        "1px solid var(--color-glass-border)",
              boxShadow:     "0 8px 22px rgba(0,0,0,0.45)",
              fontSize:      11.5,
              fontWeight:    600,
              letterSpacing: "0.01em",
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
