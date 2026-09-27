import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useLocation, useNavigate } from "react-router-dom";
import {
  ChevronLeft, ChevronRight,
  User, Settings, LogOut, LogIn,
  Queue, Captions, Devices,
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
import { TopSearch } from "./TopSearch";
import { EASE_OUT } from "../../lib/motion";

/* size of the windows caption cluster, measured from the window's top-right
   corner. the top bar is exactly this tall, so the buttons and the bar share
   one strip of OS material. */
export const CAPTION_W = 138;
export const CAPTION_H = 40;
export const TOP_BAR_H = CAPTION_H;

// win11 caption button (transparent and full-height)

// hover / press are CSS (.cap-btn in styles/layout.css): the old useState
// hover re-rendered the button on every enter/leave for a colour change
function CaptionBtn({
  onClick,
  children,
  danger,
  label,
}: {
  onClick: () => void;
  children: React.ReactNode;
  danger?: boolean;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      aria-label={label}
      className="cap-btn"
      data-danger={danger || undefined}
      style={{
        width:          46,
        height:         "100%",
        borderRadius:   0,
        border:         "none",
        cursor:         "pointer",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
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
  onClick: (e: React.MouseEvent<HTMLButtonElement>) => void;
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
        onClick={(e) => { if (!off) onClick(e); }}
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
// plain words: what the connection is doing, not how the quota is billed
const STATUS_LABEL: Record<ConnectionStatus, string> = {
  unconfigured: "Ready",
  configured:   "Using your own Spotify app",
  validating:   "Checking connection…",
  valid:        "Connected",
  invalid:      "Check API keys",
};

function AccountMenuItem({
  icon, label, onClick, danger,
}: {
  icon: React.ReactNode; label: string; onClick: () => void; danger?: boolean;
}) {
  return (
    <button
      role="menuitem"
      onClick={onClick}
      className="row-btn acct-item"
      data-danger={danger || undefined}
      style={{
        gap: 11,
        height: 36, padding: "0 10px", borderRadius: 8,
        color: danger ? "var(--color-danger)" : "var(--color-text-hi)",
        fontSize: 13, fontWeight: 500,
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
  const menuRef     = useRef<HTMLDivElement>(null);
  // opened from the keyboard -> move focus into the menu so arrows work
  const [keyboardOpened, setKeyboardOpened] = useState(false);
  const displayName = useAuthStore((s) => s.displayName);
  const imageUrl    = useAuthStore((s) => s.imageUrl);
  const loggedIn    = useAuthStore((s) => s.loggedIn);
  const status      = useCredentialsStore((s) => s.status);
  const { login, logout } = useAuth();

  const trigger = () => ref.current?.querySelector<HTMLButtonElement>(".tb-btn") ?? null;

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    // Escape closes and hands focus back to the avatar; arrows walk the items
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        setOpen(false);
        trigger()?.focus();
        return;
      }
      if (e.key !== "ArrowDown" && e.key !== "ArrowUp" && e.key !== "Home" && e.key !== "End") return;
      const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? []);
      if (items.length === 0) return;
      e.preventDefault();
      const at = items.indexOf(document.activeElement as HTMLButtonElement);
      const next =
        e.key === "Home" ? 0
        : e.key === "End" ? items.length - 1
        : e.key === "ArrowDown" ? (at + 1) % items.length
        : (at - 1 + items.length) % items.length;
      items[next]?.focus();
    };
    window.addEventListener("mousedown", onDown);
    // on document, not window: stopPropagation here keeps Escape from also
    // reaching window-level handlers (Immersive closes on Escape)
    document.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  useEffect(() => {
    if (!open || !keyboardOpened) return;
    menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')?.focus();
  }, [open, keyboardOpened]);

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
        onClick={(e) => {
          // detail 0 = activated by Enter/Space rather than a pointer
          setKeyboardOpened(e.detail === 0);
          setOpen((v) => !v);
        }}
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
          <User size={17} strokeWidth={1.75} />
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
            ref={menuRef}
            role="menu"
            aria-label="Account"
            className="glass-solid-fallback"
            initial={{ opacity: 0, y: -6, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            // out faster than in: the user has already decided
            exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.12, ease: EASE_OUT } }}
            transition={{ duration: 0.16, ease: EASE_OUT }}
            style={{
              position:      "absolute",
              top:           36,
              left:          0,
              width:         232,
              padding:       6,
              borderRadius:  12,
              transformOrigin: "top left",
              background:    "var(--color-popover)",
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
                  <span className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {STATUS_LABEL[status]}
                  </span>
                </span>
              </div>
            </div>

            <div style={{ height: 1, background: "var(--color-divider)", margin: "0 2px 6px" }} />

            {loggedIn && <AccountMenuItem icon={<User size={16} strokeWidth={2} />} label="Profile" onClick={() => go("/profile")} />}
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
   unbroken: the same OS material the top bar and sidebar sit on. the old
   dark chip drew a rectangle that belonged to nothing around it. it also
   sits flush in the window corner, which makes the close button the corner
   pixel - the whole point of a caption cluster. */
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
        <CaptionBtn label="Minimize" onClick={() => win.minimize()}>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M0 5H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </CaptionBtn>
      </Tooltip>
      <Tooltip label={maximized ? "Restore" : "Maximize"} side="bottom">
        <CaptionBtn label={maximized ? "Restore" : "Maximize"} onClick={() => win.toggleMaximize()}>
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
        <CaptionBtn label="Close" onClick={() => win.close()} danger>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
          </svg>
        </CaptionBtn>
      </Tooltip>
    </div>
  );
}

/* the top bar: a strip right of the sidebar, exactly as tall as the caption
   buttons, sitting straight on the OS material like the sidebar does.
   navigation on the left (flush with the card's left edge), search in the
   middle, panels on the right, caption buttons in the corner.

   the outer tracks are minmax(auto, 1fr): equal while there is room, so the
   search is centred over the card, but never narrower than their own
   buttons. the search track gives way first when the window gets tight. the
   right track carries the caption cluster's width as padding so the group
   never slides under the buttons. */
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

  const macChrome = isMac || macSimulated;

  return (
    <div
      data-tauri-drag-region
      style={{
        height: TOP_BAR_H,
        flexShrink: 0,
        display: "grid",
        gridTemplateColumns: "minmax(auto, 1fr) minmax(160px, 460px) minmax(auto, 1fr)",
        alignItems: "center",
        columnGap: 12,
        width: "100%",
        position: "relative",
        zIndex: 20,
        background: "transparent",
      }}
    >
      {/* who you are - where you have been - how much you can see.
          the dividers mark those three groups, so back/forward sit shoulder
          to shoulder: they are one control, not two. */}
      <div
        data-tauri-drag-region
        style={{ display: "flex", alignItems: "center", height: "100%", paddingLeft: 4 }}
      >
        <div className="tb-capsule">
          <AccountMenu />

          <span className="tb-sep" aria-hidden />

          <CapsuleButton
            label={canGoBack ? "Back" : "Nothing to go back to"}
            onClick={() => navigate(-1)}
            off={!canGoBack}
            aria-label="Back"
          >
            <ChevronLeft size={17} strokeWidth={1.75} />
          </CapsuleButton>

          <CapsuleButton
            label={canGoForward ? "Forward" : "Nothing to go forward to"}
            onClick={() => navigate(1)}
            off={!canGoForward}
            aria-label="Forward"
          >
            <ChevronRight size={17} strokeWidth={1.75} />
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
              ? <PanelLeft size={17} strokeWidth={1.75} />
              : <PanelLeftClose size={17} strokeWidth={1.75} />}
          </CapsuleButton>
        </div>
      </div>

      <TopSearch />

      {/* lyrics and queue share the right rail, so they are one group. where
          the sound comes out is a different question, past the divider. */}
      <div
        data-tauri-drag-region
        style={{
          display: "flex", alignItems: "center", justifyContent: "flex-end",
          height: "100%",
          /* mac: mirror the traffic lights' inset on the left, so the last
             icon isn't jammed against the window edge */
          paddingRight: macChrome ? 12 : CAPTION_W + 8,
        }}
      >
        <div className="tb-capsule">
          <CapsuleButton
            label={!hasTrack ? "Play a song to see lyrics" : lyricsOpen ? "Hide lyrics" : "Lyrics"}
            onClick={toggleLyrics}
            on={lyricsOpen}
            off={!hasTrack}
            aria-label="Lyrics"
            aria-pressed={lyricsOpen}
          >
            <Captions size={17} strokeWidth={1.75} active={lyricsOpen} />
          </CapsuleButton>

          <CapsuleButton
            label={queueOpen ? "Hide queue" : "Queue"}
            onClick={toggleQueue}
            on={queueOpen}
            aria-label="Queue"
            aria-pressed={queueOpen}
          >
            <Queue size={17} strokeWidth={1.75} active={queueOpen} />
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
            <Devices size={17} strokeWidth={1.75} active={devicesOpen || isRemotePlayback} />
          </CapsuleButton>
        </div>
      </div>
    </div>
  );
}
