import { useLayoutEffect, useEffect, useRef, useState, useCallback } from "react";
import { createPortal } from "react-dom";
import { motion } from "framer-motion";
import { EASE_OUT } from "@/lib/motion";
import { useMenuKeyboard } from "@/hooks/useMenuKeyboard";

export interface MenuEntry {
  label:    string;
  icon?:    React.ReactNode;
  danger?:  boolean;
  onSelect: () => void;
}

/* Two ways a menu opens, and they deserve different motion:
   - right-click: at the cursor, instantly. It's summoned constantly and the
     cursor already marks where it will be; an animation only adds lag.
   - a "⋯" button: anchored under the button, right edges aligned, growing out
     of it (transform-origin at the trigger) so the menu visibly belongs to the
     control that opened it. */
type Anchor =
  | { kind: "point"; x: number; y: number }
  | { kind: "trigger"; rect: DOMRect };

interface MenuState { anchor: Anchor; entries: MenuEntry[]; }

/* lightweight menu. returns an `onContextMenu`/`onClick` handler factory plus
  the menu element to render. one instance per component tree that needs it.
  The handler works for both: a contextmenu event opens at the pointer, a click
  (or keyboard activation) on a button anchors to that button. */
export function useContextMenu() {
  const [menu, setMenu] = useState<MenuState | null>(null);

  const open = useCallback(
    (entries: MenuEntry[]) => (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const target = e.currentTarget as HTMLElement | null;
      if (e.type !== "contextmenu" && target?.getBoundingClientRect) {
        setMenu({ anchor: { kind: "trigger", rect: target.getBoundingClientRect() }, entries });
      } else {
        setMenu({ anchor: { kind: "point", x: e.clientX, y: e.clientY }, entries });
      }
    },
    [],
  );

  const close = useCallback(() => setMenu(null), []);

  const element = menu ? (
    <ContextMenuView anchor={menu.anchor} entries={menu.entries} onClose={close} />
  ) : null;

  return { open, element };
}

function ContextMenuView({
  anchor, entries, onClose,
}: {
  anchor: Anchor; entries: MenuEntry[]; onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ x: number; y: number; origin: string } | null>(null);

  // place it before the first paint, so it never shows up in the wrong spot
  // and then jumps. keeps the menu inside the viewport.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const W = window.innerWidth;
    const H = window.innerHeight;
    if (anchor.kind === "point") {
      const nx = anchor.x + r.width  > W ? W - r.width  - 8 : anchor.x;
      const ny = anchor.y + r.height > H ? H - r.height - 8 : anchor.y;
      setPos({ x: nx, y: ny, origin: "top left" });
      return;
    }
    const a = anchor.rect;
    const below = a.bottom + 6 + r.height <= H - 8;
    const x = Math.max(8, Math.min(a.right - r.width, W - r.width - 8));
    const y = below ? a.bottom + 6 : Math.max(8, a.top - 6 - r.height);
    setPos({ x, y, origin: below ? "top right" : "bottom right" });
  }, [anchor]);

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    window.addEventListener("mousedown", onDown);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  useMenuKeyboard(ref, true, onClose);

  const animated = anchor.kind === "trigger";

  return createPortal(
    <motion.div
      ref={ref}
      role="menu"
      className="glass-solid-fallback"
      initial={animated ? { opacity: 0, scale: 0.96 } : false}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.15, ease: EASE_OUT }}
      style={{
        position:      "fixed",
        top:           pos?.y ?? 0,
        left:          pos?.x ?? 0,
        visibility:    pos ? "visible" : "hidden",
        transformOrigin: pos?.origin ?? "top left",
        minWidth:      180,
        padding:       5,
        borderRadius:  10,
        background:    "rgba(28, 28, 32, 0.96)",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
        border:        "1px solid var(--color-border)",
        boxShadow:     "0 12px 32px rgba(0,0,0,0.5)",
        zIndex:        1000,
      }}
    >
      {entries.map((entry, i) => (
        <MenuRow key={i} entry={entry} onClose={onClose} />
      ))}
    </motion.div>,
    document.body,
  );
}

function MenuRow({ entry, onClose }: { entry: MenuEntry; onClose: () => void }) {
  return (
    <button
      role="menuitem"
      className="row-btn"
      onClick={() => { entry.onSelect(); onClose(); }}
      style={{
        gap:        10,
        height:     32,
        padding:    "0 10px",
        borderRadius: 6,
        color:      entry.danger ? "var(--color-danger)" : "var(--color-text-hi)",
        fontSize:   13,
        fontWeight: 500,
      }}
    >
      {entry.icon && <span style={{ display: "flex", width: 16 }}>{entry.icon}</span>}
      <span style={{ flex: 1 }}>{entry.label}</span>
    </button>
  );
}
