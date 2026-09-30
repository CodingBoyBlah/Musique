import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Check } from "@/lib/icons";
import { usePrefsStore } from "../../store/prefs.store";
import { EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";

export const SPEEDS = [0.75, 1, 1.25, 1.5, 1.75, 2, 2.5, 3];

export function speedLabel(v: number) {
  return `${Number.isInteger(v) ? v.toFixed(0) : String(v)}×`;
}

/* podcast playback speed: a pill showing the current rate that opens a short
list above it. the rate is remembered for every episode (music always plays at
1x - see App's speed effect). */
export function SpeedControl({ compact, up = true }: { compact?: boolean; up?: boolean }) {
  const speed = usePrefsStore((s) => s.podcastSpeed);
  const setSpeed = usePrefsStore((s) => s.setPodcastSpeed);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    window.addEventListener("pointerdown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <motion.button
        type="button"
        aria-label={`Playback speed ${speedLabel(speed)}`}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        className="focus-ring tnum"
        style={{
          height: compact ? 26 : 28,
          minWidth: compact ? 40 : 46,
          padding: "0 9px",
          borderRadius: 99,
          border: "1px solid",
          borderColor: speed !== 1 ? "color-mix(in srgb, var(--color-accent) 55%, transparent)" : "rgba(255,255,255,0.14)",
          background: speed !== 1 ? "color-mix(in srgb, var(--color-accent) 20%, transparent)" : "rgba(255,255,255,0.06)",
          color: "#fff",
          fontSize: 12.5,
          fontWeight: 700,
          cursor: "pointer",
        }}
      >
        {speedLabel(speed)}
      </motion.button>
      <AnimatePresence>
        {open && (
          <motion.div
            role="menu"
            initial={{ opacity: 0, y: up ? 6 : -6, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: up ? 4 : -4, scale: 0.98, transition: { duration: 0.1 } }}
            transition={{ duration: 0.16, ease: EASE_OUT }}
            className="glass-solid-fallback"
            style={{
              position: "absolute",
              [up ? "bottom" : "top"]: "calc(100% + 8px)",
              left: "50%",
              translateX: "-50%",
              transformOrigin: up ? "bottom center" : "top center",
              zIndex: 60,
              padding: 5,
              borderRadius: 12,
              background: "var(--color-popover)",
              border: "1px solid rgba(255,255,255,0.12)",
              boxShadow: "0 16px 40px rgba(0,0,0,0.5)",
              minWidth: 112,
            }}
          >
            <div style={{ padding: "5px 9px 6px", fontSize: 10.5, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-dim)" }}>Speed</div>
            {SPEEDS.map((v) => (
              <button
                key={v}
                type="button"
                role="menuitemradio"
                aria-checked={v === speed}
                className="row-btn tnum"
                onClick={() => {
                  setSpeed(v);
                  setOpen(false);
                }}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: 10,
                  padding: "6px 9px",
                  borderRadius: 8,
                  border: "none",
                  background: v === speed ? "rgba(255,255,255,0.08)" : "transparent",
                  color: v === speed ? "var(--color-text-hi)" : "var(--color-text)",
                  fontSize: 13,
                  fontWeight: v === speed ? 700 : 500,
                  cursor: "pointer",
                  textAlign: "left",
                }}
              >
                <span>{v === 1 ? "Normal" : speedLabel(v)}</span>
                {v === speed && <Check size={13} strokeWidth={2.6} style={{ color: "var(--color-accent)" }} />}
              </button>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

// the podcast skip buttons: the circular arrow with the seconds inside it
export function SkipSeconds({ seconds, back }: { seconds: number; back?: boolean }) {
  return (
    <svg width={18} height={18} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      {back ? (
        <path d="M4.5 12a7.5 7.5 0 1 0 2.2-5.3M4.5 4v3.2h3.2" />
      ) : (
        <path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3M19.5 4v3.2h-3.2" />
      )}
      <text x="12" y="15.2" textAnchor="middle" fontSize="7.4" fontWeight="700" fill="currentColor" stroke="none" fontFamily="inherit">
        {seconds}
      </text>
    </svg>
  );
}
