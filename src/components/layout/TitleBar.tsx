import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useNavigate } from "react-router-dom";
import {
  ChevronLeft, ChevronRight, Minus, Square, Copy, X,
  MoreHorizontal, User, Settings, LogOut, LogIn,
} from "lucide-react";
import { useAuth } from "../../hooks/useAuth";
import { useAuthStore } from "../../store/auth.store";
import { useCredentialsStore, type ConnectionStatus } from "../../store/credentials.store";
import { Tooltip } from "../ui/Tooltip";
import { isMac } from "../../lib/platform";
import { gpuLayer, zTransform } from "../../lib/motion";

// win11 caption button (transparent and full-height)

function CaptionBtn({
  children, onClick, danger,
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
        width:          46,
        height:         "100%",
        border:         "none",
        background:     hover
          ? danger ? "#c42b1c" : "rgba(255,255,255,0.08)"
          : "transparent",
        color:          hover && danger ? "#fff" : "rgba(255,255,255,0.78)",
        cursor:         "pointer",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        transition:     "background 0.12s, color 0.12s",
        flexShrink:     0,
      }}
    >
      {children}
    </button>
  );
}

// round nav / ellipsis button

function PillBtn({
  children, onClick, wide, nudge,
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
        height:         30,
        width:          wide ? 38 : 30,
        borderRadius:   wide ? 8 : "50%",
        // borderless at rest, border only eases in on hover
        border:         hover ? "1px solid var(--color-glass-border)" : "1px solid transparent",
        background:     hover ? "rgba(255,255,255,0.10)" : "transparent",
        color:          hover ? "var(--color-text-hi)" : "var(--color-text)",
        cursor:         "pointer",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        transition:     "background 0.18s ease, color 0.18s ease, border-color 0.18s ease",
        flexShrink:     0,
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

// account menu (avatar / name header + Account · Settings · Log out)

const STATUS_DOT: Record<ConnectionStatus, string> = {
  unconfigured: "rgba(255,255,255,0.30)",
  configured:   "#f5a623",
  validating:   "#f5a623",
  valid:        "#34d399",
  invalid:      "#ff453a",
};
const STATUS_LABEL: Record<ConnectionStatus, string> = {
  unconfigured: "API not configured",
  configured:   "Saved not tested",
  validating:   "Connecting…",
  valid:        "Connected",
  invalid:      "Check your API keys",
};

function AccountMenuItem({
  icon, label, onClick, danger,
}: {
  icon: React.ReactNode; label: string; onClick: () => void; danger?: boolean;
}) {
  const [hover, setHover] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: "flex", alignItems: "center", gap: 11, width: "100%",
        height: 36, padding: "0 10px", borderRadius: 8, border: "none",
        background: hover ? (danger ? "rgba(255,69,58,0.14)" : "var(--color-hover)") : "transparent",
        color: danger ? "var(--color-danger)" : "var(--color-text-hi)",
        fontSize: 13, fontWeight: 500, cursor: "pointer", textAlign: "left",
        transition: "background 0.1s",
      }}
    >
      <span style={{ width: 16, display: "flex", color: danger ? "var(--color-danger)" : "var(--color-text)" }}>{icon}</span>
      <span style={{ flex: 1 }}>{label}</span>
    </button>
  );
}

function AccountMenu() {
  const [open, setOpen] = useState(false);
  const navigate    = useNavigate();
  const ref         = useRef<HTMLDivElement>(null);
  const displayName = useAuthStore((s) => s.displayName);
  const imageUrl    = useAuthStore((s) => s.imageUrl);
  const loggedIn    = useAuthStore((s) => s.loggedIn);
  const status      = useCredentialsStore((s) => s.status);
  const { login, logout } = useAuth();

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [open]);

  const go = (path: string) => { setOpen(false); navigate(path); };

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <Tooltip label="Account" side="bottom" align="start">
        <PillBtn wide onClick={() => setOpen((v) => !v)}>
          <MoreHorizontal size={16} strokeWidth={2.5} />
        </PillBtn>
      </Tooltip>
      <AnimatePresence>
        {open && (
          <motion.div
            initial={{ opacity: 0, y: -6, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -6, scale: 0.97 }}
            transition={{ duration: 0.16, ease: [0.23, 1, 0.32, 1] }}
            style={{
              position:      "absolute",
              top:           38,
              left:          0,
              width:         232,
              padding:       6,
              borderRadius:  14,
              transformOrigin: "top left",
              background:    "rgba(28, 28, 32, 0.92)",
              backdropFilter: "blur(40px) saturate(1.4)",
              WebkitBackdropFilter: "blur(40px) saturate(1.4)",
              border:        "1px solid var(--color-border)",
              boxShadow:     "0 16px 40px rgba(0,0,0,0.5)",
              zIndex:        100,
            }}
          >
            {/* identity header */}
            <div style={{ display: "flex", alignItems: "center", gap: 11, padding: "8px 8px 12px" }}>
              {imageUrl ? (
                <img src={imageUrl} alt="" style={{ width: 38, height: 38, borderRadius: "50%", objectFit: "cover", flexShrink: 0, outline: "1px solid rgba(255,255,255,0.1)" }} />
              ) : (
                <div style={{ width: 38, height: 38, borderRadius: "50%", flexShrink: 0, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <User size={18} strokeWidth={2} style={{ color: "var(--color-text-dim)" }} />
                </div>
              )}
              <div style={{ minWidth: 0, flex: 1 }}>
                <p style={{ margin: 0, fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {loggedIn ? (displayName ?? "Your account") : "Not signed in"}
                </p>
                <span style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 3 }}>
                  <span style={{ width: 7, height: 7, borderRadius: "50%", background: STATUS_DOT[status], flexShrink: 0 }} />
                  <span style={{ fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {STATUS_LABEL[status]}
                  </span>
                </span>
              </div>
            </div>

            <div style={{ height: 1, background: "var(--color-divider)", margin: "0 2px 6px" }} />

            <AccountMenuItem icon={<User size={16} strokeWidth={2} />}     label="Account"  onClick={() => go("/profile")} />
            <AccountMenuItem icon={<Settings size={16} strokeWidth={2} />} label="Settings" onClick={() => go("/settings")} />

            <div style={{ height: 1, background: "var(--color-divider)", margin: "6px 2px" }} />

            {loggedIn ? (
              <AccountMenuItem icon={<LogOut size={16} strokeWidth={2} />} label="Log out" danger onClick={() => { setOpen(false); logout(); }} />
            ) : (
              <AccountMenuItem icon={<LogIn size={16} strokeWidth={2} />} label="Log in" onClick={() => { setOpen(false); login(); }} />
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export function TitleBar() {
  const navigate = useNavigate();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    win.isMaximized().then(setMaximized).catch(() => {});
    const unlisten = win.onResized(() => {
      win.isMaximized().then(setMaximized).catch(() => {});
    });
    return () => { unlisten.then((u) => u()).catch(() => {}); };
  }, []);

  const win = getCurrentWindow();

  return (
    <>
      {/* macOS: full-width draggable strip above the bar, matching the sidebar
          one, so the native traffic lights get clean vertical space and the
          whole top row stays aligned across both columns */}
      {isMac && <div data-tauri-drag-region style={{ height: 30, flexShrink: 0 }} />}
      <div
        style={{
          height:      48,
          flexShrink:  0,
          display:     "flex",
          alignItems:  "center",
        }}
      >
      {/* left controls: menu + back/forward. no mac inset needed, the traffic
          lights sit over the sidebar column not here */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 12px", flexShrink: 0 }}>
        {/* account + settings + log out live here now (window material moved to
            Settings → Window). shown on every platform. */}
        <AccountMenu />
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
      <div data-tauri-drag-region style={{ flex: 1, height: "100%", cursor: "default" }} />

      {/* custom caption buttons - windows/linux only. macOS uses its native
          traffic lights so we render nothing on that side there. */}
      {!isMac && (
      <div style={{ display: "flex", alignItems: "stretch", height: "100%", flexShrink: 0 }}>
        <Tooltip label="Minimize" side="bottom">
          <CaptionBtn onClick={() => win.minimize()}>
            <Minus size={15} strokeWidth={1.8} />
          </CaptionBtn>
        </Tooltip>
        <Tooltip label={maximized ? "Restore" : "Maximize"} side="bottom">
          <CaptionBtn onClick={() => win.toggleMaximize()}>
            {maximized
              ? <Copy   size={12} strokeWidth={1.8} style={{ transform: "scaleX(-1)" }} />
              : <Square size={12} strokeWidth={1.8} />
            }
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
