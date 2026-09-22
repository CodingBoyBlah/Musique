import { useState, useRef, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AnimatedPlayPause } from "./AnimatedIcons";

// 1. Primary Glow Button (Fully rounded pill, matches Album Page play button, animated play/pause icon)
export function PrimaryPlayButton({
  playing = false,
  onToggle,
  label = "Play",
  size = "md",
}: {
  playing?: boolean;
  onToggle?: () => void;
  label?: string;
  size?: "sm" | "md" | "lg";
}) {
  const sizeStyles = {
    sm: { height: 36, padding: "0 18px", fontSize: 13, iconSize: 14, gap: 6 },
    md: { height: 44, padding: "0 24px", fontSize: 14, iconSize: 16, gap: 8 },
    lg: { height: 50, padding: "0 30px", fontSize: 15, iconSize: 18, gap: 9 },
  }[size];

  return (
    <motion.button
      whileHover={{ scale: 1.04 }}
      whileTap={{ scale: 0.96 }}
      onClick={onToggle}
      title={playing ? "Pause" : "Play"}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        height: sizeStyles.height,
        padding: sizeStyles.padding,
        gap: sizeStyles.gap,
        borderRadius: 99, // fully rounded pill
        border: "none",
        background: "var(--color-accent)",
        color: "var(--color-accent-text)",
        fontSize: sizeStyles.fontSize,
        fontWeight: 700,
        letterSpacing: "-0.01em",
        cursor: "pointer",
        boxShadow: playing
          ? "0 0 0 4px var(--color-accent-dim), 0 4px 18px var(--color-accent-dim)"
          : "0 4px 18px -2px var(--color-accent-dim)",
        transition: "box-shadow 0.2s, background 0.15s",
        outline: "none",
        userSelect: "none",
      }}
    >
      <AnimatedPlayPause isPlaying={playing} size={sizeStyles.iconSize} strokeWidth={0} />
      <span>{playing ? "Pause" : label}</span>
    </motion.button>
  );
}

// 2. Circular Pulse Play Button (KEPT - user loved this, added blur+scale icon animation)
export function PulsePlayButton({
  playing,
  onToggle,
  size = 56,
}: {
  playing: boolean;
  onToggle: () => void;
  size?: number;
}) {
  return (
    <div style={{ position: "relative", width: size, height: size, display: "inline-flex", alignItems: "center", justifyContent: "center" }}>
      <AnimatePresence>
        {playing && (
          <motion.div
            initial={{ scale: 0.85, opacity: 0.7 }}
            animate={{ scale: 1.5, opacity: 0 }}
            exit={{ opacity: 0 }}
            transition={{ repeat: Infinity, duration: 1.8, ease: "easeOut" }}
            style={{
              position: "absolute",
              inset: 0,
              borderRadius: 999,
              border: "2px solid var(--color-accent)",
              pointerEvents: "none",
            }}
          />
        )}
      </AnimatePresence>

      <motion.button
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.94 }}
        onClick={onToggle}
        title={playing ? "Pause" : "Play"}
        style={{
          position: "relative",
          zIndex: 1,
          width: size,
          height: size,
          borderRadius: 999,
          border: "none",
          background: "var(--color-accent)",
          color: "var(--color-accent-text)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          boxShadow: "0 8px 24px -4px var(--color-accent-dim)",
          outline: "none",
        }}
      >
        <AnimatedPlayPause isPlaying={playing} size={Math.round(size * 0.42)} strokeWidth={0} />
      </motion.button>
    </div>
  );
}

// 3. Draggable Segmented Mode Switcher (Magnetically attracts while dragging, immune to window resize)
export function SegmentedControl({
  options = ["Stereo", "Hi-Res Studio", "Spatial Atmos", "Vinyl Warmth"],
  value,
  onChange,
  layoutId = "segmented-pill",
  disabled,
  adornments,
}: {
  options?: string[];
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
              fontSize: "clamp(11.5px, 1.2vw, 12.5px)",
              fontWeight: 600,
              color: active ? "var(--color-accent-text)" : "var(--color-text-dim)",
              opacity: off ? 0.45 : 1,
              cursor: off ? "default" : "pointer",
              zIndex: 1,
              transition: "color 0.15s ease, opacity 0.15s ease",
              outline: "none",
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
