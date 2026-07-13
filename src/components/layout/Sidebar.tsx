import { useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import {
  Home, ListMusic,
  Music, Disc3, Users,
  Pin, PinOff, Plus,
  Search, ChevronDown, X,
} from "lucide-react";
import { usePinsStore } from "../../store/pins.store";
import { useContextMenu } from "../ui/ContextMenu";
import { gpuLayer, zTransform } from "../../lib/motion";
import { isMac } from "../../lib/platform";
import { useQueryClient } from "@tanstack/react-query";
import { prefetchPlaylist, prefetchAlbum } from "../../lib/prefetch";

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
  icon, label, active, onClick,
}: {
  icon: React.ReactNode; label: string; active: boolean; onClick: () => void;
}) {
  const [hover, setHover] = useState(false);
  const bg = active ? "var(--color-active)" : hover ? "var(--color-hover)" : "transparent";
  return (
    <motion.button
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileTap={{ scale: 0.98 }}
      transition={{ type: "spring", stiffness: 400, damping: 26 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        display:       "flex",
        alignItems:    "center",
        gap:           11,
        height:        34,
        width:         "100%",
        padding:       "0 10px",
        borderRadius:  8,
        border:        "none",
        fontSize:      14,
        fontWeight:    active ? 600 : 500,
        color:         active ? "var(--color-text-hi)" : hover ? "var(--color-text-hi)" : "var(--color-text)",
        background:    bg,
        cursor:        "pointer",
        transition:    "background 0.12s, color 0.12s",
        textAlign:     "left",
      }}
    >
      <span style={{
        width: 20, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
        color: active ? "var(--color-accent)" : "inherit",
      }}>
        {icon}
      </span>
      <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{label}</span>
    </motion.button>
  );
}

// collapsible section

function Section({
  label, expanded, onToggle, children,
}: {
  label: string; expanded: boolean; onToggle: () => void; children: React.ReactNode;
}) {
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

  const [spotifyOpen, setSpotifyOpen] = useState(true);
  const [libraryOpen, setLibraryOpen] = useState(true);
  const [pinsOpen,    setPinsOpen]    = useState(true);
  const [query,       setQuery]       = useState(
    () => new URLSearchParams(location.search).get("q") ?? "",
  );

  function runSearch(value: string) {
    setQuery(value);
    navigate(value.trim() ? `/search?q=${encodeURIComponent(value)}` : "/search");
  }

  return (
    <nav
      style={{
        width:         232,
        flexShrink:    0,
        display:       "flex",
        flexDirection: "column",
        overflow:      "hidden",
        background:    "var(--color-sidebar)",
      }}
    >
      {/* Notes - macos/cider/vibrancy */}
      {isMac && <div data-tauri-drag-region style={{ height: 30, flexShrink: 0 }} />}

      {/* search - 48px row, centered -- a 32px pill in a 48px row leaves an 8px
          gap to the top, matching the 8px side padding (equal inset).titlebar
          is the same height so the ... < > icons line up at this level DONE */}
      <div style={{ height: 48, flexShrink: 0, display: "flex", alignItems: "center", padding: "0 8px" }}>
        <div style={{ ...glassPill, height: 32 }}>
          <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
          <input
            value={query}
            onChange={(e) => runSearch(e.target.value)}
            onFocus={() => { if (path !== "/search") navigate(query.trim() ? `/search?q=${encodeURIComponent(query)}` : "/search"); }}
            placeholder="Search"
            spellCheck={false}
            style={{
              flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none",
              background: "transparent", color: "var(--color-text-hi)",
              fontSize: 13.5, fontWeight: 400, fontFamily: "inherit",
            }}
          />
          {query && (
            <button
              onClick={() => runSearch("")}
              title="Clear"
              style={{ display: "flex", border: "none", background: "transparent", color: "var(--color-text-dim)", cursor: "pointer", padding: 0, flexShrink: 0 }}
            >
              <X size={13} strokeWidth={2.4} />
            </button>
          )}
        </div>
      </div>

      {/* nav */}
      <div style={{ flex: 1, overflowY: "auto", padding: "4px 8px" }}>
        <Section label="Spotify" expanded={spotifyOpen} onToggle={() => setSpotifyOpen(v => !v)}>
          <NavItem icon={<Home      size={17} strokeWidth={2} />} label="Home"      active={path === "/"}                                          onClick={() => navigate("/")} />
          <NavItem icon={<ListMusic size={16} strokeWidth={2} />} label="Playlists" active={path === "/playlists" || onUnpinnedPlaylist}           onClick={() => navigate("/playlists")} />
        </Section>

        <Section label="Library" expanded={libraryOpen} onToggle={() => setLibraryOpen(v => !v)}>
          <NavItem icon={<Music size={16} strokeWidth={2} />} label="Songs"   active={onLibrary && tab === "songs"}   onClick={() => navigate("/library?tab=songs")} />
          <NavItem icon={<Disc3 size={16} strokeWidth={2} />} label="Albums"  active={onLibrary && tab === "albums"}  onClick={() => navigate("/library?tab=albums")} />
          <NavItem icon={<Users size={16} strokeWidth={2} />} label="Artists" active={onLibrary && tab === "artists"} onClick={() => navigate("/library?tab=artists")} />
        </Section>

        <Section label="Pins" expanded={pinsOpen} onToggle={() => setPinsOpen(v => !v)}>
          {pins.length === 0 ? (
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
          ) : (
            pins.map((p) => {
              // this pin is the open page -> keep it lit (and don't reset on mouse leave). matches both pinned playlists and pinned albums
              const active = openId === p.id && openType === p.type;
              return (
              <button
                key={p.id}
                onClick={() => navigate(`/${p.type}/${p.id}`)}
                onContextMenu={openMenu([
                  { label: "Unpin", icon: <PinOff size={14} />, onSelect: () => removePin(p.id) },
                ])}
                title={p.name}
                style={{
                  display: "flex", alignItems: "center", gap: 10, height: 38, width: "100%",
                  padding: "0 8px", borderRadius: 8, border: "none",
                  background: active ? "var(--color-active)" : "transparent",
                  color: active ? "var(--color-text-hi)" : "var(--color-text)",
                  cursor: "pointer", textAlign: "left",
                  transition: "background 0.12s, color 0.12s",
                }}
                onMouseEnter={(e) => {
                  if (!active) (e.currentTarget as HTMLButtonElement).style.background = "var(--color-hover)";
                  // warm the pinned page so the click opens instantly (prefetching notes)
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
                <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: 13, fontWeight: active ? 600 : 500 }}>{p.name}</span>
              </button>
              );
            })
          )}
        </Section>

        <div style={{ padding: "2px 0" }}>
          <NavItem icon={<Plus size={16} strokeWidth={2.5} />} label="All Playlists" active={false} onClick={() => navigate("/playlists")} />
        </div>
      </div>
      {menuEl}
    </nav>
  );
}
