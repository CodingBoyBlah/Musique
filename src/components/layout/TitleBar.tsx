import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useNavigate } from "react-router-dom";
import {
  ChevronLeft, ChevronRight,
  MoreHorizontal, User, Settings, LogOut, LogIn,
  ListMusic, Captions, MonitorSpeaker,
  PanelLeft,
} from "@/lib/icons";
import { useAuth } from "../../hooks/useAuth";
import { useAuthStore } from "../../store/auth.store";
import { useCredentialsStore, type ConnectionStatus } from "../../store/credentials.store";
import { useDevices } from "../../hooks/useDevices";
import { Tooltip } from "../ui/Tooltip";
import { isMac } from "../../lib/platform";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { create } from "zustand";

export const useCaptionHoverStore = create<{
  hovered: "min" | "max" | "close" | null;
  setHovered: (b: "min" | "max" | "close" | null) => void;
}>((set) => ({
  hovered: null,
  setHovered: (b) => set({ hovered: b }),
}));

// win11 caption button (transparent and full-height)

function CaptionBtn({
  onClick,
  children,
  danger,
  isHovered,
  onHoverChange,
}: {
  onClick: () => void;
  children: React.ReactNode;
  danger?: boolean;
  isHovered?: boolean;
  onHoverChange?: (hover: boolean) => void;
}) {
  const [internalHover, setInternalHover] = useState(false);
  const hover = isHovered !== undefined ? (isHovered || internalHover) : internalHover;
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => {
        setInternalHover(true);
        onHoverChange?.(true);
      }}
      onMouseLeave={() => {
        setInternalHover(false);
        onHoverChange?.(false);
      }}
      style={{
        width:          46,
        height:         "100%",
        borderRadius:   0,
        border:         "none",
        background:     hover
          ? danger ? "#e81123" : "rgba(255,255,255,0.09)"
          : "transparent",
        color:          hover && danger ? "#fff" : hover ? "#ffffff" : "rgba(255,255,255,0.72)",
        cursor:         "pointer",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        transition:     "background 0.12s, color 0.12s",
        flexShrink:     0,
        ...({ WebkitAppRegion: "no-drag" } as React.CSSProperties),
      }}
    >
      {children}
    </button>
  );
}


// account menu (avatar / name header + Account · Settings · Log out)

const STATUS_DOT: Record<ConnectionStatus, string> = {
  unconfigured: "#34d399",
  configured:   "#f5a623",
  validating:   "#f5a623",
  valid:        "#34d399",
  invalid:      "#ff453a",
};
const STATUS_LABEL: Record<ConnectionStatus, string> = {
  unconfigured: "Shared Quota (Ready)",
  configured:   "Custom Quota (Saved)",
  validating:   "Testing API…",
  valid:        "Connected",
  invalid:      "Check API keys",
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
    <div ref={ref} style={{ position: "relative", display: "flex", alignItems: "center" }}>
      <Tooltip label="Account" side="bottom" align="start">
        <button
          onClick={() => setOpen((v) => !v)}
          style={{
            width: 26,
            height: 26,
            borderRadius: 9999,
            border: "none",
            background: open ? "rgba(255, 255, 255, 0.14)" : "transparent",
            color: open ? "#ffffff" : "var(--color-text)",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            transition: "background 0.15s, color 0.15s",
          }}
          onMouseEnter={(e) => {
            if (!open) {
              (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.08)";
              (e.currentTarget as HTMLElement).style.color = "#ffffff";
            }
          }}
          onMouseLeave={(e) => {
            if (!open) {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--color-text)";
            }
          }}
        >
          <MoreHorizontal size={15} strokeWidth={1.8} />
        </button>
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

export function WindowCaptionControls({ dockedToCard }: { dockedToCard?: boolean } = {}) {
  const [maximized, setMaximized] = useState(false);
  const macSimulated = useUIStore((s) => s.macSimulated);
  const effect = useUIStore((s) => s.windowEffect);
  const materialTransparency = useUIStore((s) => s.materialTransparency);
  const backdropActive = useUIStore((s) => s.backdropActive);
  const hovered = useCaptionHoverStore((s) => s.hovered);
  const setHovered = useCaptionHoverStore((s) => s.setHovered);

  useEffect(() => {
    if (isMac) return;
    const win = getCurrentWindow();
    win.isMaximized().then(setMaximized).catch(() => {});
    const unlisten = win.onResized(() => {
      win.isMaximized().then(setMaximized).catch(() => {});
    });
    return () => { unlisten.then((u) => u()).catch(() => {}); };
  }, []);

  if (isMac || macSimulated) return null;
  const win = getCurrentWindow();

  const isTranslucent = backdropActive && effect !== "none";
  const cardBg = isTranslucent
    ? `rgba(18, 18, 20, ${Math.max(0.66, Math.min(0.85, 0.78 * (1 - materialTransparency * 0.28)))})`
    : "rgba(19, 19, 19, 0.94)";

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        zIndex: 25,
        display: "flex",
        alignItems: "stretch",
        width: 138,
        height: 40,
        background: dockedToCard ? "transparent" : cardBg,
        backdropFilter: !dockedToCard && isTranslucent ? "blur(32px) saturate(140%)" : undefined,
        WebkitBackdropFilter: !dockedToCard && isTranslucent ? "blur(32px) saturate(140%)" : undefined,
        borderBottomLeftRadius: 10,
        borderLeft: dockedToCard ? "1px solid rgba(255, 255, 255, 0.08)" : "none",
        borderBottom: dockedToCard ? "1px solid rgba(255, 255, 255, 0.08)" : "none",
        borderTop: "none",
        borderRight: "none",
        overflow: "hidden",
        pointerEvents: "auto",
        ...({ WebkitAppRegion: "no-drag" } as React.CSSProperties),
      }}
    >
      <Tooltip label="Minimize" side="bottom">
        <CaptionBtn
          onClick={() => win.minimize()}
          isHovered={hovered === "min"}
          onHoverChange={(h) => setHovered(h ? "min" : null)}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M0 5H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </CaptionBtn>
      </Tooltip>
      <Tooltip label={maximized ? "Restore" : "Maximize"} side="bottom">
        <CaptionBtn
          onClick={() => win.toggleMaximize()}
          isHovered={hovered === "max"}
          onHoverChange={(h) => setHovered(h ? "max" : null)}
        >
          {maximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
              <path d="M2.5 0.5H9.5V7.5H7.5" fill="none" stroke="currentColor" strokeWidth="1" />
              <rect x="0.5" y="2.5" width="7" height="7" fill="none" stroke="currentColor" strokeWidth="1" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
              <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1" />
            </svg>
          )}
        </CaptionBtn>
      </Tooltip>
      <Tooltip label="Close" side="bottom" align="end">
        <CaptionBtn
          onClick={() => win.close()}
          danger
          isHovered={hovered === "close"}
          onHoverChange={(h) => setHovered(h ? "close" : null)}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
          </svg>
        </CaptionBtn>
      </Tooltip>
    </div>
  );
}

export function WindowCaptionHitboxOverlay() {
  const macSimulated = useUIStore((s) => s.macSimulated);
  const setHovered = useCaptionHoverStore((s) => s.setHovered);

  if (isMac || macSimulated) return null;
  const win = getCurrentWindow();

  const hitStyle: React.CSSProperties = {
    position: "absolute",
    pointerEvents: "auto",
    cursor: "pointer",
    background: "transparent",
    ...({ WebkitAppRegion: "no-drag" } as React.CSSProperties),
  };

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        zIndex: 9999,
        pointerEvents: "none",
        width: 142,
        height: 44,
      }}
    >
      {/* 4px top margin hitbox above Minimize */}
      <div
        style={{
          ...hitStyle,
          top: 0,
          right: 96,
          width: 46,
          height: 5,
        }}
        onMouseEnter={() => setHovered("min")}
        onMouseLeave={() => setHovered(null)}
        onClick={() => win.minimize()}
      />

      {/* 4px top margin hitbox above Maximize */}
      <div
        style={{
          ...hitStyle,
          top: 0,
          right: 50,
          width: 46,
          height: 5,
        }}
        onMouseEnter={() => setHovered("max")}
        onMouseLeave={() => setHovered(null)}
        onClick={() => win.toggleMaximize()}
      />

      {/* 4px top margin hitbox + exact top-right corner above Close */}
      <div
        style={{
          ...hitStyle,
          top: 0,
          right: 0,
          width: 50,
          height: 5,
        }}
        onMouseEnter={() => setHovered("close")}
        onMouseLeave={() => setHovered(null)}
        onClick={() => win.close()}
      />

      {/* 4px right margin hitbox along the right edge of Close */}
      <div
        style={{
          ...hitStyle,
          top: 5,
          right: 0,
          width: 5,
          height: 39,
        }}
        onMouseEnter={() => setHovered("close")}
        onMouseLeave={() => setHovered(null)}
        onClick={() => win.close()}
      />
    </div>
  );
}

export function TitleBar() {
  const navigate = useNavigate();
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const toggleLyrics = usePlayerStore((s) => s.toggleLyrics);
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const toggleQueue = usePlayerStore((s) => s.toggleQueue);
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const macSimulated = useUIStore((s) => s.macSimulated);
  const { activeDevice, isRemotePlayback, devicesOpen, toggleDevices } = useDevices();

  return (
    <div
      style={{
        height: 48,
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        width: "100%",
        paddingLeft: 12,
        paddingRight: (!isMac && !macSimulated && !lyricsOpen && !queueOpen) ? 0 : 12,
        position: "relative",
        zIndex: 10,
        pointerEvents: "auto",
      }}
    >
      {/* Left controls: single unified capsule pill matching Cider */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          height: 32,
          borderRadius: 9999,
          background: "rgba(255, 255, 255, 0.05)",
          border: "1px solid rgba(255, 255, 255, 0.10)",
          boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.07), 0 2px 8px rgba(0, 0, 0, 0.2)",
          backdropFilter: "blur(20px)",
          WebkitBackdropFilter: "blur(20px)",
          padding: 2,
          gap: 2,
          flexShrink: 0,
        }}
      >
        <AccountMenu />

        <div style={{ width: 1, height: 14, background: "rgba(255, 255, 255, 0.1)", margin: "0 1px" }} />

        <Tooltip label="Back" side="bottom">
          <button
            onClick={() => navigate(-1)}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: "transparent",
              color: "var(--color-text)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.08)";
              (e.currentTarget as HTMLElement).style.color = "#ffffff";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--color-text)";
            }}
          >
            <ChevronLeft size={15} strokeWidth={1.8} />
          </button>
        </Tooltip>

        <div style={{ width: 1, height: 14, background: "rgba(255, 255, 255, 0.1)", margin: "0 1px" }} />

        <Tooltip label="Forward" side="bottom">
          <button
            onClick={() => navigate(1)}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: "transparent",
              color: "var(--color-text)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.08)";
              (e.currentTarget as HTMLElement).style.color = "#ffffff";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--color-text)";
            }}
          >
            <ChevronRight size={15} strokeWidth={1.8} />
          </button>
        </Tooltip>

        <div style={{ width: 1, height: 14, background: "rgba(255, 255, 255, 0.1)", margin: "0 1px" }} />

        <Tooltip label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} side="bottom">
          <button
            onClick={toggleSidebar}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: sidebarCollapsed ? "rgba(255, 255, 255, 0.14)" : "transparent",
              color: sidebarCollapsed ? "#ffffff" : "var(--color-text)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              if (!sidebarCollapsed) {
                (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.08)";
                (e.currentTarget as HTMLElement).style.color = "#ffffff";
              }
            }}
            onMouseLeave={(e) => {
              if (!sidebarCollapsed) {
                (e.currentTarget as HTMLElement).style.background = "transparent";
                (e.currentTarget as HTMLElement).style.color = "var(--color-text)";
              }
            }}
          >
            <PanelLeft size={15} strokeWidth={1.75} />
          </button>
        </Tooltip>
      </div>

      {/* Center drag region */}
      <div data-tauri-drag-region style={{ flex: 1, height: "100%", cursor: "default" }} />

      {/* right actions capsule: lyrics, queue, devices */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          height: 32,
          borderRadius: 9999,
          background: "rgba(255, 255, 255, 0.06)",
          border: "1px solid rgba(255, 255, 255, 0.1)",
          boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.1), 0 2px 8px rgba(0, 0, 0, 0.25)",
          backdropFilter: "blur(20px)",
          WebkitBackdropFilter: "blur(20px)",
          padding: 2,
          gap: 2,
          flexShrink: 0,
          marginRight: (!isMac && !macSimulated && !lyricsOpen && !queueOpen) ? 148 : 0,
        }}
      >
        <Tooltip label={lyricsOpen ? "Close lyrics" : "Lyrics"} side="bottom">
          <button
            onClick={() => { if (currentTrack) toggleLyrics(); }}
            disabled={!currentTrack}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: lyricsOpen ? "rgba(255, 255, 255, 0.18)" : "transparent",
              color: lyricsOpen ? "#ffffff" : !currentTrack ? "rgba(255,255,255,0.25)" : "rgba(255, 255, 255, 0.72)",
              cursor: currentTrack ? "pointer" : "default",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxShadow: lyricsOpen ? "0 1px 4px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.15)" : "none",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              if (!lyricsOpen && currentTrack) {
                (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.1)";
                (e.currentTarget as HTMLElement).style.color = "#ffffff";
              }
            }}
            onMouseLeave={(e) => {
              if (!lyricsOpen && currentTrack) {
                (e.currentTarget as HTMLElement).style.background = "transparent";
                (e.currentTarget as HTMLElement).style.color = "rgba(255, 255, 255, 0.72)";
              }
            }}
          >
            <Captions size={15} strokeWidth={1.65} />
          </button>
        </Tooltip>

        <div style={{ width: 1, height: 14, background: "rgba(255, 255, 255, 0.1)", margin: "0 1px" }} />

        <Tooltip label={queueOpen ? "Close queue" : "Queue"} side="bottom">
          <button
            onClick={toggleQueue}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: queueOpen ? "rgba(255, 255, 255, 0.18)" : "transparent",
              color: queueOpen ? "#ffffff" : "rgba(255, 255, 255, 0.72)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxShadow: queueOpen ? "0 1px 4px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.15)" : "none",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              if (!queueOpen) {
                (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.1)";
                (e.currentTarget as HTMLElement).style.color = "#ffffff";
              }
            }}
            onMouseLeave={(e) => {
              if (!queueOpen) {
                (e.currentTarget as HTMLElement).style.background = "transparent";
                (e.currentTarget as HTMLElement).style.color = "rgba(255, 255, 255, 0.72)";
              }
            }}
          >
            <ListMusic size={15} strokeWidth={1.65} />
          </button>
        </Tooltip>

        <div style={{ width: 1, height: 14, background: "rgba(255, 255, 255, 0.1)", margin: "0 1px" }} />

        <Tooltip label={isRemotePlayback && activeDevice ? `Device: ${activeDevice.name}` : devicesOpen ? "Close devices" : "Devices"} side="bottom">
          <button
            data-devices-trigger="true"
            onClick={toggleDevices}
            style={{
              width: 26,
              height: 26,
              borderRadius: 9999,
              border: "none",
              background: devicesOpen ? "rgba(255, 255, 255, 0.18)" : isRemotePlayback ? "rgba(30, 215, 96, 0.18)" : "transparent",
              color: isRemotePlayback ? "var(--color-primary, #1ed760)" : devicesOpen ? "#ffffff" : "rgba(255, 255, 255, 0.72)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxShadow: devicesOpen ? "0 1px 4px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.15)" : "none",
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={(e) => {
              if (!devicesOpen && !isRemotePlayback) {
                (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.1)";
                (e.currentTarget as HTMLElement).style.color = "#ffffff";
              }
            }}
            onMouseLeave={(e) => {
              if (!devicesOpen && !isRemotePlayback) {
                (e.currentTarget as HTMLElement).style.background = "transparent";
                (e.currentTarget as HTMLElement).style.color = "rgba(255, 255, 255, 0.72)";
              }
            }}
          >
            <MonitorSpeaker size={15} strokeWidth={1.65} />
          </button>
        </Tooltip>
      </div>
    </div>
  );
}
