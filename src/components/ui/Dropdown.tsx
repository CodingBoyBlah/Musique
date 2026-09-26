import { useCallback, useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown, Check } from "@/lib/icons";
import { EASE_OUT, PRESS, PRESS_TRANSITION, gpuLayer, zTransform } from "../../lib/motion";
import { useMenuKeyboard } from "@/hooks/useMenuKeyboard";

export interface DropdownOption<T extends string> {
  value: T;
  label: string;
}

interface Props<T extends string> {
  value:    T;
  options:  DropdownOption<T>[];
  onChange: (value: T) => void;
  align?:   "left" | "right";
  minWidth?: number;
  title?:   string;
}

/* small dark dropdown matching the window-effect menu   custom (not native
 <select>) so the popup is themed and animates origin-aware off the trigger. */
export function Dropdown<T extends string>({
  value, options, onChange, align = "left", minWidth = 140, title,
}: Props<T>) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const current = options.find((o) => o.value === value);
  const close = useCallback(() => setOpen(false), []);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  useMenuKeyboard(menuRef, open, close);

  return (
    <div ref={ref} style={{ position: "relative", flexShrink: 0 }}>
      <motion.button
        onClick={() => setOpen((v) => !v)}
        onKeyDown={(e) => {
          if (!open && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
            e.preventDefault();
            setOpen(true);
          }
        }}
        title={title}
        aria-haspopup="menu"
        aria-expanded={open}
        className="focus-ring"
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        transformTemplate={zTransform}
        style={{
          ...gpuLayer,
          display: "flex", alignItems: "center", gap: 7, height: 32,
          padding: "0 10px", borderRadius: 8, cursor: "pointer",
          border: "1px solid var(--color-glass-border)",
          background: open ? "var(--color-surface-2)" : "var(--color-glass)",
          color: "var(--color-text-hi)", fontSize: 12.5, fontWeight: 500,
          fontFamily: "inherit", whiteSpace: "nowrap",
          transition: "background 0.12s",
        }}
      >
        <span>{current?.label ?? value}</span>
        <motion.span animate={{ rotate: open ? 180 : 0 }} transition={{ duration: 0.18, ease: EASE_OUT }} style={{ display: "flex" }}>
          <ChevronDown size={13} strokeWidth={2.5} style={{ color: "var(--color-text-dim)" }} />
        </motion.span>
      </motion.button>

      <AnimatePresence>
        {open && (
          <motion.div
            ref={menuRef}
            role="menu"
            className="glass-solid-fallback"
            initial={{ opacity: 0, scale: 0.95, y: -4 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            // out faster than in
            exit={{ opacity: 0, scale: 0.96, y: -4, transition: { duration: 0.1, ease: EASE_OUT } }}
            transition={{ duration: 0.14, ease: EASE_OUT }}
            style={{
              position: "absolute", top: 38, minWidth,
              left:  align === "left"  ? 0 : undefined,
              right: align === "right" ? 0 : undefined,
              padding: 5, borderRadius: 11, zIndex: 60,
              transformOrigin: align === "right" ? "top right" : "top left",
              background: "rgba(28, 28, 32, 0.94)",
              backdropFilter: "blur(20px)", WebkitBackdropFilter: "blur(20px)",
              border: "1px solid var(--color-border)",
              boxShadow: "0 12px 32px rgba(0,0,0,0.45)",
            }}
          >
            {options.map((o) => {
              const active = o.value === value;
              return (
                <button
                  key={o.value}
                  role="menuitemradio"
                  aria-checked={active}
                  className="row-btn"
                  onClick={() => { onChange(o.value); setOpen(false); }}
                  style={{
                    gap: 8,
                    height: 32, padding: "0 9px", borderRadius: 7,
                    color: active ? "var(--color-text-hi)" : "var(--color-text)",
                    fontSize: 12.5, fontWeight: active ? 600 : 500,
                    whiteSpace: "nowrap",
                  }}
                >
                  <span style={{ flex: 1 }}>{o.label}</span>
                  {active && <Check size={14} strokeWidth={2.5} style={{ color: "var(--color-accent)" }} />}
                </button>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
