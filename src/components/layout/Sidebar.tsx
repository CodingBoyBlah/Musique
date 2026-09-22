import { useState, useRef, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import {
  Home, ListMusic,
  Music, Disc3, Users,
  Pin, PinOff,
  Search, ChevronDown,
  PanelLeftClose,
} from "@/lib/icons";
import { usePinsStore } from "../../store/pins.store";
import { useUIStore } from "../../store/ui.store";
import { useContextMenu } from "../ui/ContextMenu";
import { gpuLayer, zTransform } from "../../lib/motion";
import { isMac } from "../../lib/platform";
import { useQueryClient } from "@tanstack/react-query";
import { prefetchPlaylist, prefetchAlbum } from "../../lib/prefetch";
import { Tooltip } from "../ui/Tooltip";

// frosted glass pill -- (search bar / account bar)

const glassPill: React.CSSProperties = {
  width:        "100%",
  height:       32,
  borderRadius: 8,
  background:   "rgba(255, 255, 255, 0.04)",
  border:       "1px solid rgba(255, 255, 255, 0.08)",
  display:      "flex",
  alignItems:   "center",
  gap:          8,
  padding:      "0 10px",
  flexShrink:   0,
};

// nav item. active state passed in explicitly so we don't get multi highlight

function NavItem({
  icon, label, active, onClick, collapsed,
}: {
  icon: React.ReactNode; label: string; active: boolean; onClick: () => void; collapsed?: boolean;
}) {
  const btn = (
    <motion.button
      onClick={onClick}
      whileTap={{ scale: 0.98 }}
      whileHover={active ? {} : { backgroundColor: "var(--color-hover)" }}
      transition={{ type: "spring", stiffness: 400, damping: 26 }}
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
          transition={{ type: "spring", stiffness: 500, damping: 36 }}
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
      <span style={{
        position: "relative",
        zIndex: 1,
        width: 20, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
        color: active ? "#ffffff" : "var(--color-text-dim)",
        transition: "color 0.15s ease",
      }}>
        {icon}
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
        style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          width: "100%", height: 28, padding: "0 10px", border: "none", background: "transparent",
          color: "rgba(255, 255, 255, 0.40)", fontSize: 11, fontWeight: 700,
          letterSpacing: "0.08em", textTransform: "uppercase", cursor: "pointer",
        }}
        onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)"; }}
        onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255, 255, 255, 0.40)"; }}
      >
        <span>{label}</span>
        <motion.span animate={{ rotate: expanded ? 0 : -90 }} transition={{ duration: 0.18 }} style={{ display: "flex" }}>
          <ChevronDown size={12} strokeWidth={2.5} />
        </motion.span>
      </button>
      <AnimatePresence initial={false}>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
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
  const toggleSidebar    = useUIStore((s) => s.toggleSidebar);
  const macSimulated     = useUIStore((s) => s.macSimulated);
  const setSearchPaletteOpen = useUIStore((s) => s.setSearchPaletteOpen);
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

  const [spotifyOpen, setSpotifyOpen] = useState(true);
  const [libraryOpen, setLibraryOpen] = useState(true);
  const [pinsOpen,    setPinsOpen]    = useState(true);
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <motion.nav
      animate={{ width: isCollapsed ? 64 : 232 }}
      transition={{ duration: 0.22, ease: [0.23, 1, 0.32, 1] }}
      style={{
        width:         isCollapsed ? 64 : 232,
        flexShrink:    0,
        display:       "flex",
        flexDirection: "column",
        overflow:      "hidden",
        background:    "transparent",
        borderRight:   "none",
        paddingTop:    4,
      }}
    >
      {/* macOS traffic lights (native Mac OR Ctrl+Shift+M simulated on Windows) */}
      {(isMac || macSimulated) && (
        <div
          data-tauri-drag-region
          style={{
            height: 32,
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: isCollapsed ? "0 0 0 16px" : "0 14px",
            flexShrink: 0,
          }}
          title={!isMac ? "Mac traffic lights preview (Ctrl+Shift+M to toggle)" : undefined}
        >
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              background: "#ff5f57",
              boxShadow: "inset 0 0 0 1px rgba(0, 0, 0, 0.18)",
            }}
          />
          {!isCollapsed && (
            <>
              <div
                style={{
                  width: 12,
                  height: 12,
                  borderRadius: "50%",
                  background: "#febc2e",
                  boxShadow: "inset 0 0 0 1px rgba(0, 0, 0, 0.18)",
                }}
              />
              <div
                style={{
                  width: 12,
                  height: 12,
                  borderRadius: "50%",
                  background: "#28c840",
                  boxShadow: "inset 0 0 0 1px rgba(0, 0, 0, 0.18)",
                }}
              />
            </>
          )}
        </div>
      )}

      {/* header row: search or toggle */}
      <div style={{ height: 48, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: isCollapsed ? "center" : "space-between", padding: "0 8px" }}>
        {isCollapsed ? (
          <Tooltip label="Search (Ctrl+K)" side="right">
            <button
              onClick={() => setSearchPaletteOpen(true)}
              style={{
                width: 34, height: 34, borderRadius: 8,
                border: "1px solid rgba(255, 255, 255, 0.08)",
                background: "rgba(255, 255, 255, 0.04)",
                color: "var(--color-text-dim)",
                display: "flex", alignItems: "center", justifyContent: "center",
                cursor: "pointer",
                transition: "background 0.15s, color 0.15s",
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLButtonElement).style.background = "rgba(255, 255, 255, 0.08)";
                (e.currentTarget as HTMLButtonElement).style.color = "#ffffff";
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLButtonElement).style.background = "rgba(255, 255, 255, 0.04)";
                (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)";
              }}
            >
              <Search size={16} strokeWidth={2.2} />
            </button>
          </Tooltip>
        ) : (
          <>
            <div
              onClick={() => setSearchPaletteOpen(true)}
              style={{ ...glassPill, height: 32, cursor: "pointer", flex: 1, marginRight: 6 }}
            >
              <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
              <input
                ref={inputRef}
                value=""
                onFocus={() => setSearchPaletteOpen(true)}
                onClick={() => setSearchPaletteOpen(true)}
                readOnly
                placeholder="Search"
                style={{
                  flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none",
                  background: "transparent", color: "var(--color-text-hi)",
                  fontSize: 13.5, fontWeight: 400, fontFamily: "inherit",
                  cursor: "pointer",
                }}
              />
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 650,
                  color: "var(--color-text-dim)",
                  background: "rgba(255, 255, 255, 0.08)",
                  border: "1px solid rgba(255, 255, 255, 0.08)",
                  borderRadius: 4,
                  padding: "1px 5px",
                  lineHeight: "14px",
                  userSelect: "none",
                  flexShrink: 0,
                }}
              >
                {isMac ? "⌘K" : "Ctrl K"}
              </span>
            </div>
            <Tooltip label="Collapse sidebar" side="bottom">
              <button
                onClick={toggleSidebar}
                style={{
                  width: 32, height: 32, borderRadius: 8, border: "none",
                  background: "transparent", color: "var(--color-text-dim)",
                  display: "flex", alignItems: "center", justifyContent: "center",
                  cursor: "pointer", flexShrink: 0,
                }}
                onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)"; }}
                onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)"; }}
              >
                <PanelLeftClose size={16} strokeWidth={2} />
              </button>
            </Tooltip>
          </>
        )}
      </div>

      {/* nav */}
      <div style={{ flex: 1, overflowY: "auto", padding: isCollapsed ? "4px 6px" : "4px 8px" }}>
        <Section label="Discover" expanded={spotifyOpen} onToggle={() => setSpotifyOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={<Home      size={16} strokeWidth={2} />} label="Home"      active={path === "/"}                                          onClick={() => navigate("/")} collapsed={isCollapsed} />
          <NavItem icon={<ListMusic size={16} strokeWidth={2} />} label="Playlists" active={path === "/playlists" || onUnpinnedPlaylist}           onClick={() => navigate("/playlists")} collapsed={isCollapsed} />
        </Section>

        <Section label="Library" expanded={libraryOpen} onToggle={() => setLibraryOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={<Music size={16} strokeWidth={2} />} label="Songs"   active={onLibrary && tab === "songs"}   onClick={() => navigate("/library?tab=songs")} collapsed={isCollapsed} />
          <NavItem icon={<Disc3 size={16} strokeWidth={2} />} label="Albums"  active={onLibrary && tab === "albums"}  onClick={() => navigate("/library?tab=albums")} collapsed={isCollapsed} />
          <NavItem icon={<Users size={16} strokeWidth={2} />} label="Artists" active={onLibrary && tab === "artists"} onClick={() => navigate("/library?tab=artists")} collapsed={isCollapsed} />
        </Section>

        <Section label="Pins" expanded={pinsOpen} onToggle={() => setPinsOpen(v => !v)} collapsed={isCollapsed}>
          {pins.length === 0 ? (
            !isCollapsed ? (
              <div style={{ padding: "0 2px" }}>
                <div
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
                  whileTap={{ scale: 0.98 }}
                  whileHover={active ? {} : { backgroundColor: "var(--color-hover)" }}
                  transition={{ type: "spring", stiffness: 400, damping: 26 }}
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
                  onMouseEnter={() => {
                    if (p.type === "album") prefetchAlbum(qc, p.id);
                    else prefetchPlaylist(qc, p.id);
                  }}
                >
                  {active && (
                    <motion.div
                      layoutId="activeNavPill"
                      transition={{ type: "spring", stiffness: 500, damping: 36 }}
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
                      <img src={p.image_url} alt="" style={{ width: 26, height: 26, borderRadius: 5, objectFit: "cover" }} />
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
