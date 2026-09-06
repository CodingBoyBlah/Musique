import { useState, useRef, type ReactNode } from "react";
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
  const triggerRef = useRef<HTMLSpanElement>(null);
  const off = side === "top" ? 5 : -5;

  const handleMouseEnter = () => {
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
    setOpen(true);
  };

  const isRight = side === "right";
  const horiz = isRight
    ? { left: "calc(100% + 10px)", top: "50%", bottom: "auto" as const }
    : {
        ...(effectiveAlign === "center" ? { left: "50%" as const } : effectiveAlign === "end" ? { right: 0 } : { left: 0 }),
        bottom: side === "top" ? "calc(100% + 9px)" : "auto",
        top: side === "bottom" ? "calc(100% + 9px)" : "auto",
      };
  const tx = effectiveAlign === "center" ? "-50%" : "0%";

  return (
    <span
      ref={triggerRef}
      style={{
        position: "relative",
        display: "inline-flex",
        verticalAlign: "middle",
        zIndex: open ? 60 : undefined,
      }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={() => setOpen(false)}
    >
      {children}
      <AnimatePresence>
        {open && (
          <motion.span
            role="tooltip"
            initial={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
            animate={isRight ? { opacity: 1, x: 0, y: "-50%", scale: 1 } : { opacity: 1, y: 0, scale: 1, x: tx }}
            exit={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
            transition={{ duration: 0.15, ease: [0.23, 1, 0.32, 1] }}
            style={{
              position:      "absolute",
              ...horiz,
              whiteSpace:    "nowrap",
              pointerEvents: "none",
              zIndex:        1000,
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
