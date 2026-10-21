import { useState, useRef, type ReactNode } from "react";
import { motion } from "framer-motion";
import "../../styles/ui.css";

// 3. Draggable Segmented Mode Switcher (Magnetically attracts while dragging, immune to window resize)
export function SegmentedControl({
  options,
  value,
  onChange,
  layoutId = "segmented-pill",
  disabled,
  adornments,
}: {
  options: string[];
  value: string;
  onChange: (val: string) => void;
  layoutId?: string;
  /** Labels that render greyed out and cannot be selected or dragged to. */
  disabled?: string[];
  /** Extra content rendered after a label, keyed by label - e.g. an info icon
   *  explaining why that option is unavailable. */
  adornments?: Record<string, ReactNode>;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [dragIndex, setDragIndex] = useState<number | null>(null);

  const isDraggingRef = useRef(false);
  const startXRef = useRef(0);

  const isDisabled = (opt: string) => disabled?.includes(opt) ?? false;

  // Active option is either the magnetically attracted drag target or the committed value
  const activeOption = dragIndex !== null ? options[dragIndex] : value;

  // Falls back to the committed value's index so a control whose options are
  // all disabled can't snap the pill somewhere unselectable.
  const getClosestIndex = (clientX: number) => {
    let closest = Math.max(0, options.indexOf(value));
    let minDistance = Infinity;

    tabRefs.current.forEach((tab, idx) => {
      // Skip unavailable options so dragging glides past them rather than
      // snapping onto something that can't be chosen.
      if (!tab || isDisabled(options[idx])) return;
      const rect = tab.getBoundingClientRect();
      const tabCenter = rect.left + rect.width / 2;
      const dist = Math.abs(clientX - tabCenter);
      if (dist < minDistance) {
        minDistance = dist;
        closest = idx;
      }
    });

    return closest;
  };

  const handlePointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    startXRef.current = e.clientX;
    isDraggingRef.current = false;
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (e.buttons !== 1) return;
    const deltaX = e.clientX - startXRef.current;

    // Only engage drag after moving past 6px threshold
    if (!isDraggingRef.current) {
      if (Math.abs(deltaX) > 6) {
        isDraggingRef.current = true;
        try {
          (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
        } catch {}
      } else {
        return;
      }
    }

    // Magnetically attract and snap to the closest option while dragging
    const closest = getClosestIndex(e.clientX);
    setDragIndex(closest);
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (isDraggingRef.current) {
      isDraggingRef.current = false;
      try {
        (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      } catch {}

      const closest = getClosestIndex(e.clientX);
      onChange(options[closest]);
    }
    setDragIndex(null);
  };

  return (
    <div
      ref={containerRef}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      role="radiogroup"
      style={{
        position: "relative",
        display: "inline-flex",
        alignItems: "center",
        padding: 4,
        borderRadius: 12,
        background: "var(--color-surface)",
        border: "1px solid var(--color-border)",
        gap: 4,
        userSelect: "none",
        touchAction: "none",
        cursor: dragIndex !== null ? "grabbing" : "grab",
        maxWidth: "100%",
        boxSizing: "border-box",
        overflowX: "auto",
        scrollbarWidth: "none",
      }}
    >
      {options.map((opt, idx) => {
        const off = isDisabled(opt);
        const active = opt === activeOption && !off;
        const adornment = adornments?.[opt];

        return (
          <button
            key={opt}
            ref={(el) => {
              tabRefs.current[idx] = el;
            }}
            // aria-disabled rather than the `disabled` attribute on purpose:
            // a disabled button swallows pointer events, which would stop any
            // adornment tooltip from opening - exactly when it's the only
            // thing explaining why the option is greyed out.
            aria-disabled={off || undefined}
            role="radio"
            aria-checked={active}
            className="seg-opt"
            onClick={() => { if (!off) onChange(opt); }}
            style={{
              position: "relative",
              display: "inline-flex",
              alignItems: "center",
              gap: 5,
              padding: "7px clamp(8px, 1.4vw, 16px)",
              borderRadius: 8,
              border: "none",
              background: "transparent",
              fontSize: "0.875rem",
              fontWeight: 600,
              color: active ? "var(--color-accent-text)" : "var(--color-text-dim)",
              opacity: off ? 0.45 : 1,
              cursor: off ? "default" : "pointer",
              zIndex: 1,
              transition: "color 0.15s ease, opacity 0.15s ease",
              userSelect: "none",
              whiteSpace: "nowrap",
            }}
          >
            {active && (
              <motion.div
                layoutId={layoutId}
                transition={{
                  type: "spring",
                  stiffness: 420,
                  damping: 38,
                  mass: 0.8,
                }}
                style={{
                  position: "absolute",
                  inset: 0,
                  borderRadius: 8,
                  background: "var(--color-accent)",
                  zIndex: -1,
                  boxShadow: "0 4px 14px -2px var(--color-accent-dim)",
                }}
              />
            )}
            <span style={{ position: "relative", zIndex: 1 }}>{opt}</span>
            {adornment && (
              <span
                style={{ position: "relative", zIndex: 1, display: "inline-flex", alignItems: "center" }}
                // The adornment is its own affordance (a tooltip trigger), so a
                // click on it shouldn't also pick the option.
                onClick={(e) => e.stopPropagation()}
                onPointerDown={(e) => e.stopPropagation()}
              >
                {adornment}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
