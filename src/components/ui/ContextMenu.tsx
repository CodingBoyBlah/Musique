import { useEffect, useRef, useState, useCallback } from "react";
import { createPortal } from "react-dom";

export interface MenuEntry {
  label:    string;
  icon?:    React.ReactNode;
  danger?:  boolean;
  onSelect: () => void;
}

interface MenuState { x: number; y: number; entries: MenuEntry[]; }

/* lightweight right-click menu. returns an `onContextMenu` handler factory plus
  the menu element to render. one instance per component tree that needs it */
export function useContextMenu() {
  const [menu, setMenu] = useState<MenuState | null>(null);

  const open = useCallback(
    (entries: MenuEntry[]) => (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setMenu({ x: e.clientX, y: e.clientY, entries });
    },
    [],
  );

  const close = useCallback(() => setMenu(null), []);

  const element = menu ? (
    <ContextMenuView x={menu.x} y={menu.y} entries={menu.entries} onClose={close} />
  ) : null;

  return { open, element };
}

function ContextMenuView({
  x, y, entries, onClose,
}: {
  x: number; y: number; entries: MenuEntry[]; onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x, y });

  // keep the menu inside the viewport
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const nx = x + r.width  > window.innerWidth  ? window.innerWidth  - r.width  - 8 : x;
    const ny = y + r.height > window.innerHeight ? window.innerHeight - r.height - 8 : y;
    setPos({ x: nx, y: ny });
  }, [x, y]);

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={ref}
      style={{
        position:      "fixed",
        top:           pos.y,
        left:          pos.x,
        minWidth:      180,
        padding:       5,
        borderRadius:  10,
        background:    "rgba(28, 28, 32, 0.96)",
        backdropFilter: "blur(40px)",
        WebkitBackdropFilter: "blur(40px)",
        border:        "1px solid var(--color-border)",
        boxShadow:     "0 12px 32px rgba(0,0,0,0.5)",
        zIndex:        1000,
      }}
    >
      {entries.map((entry, i) => (
        <MenuRow key={i} entry={entry} onClose={onClose} />
      ))}
    </div>,
    document.body,
  );
}

function MenuRow({ entry, onClose }: { entry: MenuEntry; onClose: () => void }) {
  const [hover, setHover] = useState(false);
  return (
    <button
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onClick={() => { entry.onSelect(); onClose(); }}
      style={{
        display:    "flex",
        alignItems: "center",
        gap:        10,
        width:      "100%",
        height:     32,
        padding:    "0 10px",
        borderRadius: 6,
        border:     "none",
        background: hover ? "var(--color-hover)" : "transparent",
        color:      entry.danger ? "var(--color-danger)" : "var(--color-text-hi)",
        fontSize:   13,
        fontWeight: 500,
        cursor:     "pointer",
        textAlign:  "left",
        transition: "background 0.1s",
      }}
    >
      {entry.icon && <span style={{ display: "flex", width: 16 }}>{entry.icon}</span>}
      <span style={{ flex: 1 }}>{entry.label}</span>
    </button>
  );
}
