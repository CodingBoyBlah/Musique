import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useNavigate } from "react-router-dom";
import {
  ChevronLeft,
  ChevronRight,
  Minus,
  Square,
  Copy,
  X,
  MoreHorizontal,
  Layers,
  Sparkles,
  Check,
  Ban,
} from "lucide-react";
import { setWindowEffect, type WindowEffect } from "../../api/window";
import { useUIStore } from "../../store/ui.store";
import { Tooltip } from "../ui/Tooltip";
import { isMac, isWindows } from "../../lib/platform";
import { gpuLayer, zTransform } from "../../lib/motion";

// win11 caption button (transparent and full-height)

function CaptionBtn({
  children,
  onClick,
  danger,
}: {
  children: React.ReactNode;
  onClick: () => void;
  danger?: boolean;
}) {
  const [hover, setHover] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        width: 46,
        height: "100%",
        border: "none",
        background: hover
          ? danger
            ? "#c42b1c"
            : "rgba(255,255,255,0.08)"
          : "transparent",
        color: hover && danger ? "#fff" : "rgba(255,255,255,0.78)",
        cursor: "pointer",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        transition: "background 0.12s, color 0.12s",
        flexShrink: 0,
      }}
    >
      {children}
    </button>
  );
}

// round nav / ellipsis button

function PillBtn({
  children,
  onClick,
  wide,
  nudge,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  wide?: boolean;
  // nudge the glyph a hair this way on hover (back left, forward right)
  nudge?: "left" | "right";
}) {
  const [hover, setHover] = useState(false);
  const nudgeX = nudge && hover ? (nudge === "left" ? -2 : 2) : 0;
  return (
    <motion.button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ scale: 0.97 }}
      whileTap={{ scale: 0.92 }}
      transition={{ type: "spring", stiffness: 400, damping: 22 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        height: 30,
        width: wide ? 38 : 30,
        borderRadius: wide ? 8 : "50%",
        // borderless at rest, border only eases in on hover
        border: hover
          ? "1px solid var(--color-glass-border)"
          : "1px solid transparent",
        background: hover ? "rgba(255,255,255,0.10)" : "transparent",
        color: hover ? "var(--color-text-hi)" : "var(--color-text)",
        cursor: "pointer",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        transition:
          "background 0.18s ease, color 0.18s ease, border-color 0.18s ease",
        flexShrink: 0,
      }}
    >
      {/* directional micro-slide - chevron leans the way it'll take you */}
      <motion.span
        style={{ display: "flex" }}
        animate={{ x: nudgeX }}
        transition={{ type: "spring", stiffness: 500, damping: 28 }}
      >
        {children}
      </motion.span>
    </motion.button>
  );
}

// effect menu (Mica / Acrylic switcher)  UPDATE: None choice added too

function MenuItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  const [hover, setHover] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        width: "100%",
        height: 34,
        padding: "0 10px",
        borderRadius: 7,
        border: "none",
        background: hover ? "var(--color-hover)" : "transparent",
        color: "var(--color-text-hi)",
        fontSize: 13,
        fontWeight: 500,
        cursor: "pointer",
        textAlign: "left",
        transition: "background 0.1s",
      }}
    >
      <span
        style={{
          width: 16,
          display: "flex",
          color: active ? "var(--color-accent)" : "var(--color-text)",
        }}
      >
        {icon}
      </span>
      <span style={{ flex: 1 }}>{label}</span>
      {active && (
        <Check
          size={14}
          strokeWidth={2.5}
          style={{ color: "var(--color-accent)" }}
        />
      )}
    </button>
  );
}

function EffectMenu() {
  const [open, setOpen] = useState(false);
  const effect = useUIStore((s) => s.windowEffect);
  const setEffect = useUIStore((s) => s.setWindowEffect);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node))
        setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  async function pick(mode: WindowEffect) {
    setEffect(mode);
    setOpen(false);
    try {
      await setWindowEffect(mode);
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <Tooltip label="Window material" side="bottom" align="start">
        <PillBtn wide onClick={() => setOpen((v) => !v)}>
          <MoreHorizontal size={16} strokeWidth={2.5} />
        </PillBtn>
      </Tooltip>
      {open && (
        <div
          style={{
            position: "absolute",
            top: 38,
            left: 0,
            width: 190,
            padding: 6,
            borderRadius: 12,
            background: "rgba(28, 28, 32, 0.92)",
            backdropFilter: "blur(40px)",
            WebkitBackdropFilter: "blur(40px)",
            border: "1px solid var(--color-border)",
            boxShadow: "0 12px 32px rgba(0,0,0,0.45)",
            zIndex: 100,
          }}
        >
          <p
            style={{
              margin: 0,
              padding: "5px 10px 6px",
              fontSize: 10.5,
              fontWeight: 700,
              letterSpacing: "0.06em",
              textTransform: "uppercase",
              color: "var(--color-text-dim)",
            }}
          >
            Window material
          </p>
          <MenuItem
            icon={<Layers size={15} strokeWidth={2} />}
            label="Mica mode"
            active={effect === "mica"}
            onClick={() => pick("mica")}
          />
          <MenuItem
            icon={<Sparkles size={15} strokeWidth={2} />}
            label="Acrylic mode"
            active={effect === "acrylic"}
            onClick={() => pick("acrylic")}
          />
          <MenuItem
            icon={<Ban size={15} strokeWidth={2} />}
            label="No material"
            active={effect === "none"}
            onClick={() => pick("none")}
          />
        </div>
      )}
    </div>
  );
}

export function TitleBar() {
  const navigate = useNavigate();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    win
      .isMaximized()
      .then(setMaximized)
      .catch(() => {});
    const unlisten = win.onResized(() => {
      win
        .isMaximized()
        .then(setMaximized)
        .catch(() => {});
    });
    return () => {
      unlisten.then((u) => u()).catch(() => {});
    };
  }, []);

  const win = getCurrentWindow();

  return (
    <>
      {/* macOS: full-width draggable strip above the bar, matching the sidebar
          one, so the native traffic lights get clean vertical space and the
          whole top row stays aligned across both columns */}
      {isMac && (
        <div data-tauri-drag-region style={{ height: 30, flexShrink: 0 }} />
      )}
      <div
        style={{
          height: 48,
          flexShrink: 0,
          display: "flex",
          alignItems: "center",
        }}
      >
        {/* left controls: menu + back/forward. no mac inset needed, the traffic
          lights sit over the sidebar column not here */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: "0 12px",
            flexShrink: 0,
          }}
        >
          {/* Mica/Acrylic switcher is windows-only material, hide elsewhere */}
          {isWindows && <EffectMenu />}
          <Tooltip label="Back" side="bottom">
            <PillBtn onClick={() => navigate(-1)} nudge="left">
              <ChevronLeft size={15} strokeWidth={2.5} />
            </PillBtn>
          </Tooltip>
          <Tooltip label="Forward" side="bottom">
            <PillBtn onClick={() => navigate(1)} nudge="right">
              <ChevronRight size={15} strokeWidth={2.5} />
            </PillBtn>
          </Tooltip>
        </div>

        {/* drag region */}
        <div
          data-tauri-drag-region
          style={{ flex: 1, height: "100%", cursor: "default" }}
        />

        {/* custom caption buttons - windows/linux only. macOS uses its native
          traffic lights so we render nothing on that side there. */}
        {!isMac && (
          <div
            style={{
              display: "flex",
              alignItems: "stretch",
              height: "100%",
              flexShrink: 0,
            }}
          >
            <Tooltip label="Minimize" side="bottom">
              <CaptionBtn onClick={() => win.minimize()}>
                <Minus size={15} strokeWidth={1.8} />
              </CaptionBtn>
            </Tooltip>
            <Tooltip label={maximized ? "Restore" : "Maximize"} side="bottom">
              <CaptionBtn onClick={() => win.toggleMaximize()}>
                {maximized ? (
                  <Copy
                    size={12}
                    strokeWidth={1.8}
                    style={{ transform: "scaleX(-1)" }}
                  />
                ) : (
                  <Square size={12} strokeWidth={1.8} />
                )}
              </CaptionBtn>
            </Tooltip>
            <Tooltip label="Close" side="bottom" align="end">
              <CaptionBtn onClick={() => win.close()} danger>
                <X size={16} strokeWidth={1.8} />
              </CaptionBtn>
            </Tooltip>
          </div>
        )}
      </div>
    </>
  );
}
