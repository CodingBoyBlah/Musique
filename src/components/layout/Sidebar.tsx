import { useState, useEffect } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import {
  Home, ListMusic,
  Music, Disc3, User,
  Pin, PinOff,
  ChevronDown,
  type LucideIcon,
} from "@/lib/icons";
import { usePinsStore } from "../../store/pins.store";
import { useUIStore } from "../../store/ui.store";
import { useContextMenu } from "../ui/ContextMenu";
import { gpuLayer, zTransform, EASE_OUT, SPRING, PRESS_TRANSITION } from "../../lib/motion";
import { useQueryClient } from "@tanstack/react-query";
import { prefetchPlaylist, prefetchAlbum } from "../../lib/prefetch";
import { Tooltip } from "../ui/Tooltip";
import { isMac } from "../../lib/platform";

// nav item. active state passed in explicitly so we don't get multi highlight

function NavItem({
  icon: Icon, label, active, onClick, collapsed,
}: {
  icon: LucideIcon; label: string; active: boolean; onClick: () => void; collapsed?: boolean;
}) {
  const btn = (
    <motion.button
      // hover fill is CSS (.sb-item, hover-capable pointers only); framer's
      // whileHover also fired on touch taps and stuck there
      className="nav-item sb-item focus-ring"
      data-active={active || undefined}
      aria-current={active ? "page" : undefined}
      aria-label={collapsed ? label : undefined}
      onClick={onClick}
      whileTap={{ scale: 0.98 }}
      transition={PRESS_TRANSITION}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        position:      "relative",
        display:       "flex",
        alignItems:    "center",
        justifyContent: collapsed ? "center" : "flex-start",
        gap:           collapsed ? 0 : 11,
        height:        34,
        /* collapsed: fixed 34x34 square, centred in the column, so the active
         pill (inset: 0) is a perfect square */
        width:         collapsed ? 34 : "100%",
        margin:        collapsed ? "0 auto" : undefined,
        padding:       collapsed ? 0 : "0 10px",
        borderRadius:  8,
        border:        "none",
        fontSize:      13.5,
        fontWeight:    active ? 600 : 500,
        color:         active ? "#ffffff" : "var(--color-text)",
        background:    "transparent",
        cursor:        "pointer",
        textAlign:     collapsed ? "center" : "left",
      }}
    >
      {active && (
        <motion.div
          layoutId="activeNavPill"
          transition={SPRING}
          style={{
            position: "absolute",
            inset: 0,
            borderRadius: 8,
            background: "var(--color-accent)",
            boxShadow: "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))",
            zIndex: 0,
          }}
        />
      )}
      <span className="nav-icon" data-active={active || undefined} style={{
        position: "relative",
        zIndex: 1,
        width: 20, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
      }}>
        <Icon size={18} strokeWidth={1.7} active={active} />
      </span>
      {!collapsed && (
        <span style={{ position: "relative", zIndex: 1, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: active ? "#ffffff" : "inherit" }}>{label}</span>
      )}
    </motion.button>
  );

  if (collapsed) {
    return <Tooltip label={label} side="right">{btn}</Tooltip>;
  }
  return btn;
}

// collapsible section

function Section({
  label, expanded, onToggle, children, collapsed,
}: {
  label: string; expanded: boolean; onToggle: () => void; children: React.ReactNode; collapsed?: boolean;
}) {
  if (collapsed) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: 3, marginBottom: 8 }}>
        <div style={{ height: 1, background: "var(--color-border)", margin: "4px 6px 6px" }} />
        {children}
      </div>
    );
  }

  return (
    <div>
      <button
        onClick={onToggle}
        aria-expanded={expanded}
        className="sb-section-head focus-ring"
        style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          width: "100%", height: 28, padding: "0 10px", border: "none", background: "transparent",
          borderRadius: 6,
          color: "rgba(255, 255, 255, 0.45)", fontSize: 11, fontWeight: 700,
          letterSpacing: "0.08em", textTransform: "uppercase", cursor: "pointer",
          font: "inherit",
        }}
      >
        <span style={{ fontSize: 11 }}>{label}</span>
        <motion.span animate={{ rotate: expanded ? 0 : -90 }} transition={{ duration: 0.18, ease: EASE_OUT }} style={{ display: "flex" }}>
          <ChevronDown size={12} strokeWidth={2.5} />
        </motion.span>
      </button>
      <AnimatePresence initial={false}>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: EASE_OUT }}
            style={{ overflow: "hidden" }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: 2, padding: "2px 0 6px" }}>
              {children}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

// sidebar

export default function Sidebar() {
  const navigate    = useNavigate();
  const location    = useLocation();
  const pins        = usePinsStore((s) => s.pins);
  const removePin   = usePinsStore((s) => s.removePin);
  const qc          = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const path = location.pathname;
  const tab  = new URLSearchParams(location.search).get("tab") ?? "songs";
  const onLibrary = path === "/library";

  /* which library item (if any) is open + is it pinned. lets the sidebar light
   up the specific pinned playlist when its open, and only fall back to
   lighting the "Playlists" button for unpinned ones */

  const openMatch      = path.match(/^\/(playlist|album)\/(.+)$/);
  const openType       = openMatch?.[1] ?? null;   // "playlist" | "album"
  const openId         = openMatch?.[2] ?? null;
  const openIsPinned   = openId != null && pins.some((p) => p.id === openId);
  const onUnpinnedPlaylist = openType === "playlist" && !openIsPinned;

  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const macSimulated     = useUIStore((s) => s.macSimulated);
  const [isNarrow, setIsNarrow] = useState(() =>
    typeof window !== "undefined" ? window.innerWidth < 768 : false
  );

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mql = window.matchMedia("(max-width: 767px)");
    const handler = (e: MediaQueryListEvent) => setIsNarrow(e.matches);
    setIsNarrow(mql.matches);
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  const isCollapsed = sidebarCollapsed || isNarrow;
  /* mac keeps its native decorations, so the traffic lights land in this
     column at fixed OS coordinates: 12px dots on a 20px pitch, first centre
     at x=20, last edge at x=66. the collapsed rail widens to 72 so all three
     still fit instead of being cropped down to the red one. */
  const macChrome = isMac || macSimulated;
  const railWidth = isCollapsed ? (macChrome ? 72 : 64) : 232;

  const [spotifyOpen, setSpotifyOpen] = useState(true);
  const [libraryOpen, setLibraryOpen] = useState(true);
  const [pinsOpen,    setPinsOpen]    = useState(true);

  return (
    /* The width tween stays: the rail's labels clip rather than reflow as it
       narrows, and a 220ms ease-out is short enough that the per-frame layout
       it costs never lands on anything the user is reading. Replacing it with
       a transform would mean the page card no longer tracks the rail edge. */
    <motion.nav
      aria-label="Sidebar"
      animate={{ width: railWidth }}
      transition={{ duration: 0.22, ease: EASE_OUT }}
      style={{
        width:         railWidth,
        flexShrink:    0,
        display:       "flex",
        flexDirection: "column",
        overflow:      "hidden",
        background:    "transparent",
        borderRight:   "none",
        /* the rail runs to the top of the window. on windows the first
           section header is pushed down just enough to sit on the top bar's
           centre line (40px bar: 2px strip + 4px nav padding + 28px header),
           so the two read as one row */
        paddingTop:    macChrome ? 4 : 0,
      }}
    >
      {macChrome ? (
        /* macOS traffic lights (native Mac OR Ctrl+Shift+M simulated on
           Windows). the padding and gap mirror the native geometry exactly,
           in both the expanded and the collapsed rail, so these sit under the
           real buttons rather than beside them. */
        <div
          data-tauri-drag-region
          style={{ height: 32, display: "flex", alignItems: "center", gap: 8, padding: "0 14px", flexShrink: 0 }}
          title={!isMac ? "Mac traffic lights preview (Ctrl+Shift+M to toggle)" : undefined}
        >
          {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
            <div
              key={c}
              style={{
                width: 12, height: 12, flexShrink: 0, borderRadius: "50%",
                background: c, boxShadow: "inset 0 0 0 1px rgba(0, 0, 0, 0.18)",
              }}
            />
          ))}
        </div>
      ) : (
        // a strip of window-drag above the first header
        <div data-tauri-drag-region style={{ height: 2, flexShrink: 0 }} />
      )}

      {/* nav */}
      <div style={{ flex: 1, overflowY: "auto", padding: isCollapsed ? "4px 6px" : "4px 8px" }}>
        <Section label="Discover" expanded={spotifyOpen} onToggle={() => setSpotifyOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={Home} label="Home"      active={path === "/"}                                          onClick={() => navigate("/")} collapsed={isCollapsed} />
          <NavItem icon={ListMusic} label="Playlists" active={path === "/playlists" || onUnpinnedPlaylist}           onClick={() => navigate("/playlists")} collapsed={isCollapsed} />
        </Section>

        <Section label="Library" expanded={libraryOpen} onToggle={() => setLibraryOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={Music} label="Songs"   active={onLibrary && tab === "songs"}   onClick={() => navigate("/library?tab=songs")} collapsed={isCollapsed} />
          <NavItem icon={Disc3} label="Albums"  active={onLibrary && tab === "albums"}  onClick={() => navigate("/library?tab=albums")} collapsed={isCollapsed} />
          <NavItem icon={User} label="Artists" active={onLibrary && tab === "artists"} onClick={() => navigate("/library?tab=artists")} collapsed={isCollapsed} />
        </Section>

        <Section label="Pins" expanded={pinsOpen} onToggle={() => setPinsOpen(v => !v)} collapsed={isCollapsed}>
          {pins.length === 0 ? (
            !isCollapsed ? (
              <div style={{ padding: "0 2px" }}>
                <div
                  className="t-caption"
                  style={{
                    borderRadius: 8,
                    border:       "1.5px dashed var(--color-glass-border)",
                    background:   "var(--color-glass)",
                    padding:      "12px 13px",
                    fontSize:     12,
                    color:        "var(--color-text-dim)",
                    lineHeight:   1.5,
                    display:      "flex",
                    alignItems:   "flex-start",
                    gap:          8,
                  }}
                >
                  <Pin size={13} strokeWidth={2} style={{ flexShrink: 0, marginTop: 1 }} />
                  <span>No pins yet. Right-click a playlist to pin it.</span>
                </div>
              </div>
            ) : null
          ) : (
            pins.map((p) => {
              const active = openId === p.id && openType === p.type;
              const btn = (
                <motion.button
                  key={p.id}
                  onClick={() => navigate(`/${p.type}/${p.id}`)}
                  onContextMenu={openMenu([
                    { label: "Unpin", icon: <PinOff size={14} />, onSelect: () => removePin(p.id) },
                  ])}
                  title={isCollapsed ? undefined : p.name}
                  aria-label={isCollapsed ? p.name : undefined}
                  aria-current={active ? "page" : undefined}
                  className="sb-item focus-ring"
                  data-active={active || undefined}
                  whileTap={{ scale: 0.98 }}
                  transition={PRESS_TRANSITION}
                  transformTemplate={zTransform}
                  style={{
                    ...gpuLayer,
                    position:      "relative",
                    display:       "flex",
                    alignItems:    "center",
                    justifyContent: isCollapsed ? "center" : "flex-start",
                    gap:           isCollapsed ? 0 : 10,
                    height:        34,
                    width:         isCollapsed ? 34 : "100%",
                    margin:        isCollapsed ? "0 auto" : undefined,
                    padding:       isCollapsed ? 0 : "0 8px",
                    borderRadius:  8,
                    border:        "none",
                    background:    "transparent",
                    color:         active ? "#ffffff" : "var(--color-text)",
                    cursor:        "pointer",
                    textAlign:     isCollapsed ? "center" : "left",
                  }}
                  onPointerEnter={() => {
                    if (p.type === "album") prefetchAlbum(qc, p.id);
                    else prefetchPlaylist(qc, p.id);
                  }}
                >
                  {active && (
                    <motion.div
                      layoutId="activeNavPill"
                      transition={SPRING}
                      style={{
                        position: "absolute",
                        inset: 0,
                        borderRadius: 8,
                        background: "var(--color-accent)",
                        boxShadow: "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))",
                        zIndex: 0,
                      }}
                    />
                  )}
                  <span style={{ position: "relative", zIndex: 1, display: "flex", flexShrink: 0 }}>
                    {p.image_url ? (
                      <img src={coverUrl(p.image_url, 26) ?? p.image_url} alt="" style={{ width: 26, height: 26, borderRadius: 5, objectFit: "cover" }} />
                    ) : (
                      <div style={{ width: 26, height: 26, borderRadius: 5, background: active ? "rgba(255,255,255,0.2)" : "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                        <ListMusic size={14} style={{ color: active ? "#ffffff" : "var(--color-text-dim)" }} />
                      </div>
                    )}
                  </span>
                  {!isCollapsed && (
                    <span style={{ position: "relative", zIndex: 1, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: 13, fontWeight: active ? 600 : 500 }}>
                      {p.name}
                    </span>
                  )}
                </motion.button>
              );

              if (isCollapsed) {
                return <Tooltip key={p.id} label={p.name} side="right">{btn}</Tooltip>;
              }
              return btn;
            })
          )}
        </Section>
      </div>
      {menuEl}
    </motion.nav>
  );
}
