import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useLocation, useNavigate } from "react-router-dom";
import {
  ChevronLeft, ChevronRight,
  User, Settings, LogOut, LogIn,
  ListMusic, Captions, MonitorSpeaker,
  PanelLeft, PanelLeftClose,
} from "@/lib/icons";
import { useAuth } from "../../hooks/useAuth";
import { useAuthStore } from "../../store/auth.store";
import { useCredentialsStore, type ConnectionStatus } from "../../store/credentials.store";
import { useDevices } from "../../hooks/useDevices";
import { Tooltip } from "../ui/Tooltip";
import { isMac } from "../../lib/platform";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";

/* size of the windows caption cluster, measured from the window's top-right
   corner. the island card is inset 4px, so the notch carved out of the card
   is CAPTION_W - 4 by CAPTION_H - 4 (see Layout). */
export const CAPTION_W = 138;
export const CAPTION_H = 40;

// win11 caption button (transparent and full-height)

function CaptionBtn({
  onClick,
  children,
  danger,
}: {
  onClick: () => void;
  children: React.ReactNode;
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


/* one control inside a title-bar capsule.

   hover / press / focus live in CSS (.tb-btn in index.css) rather than in
   onMouseEnter handlers that write straight to node.style - those got wiped
   by any unrelated re-render while the cursor was still on the button, so the
   hover fill would silently vanish. `off` marks a control that is present but
   has nothing to act on; it stays hoverable (and keeps its tooltip, which is
   where the reason lives) instead of going `disabled` and swallowing the
   pointer events the tooltip needs. */
function CapsuleButton({
  label, onClick, children, on, off, tone, align, ...rest
}: {
  label: React.ReactNode;
  onClick: () => void;
  children: React.ReactNode;
  on?: boolean;
  off?: boolean;
  tone?: "remote";
  align?: "center" | "start" | "end";
} & Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onClick" | "children">) {
  return (
    <Tooltip label={label} side="bottom" align={align}>
      <button
        className="tb-btn"
        onClick={() => { if (!off) onClick(); }}
        aria-disabled={off || undefined}
        data-on={on ? "true" : undefined}
        data-tone={tone}
        {...rest}
      >
        {children}
      </button>
    </Tooltip>
  );
}

/* can we actually go back / forward?

   react-router's BrowserRouter stamps its position into history.state.idx, so
   that index plus history.length says whether either arrow has anywhere to
   go. if the stamp is missing (first paint, a non-router entry) assume both
   work - never block navigation over a missing hint. */
function useHistoryEdges() {
  const location = useLocation();
  const [edges, setEdges] = useState({ back: true, forward: true });

  useEffect(() => {
    const read = () => {
      const idx = (window.history.state as { idx?: number } | null)?.idx;
      if (typeof idx !== "number") { setEdges({ back: true, forward: true }); return; }
      setEdges({ back: idx > 0, forward: idx < window.history.length - 1 });
    };
    read();
    window.addEventListener("popstate", read);
    return () => window.removeEventListener("popstate", read);
  }, [location.key]);

  return edges;
}


// account menu (avatar / name header + Account - Settings - Log out)

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

  /* the avatar IS the button. a "..." glyph said nothing about whose account
     this is; the photo does, and it is the affordance people already know
     from every other player. inset to 20px inside the 26px hit target so the
     hover / open fill reads as a ring around it. */
  const needsAttention = status === "invalid";

  return (
    <div ref={ref} style={{ position: "relative", display: "flex", alignItems: "center" }}>
      <CapsuleButton
        label={needsAttention ? STATUS_LABEL.invalid : (displayName ?? "Account")}
        align="start"
        onClick={() => setOpen((v) => !v)}
        on={open}
        aria-label="Account"
        aria-expanded={open}
        aria-haspopup="menu"
        style={{ position: "relative" }}
      >
        {imageUrl ? (
          <img
            src={imageUrl}
            alt=""
            style={{ width: 20, height: 20, borderRadius: "50%", objectFit: "cover", display: "block" }}
          />
        ) : (
          <User size={15} strokeWidth={1.9} />
        )}
        {needsAttention && (
          <span
            aria-hidden
            style={{
              position: "absolute", right: 0, bottom: 0,
              width: 8, height: 8, borderRadius: "50%",
              background: "var(--color-danger)",
              boxShadow: "0 0 0 1.5px rgba(16, 16, 20, 0.95)",
            }}
          />
        )}
      </CapsuleButton>
      <AnimatePresence>
        {open && (
          <motion.div
            role="menu"
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

/* minimise / maximise / close.

   the cluster paints NOTHING of its own - no fill, no border, no blur. the
   window root is transparent, so whatever is behind it shows through
   unbroken: the OS Mica the sidebar sits on when the island card is clipped
   away for it (Layout owns that notch), and the lyrics or queue rail, at its
   own transparency, when one of those is open. the old dark chip drew a
   rectangle that belonged to neither. it also sits flush in the window
   corner, which makes the close button the corner pixel - the whole point of
   a caption cluster. */
export function WindowCaptionControls() {
  const [maximized, setMaximized] = useState(false);
  const macSimulated = useUIStore((s) => s.macSimulated);

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

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        zIndex: 25,
        display: "flex",
        alignItems: "stretch",
        width: CAPTION_W,
        height: CAPTION_H,
        background: "transparent",
        borderBottomLeftRadius: 10,
        border: "none",
        overflow: "hidden",
        pointerEvents: "auto",
        ...({ WebkitAppRegion: "no-drag" } as React.CSSProperties),
      }}
    >
      <Tooltip label="Minimize" side="bottom">
        <CaptionBtn onClick={() => win.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M0 5H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </CaptionBtn>
      </Tooltip>
      <Tooltip label={maximized ? "Restore" : "Maximize"} side="bottom">
        <CaptionBtn onClick={() => win.toggleMaximize()}>
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
        <CaptionBtn onClick={() => win.close()} danger>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
          </svg>
        </CaptionBtn>
      </Tooltip>
    </div>
  );
}

export function TitleBar() {
  const navigate = useNavigate();
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const toggleLyrics = usePlayerStore((s) => s.toggleLyrics);
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const toggleQueue = usePlayerStore((s) => s.toggleQueue);
  const hasTrack = usePlayerStore((s) => s.currentTrack !== null);
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const macSimulated = useUIStore((s) => s.macSimulated);
  const { activeDevice, isRemotePlayback, devicesOpen, toggleDevices } = useDevices();
  const { back: canGoBack, forward: canGoForward } = useHistoryEdges();

  const docked = !isMac && !macSimulated && !lyricsOpen && !queueOpen;

  return (
    <div
      style={{
        height: 48,
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        width: "100%",
        paddingLeft: 12,
        paddingRight: docked ? 0 : 12,
        position: "relative",
        zIndex: 10,
        pointerEvents: "auto",
      }}
    >
      {/* who you are - where you have been - how much you can see.
          the dividers mark those three groups, so back/forward now sit
          shoulder to shoulder: they are one control, not two. */}
      <div className="tb-capsule">
        <AccountMenu />

        <span className="tb-sep" aria-hidden />

        <CapsuleButton
          label={canGoBack ? "Back" : "Nothing to go back to"}
          onClick={() => navigate(-1)}
          off={!canGoBack}
          aria-label="Back"
        >
          <ChevronLeft size={15} strokeWidth={1.9} />
        </CapsuleButton>

        <CapsuleButton
          label={canGoForward ? "Forward" : "Nothing to go forward to"}
          onClick={() => navigate(1)}
          off={!canGoForward}
          aria-label="Forward"
        >
          <ChevronRight size={15} strokeWidth={1.9} />
        </CapsuleButton>

        <span className="tb-sep" aria-hidden />

        {/* the icon shows what the click will do, so the button never has to
            sit in a lit "on" state that reads as a selected mode */}
        <CapsuleButton
          label={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
          onClick={toggleSidebar}
          aria-label={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
        >
          {sidebarCollapsed
            ? <PanelLeft size={15} strokeWidth={1.9} />
            : <PanelLeftClose size={15} strokeWidth={1.9} />}
        </CapsuleButton>
      </div>

      {/* Center drag region */}
      <div data-tauri-drag-region style={{ flex: 1, height: "100%", cursor: "default" }} />

      {/* lyrics and queue share the right rail, so they are one group. where
          the sound comes out is a different question, past the divider. */}
      <div
        className="tb-capsule"
        style={{ marginRight: docked ? CAPTION_W - 4 + 8 : 0 }}
      >
        <CapsuleButton
          label={!hasTrack ? "Play a song to see lyrics" : lyricsOpen ? "Hide lyrics" : "Lyrics"}
          onClick={toggleLyrics}
          on={lyricsOpen}
          off={!hasTrack}
          aria-label="Lyrics"
          aria-pressed={lyricsOpen}
        >
          <Captions size={15} strokeWidth={1.75} />
        </CapsuleButton>

        <CapsuleButton
          label={queueOpen ? "Hide queue" : "Queue"}
          onClick={toggleQueue}
          on={queueOpen}
          aria-label="Queue"
          aria-pressed={queueOpen}
        >
          <ListMusic size={15} strokeWidth={1.75} />
        </CapsuleButton>

        <span className="tb-sep" aria-hidden />

        <CapsuleButton
          label={isRemotePlayback && activeDevice ? `Playing on ${activeDevice.name}` : devicesOpen ? "Hide devices" : "Devices"}
          onClick={toggleDevices}
          on={devicesOpen && !isRemotePlayback}
          tone={isRemotePlayback ? "remote" : undefined}
          align="end"
          aria-label="Devices"
          aria-expanded={devicesOpen}
          data-devices-trigger="true"
        >
          <MonitorSpeaker size={15} strokeWidth={1.75} />
        </CapsuleButton>
      </div>
    </div>
  );
}
