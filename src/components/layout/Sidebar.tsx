import { useState, useRef, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import {
  Home, ListMusic,
  Music, Disc3, Users,
  Pin, PinOff,
  Search, ChevronDown, X,
  PanelLeftClose, PanelLeft,
} from "lucide-react";
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
  height:       36,
  borderRadius: 8,
  background:   "var(--color-glass)",
  border:       "1px solid var(--color-glass-border)",
  display:      "flex",
  alignItems:   "center",
  gap:          9,
  padding:      "0 11px",
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
        display:       "flex",
        alignItems:    "center",
        justifyContent: collapsed ? "center" : "flex-start",
        gap:           collapsed ? 0 : 11,
        height:        34,
        width:         "100%",
        padding:       collapsed ? 0 : "0 10px",
        borderRadius:  8,
        border:        "none",
        fontSize:      14,
        fontWeight:    active ? 600 : 500,
        color:         active ? "var(--color-text-hi)" : "var(--color-text)",
        background:    active ? "var(--color-active)" : "transparent",
        cursor:        "pointer",
        transition:    "background 0.12s, color 0.12s",
        textAlign:     collapsed ? "center" : "left",
      }}
    >
      <span style={{
        width: 20, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
        color: active ? "var(--color-accent)" : "inherit",
      }}>
        {icon}
      </span>
      {!collapsed && (
        <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{label}</span>
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
          width: "100%", height: 30, padding: "0 10px", border: "none", background: "transparent",
          color: "var(--color-text-dim)", fontSize: 11, fontWeight: 700,
          letterSpacing: "0.06em", textTransform: "uppercase", cursor: "pointer",
        }}
        onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text)"; }}
        onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)"; }}
      >
        <span>{label}</span>
        <motion.span animate={{ rotate: expanded ? 0 : -90 }} transition={{ duration: 0.18 }} style={{ display: "flex" }}>
          <ChevronDown size={13} strokeWidth={2.5} />
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
  const [query,       setQuery]       = useState(
    () => new URLSearchParams(location.search).get("q") ?? "",
  );
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const q = new URLSearchParams(location.search).get("q") ?? "";
    setQuery(q);
  }, [location.search]);

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter" && query.trim()) {
      navigate(`/search?q=${encodeURIComponent(query.trim())}`);
    }
  }

  return (
    <nav
      style={{
        width:         isCollapsed ? 64 : 232,
        flexShrink:    0,
        display:       "flex",
        flexDirection: "column",
        overflow:      "hidden",
        background:    "var(--color-sidebar)",
        transition:    "width 0.22s cubic-bezier(0.23, 1, 0.32, 1)",
      }}
    >
      {/* Notes - macos/cider/vibrancy */}
      {isMac && <div data-tauri-drag-region style={{ height: 30, flexShrink: 0 }} />}

      {/* header row: search or toggle */}
      <div style={{ height: 48, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: isCollapsed ? "center" : "space-between", padding: "0 8px" }}>
        {isCollapsed ? (
          <Tooltip label={isNarrow ? "Search" : "Expand sidebar"} side="right">
            <button
              onClick={() => {
                if (!isNarrow) toggleSidebar();
                else navigate("/search");
              }}
              style={{
                width: 36, height: 34, borderRadius: 8, border: "none",
                background: "var(--color-glass)", color: "var(--color-text-dim)",
                display: "flex", alignItems: "center", justifyContent: "center",
                cursor: "pointer",
              }}
              onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)"; }}
              onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)"; }}
            >
              {isNarrow ? <Search size={15} strokeWidth={2.2} /> : <PanelLeft size={16} strokeWidth={2} />}
            </button>
          </Tooltip>
        ) : (
          <>
            <div
              onClick={() => inputRef.current?.focus()}
              style={{ ...glassPill, height: 32, cursor: "text", flex: 1, marginRight: 6 }}
            >
              <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
              <input
                ref={inputRef}
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Search"
                spellCheck={false}
                style={{
                  flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none",
                  background: "transparent", color: "var(--color-text-hi)",
                  fontSize: 13.5, fontWeight: 400, fontFamily: "inherit",
                }}
              />
              {query && (
                <Tooltip label="Clear search" side="top">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setQuery("");
                      inputRef.current?.focus();
                      if (path === "/search") navigate("/");
                    }}
                    style={{ display: "flex", border: "none", background: "transparent", color: "var(--color-text-dim)", cursor: "pointer", padding: 0, flexShrink: 0 }}
                  >
                    <X size={13} strokeWidth={2.4} />
                  </button>
                </Tooltip>
              )}
            </div>
            <Tooltip label="Collapse sidebar" side="bottom">
              <button
                onClick={toggleSidebar}
                style={{
                  width: 30, height: 30, borderRadius: 8, border: "none",
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
        <Section label="Spotify" expanded={spotifyOpen} onToggle={() => setSpotifyOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={<Home      size={17} strokeWidth={2} />} label="Home"      active={path === "/"}                                          onClick={() => navigate("/")} collapsed={isCollapsed} />
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
                <button
                  key={p.id}
                  onClick={() => navigate(`/${p.type}/${p.id}`)}
                  onContextMenu={openMenu([
                    { label: "Unpin", icon: <PinOff size={14} />, onSelect: () => removePin(p.id) },
                  ])}
                  title={isCollapsed ? undefined : p.name}
                  style={{
                    display: "flex", alignItems: "center", justifyContent: isCollapsed ? "center" : "flex-start",
                    gap: isCollapsed ? 0 : 10, height: 38, width: "100%",
                    padding: isCollapsed ? 0 : "0 8px", borderRadius: 8, border: "none",
                    background: active ? "var(--color-active)" : "transparent",
                    color: active ? "var(--color-text-hi)" : "var(--color-text)",
                    cursor: "pointer", textAlign: isCollapsed ? "center" : "left",
                    transition: "background 0.12s, color 0.12s",
                  }}
                  onMouseEnter={(e) => {
                    if (!active) (e.currentTarget as HTMLButtonElement).style.background = "var(--color-hover)";
                    if (p.type === "album") prefetchAlbum(qc, p.id);
                    else prefetchPlaylist(qc, p.id);
                  }}
                  onMouseLeave={(e) => { if (!active) (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
                >
                  {p.image_url ? (
                    <img src={p.image_url} alt="" style={{ width: 28, height: 28, borderRadius: 5, objectFit: "cover", flexShrink: 0 }} />
                  ) : (
                    <div style={{ width: 28, height: 28, borderRadius: 5, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
                      <ListMusic size={14} style={{ color: active ? "var(--color-accent)" : "var(--color-text-dim)" }} />
                    </div>
                  )}
                  {!isCollapsed && (
                    <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: 13, fontWeight: active ? 600 : 500 }}>{p.name}</span>
                  )}
                </button>
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
    </nav>
  );
}
