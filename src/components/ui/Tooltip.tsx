import { useState, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface Props {
  label:    ReactNode;
  children: ReactNode;
  side?:    "top" | "bottom";
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
  const off = side === "top" ? 5 : -5;

  const horiz =
    align === "center" ? { left: "50%" as const }
    : align === "end"  ? { right: 0 }
    : { left: 0 };
  const tx = align === "center" ? "-50%" : "0%";

  return (
    <span
      style={{ position: "relative", display: "inline-flex", verticalAlign: "middle" }}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      {children}
      <AnimatePresence>
        {open && (
          <motion.span
            role="tooltip"
            initial={{ opacity: 0, y: off, scale: 0.94, x: tx }}
            animate={{ opacity: 1, y: 0, scale: 1, x: tx }}
            exit={{ opacity: 0, y: off, scale: 0.94, x: tx }}
            transition={{ duration: 0.15, ease: [0.23, 1, 0.32, 1] }}
            style={{
              position:      "absolute",
              ...horiz,
              bottom:        side === "top" ? "calc(100% + 9px)" : "auto",
              top:           side === "bottom" ? "calc(100% + 9px)" : "auto",
              whiteSpace:    "nowrap",
              pointerEvents: "none",
              zIndex:        300,
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
      </AnimatePresence>
    </span>
  );
}
