import { useState, useEffect, useLayoutEffect, useRef, useCallback } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence, animate, useMotionValue, useTransform, useReducedMotion, type MotionValue } from "framer-motion";
import {
  Home, ListMusic,
  Music, Disc3, User, Mic, BarChart,
  Pin, PinOff,
  ChevronDown, Folder,
  type LucideIcon,
} from "@/lib/icons";
import { usePinsStore, type PinnedItem } from "../../store/pins.store";
import { useMyPlaylists } from "../../hooks/useLibrary";
import { usePlaylistFolders } from "../../hooks/usePlaylistFolders";
import { flattenRows, type SidebarRow } from "../../lib/rootlist";
import type { RootItem } from "../../api/internal";
import { useUIStore } from "../../store/ui.store";
import { useContextMenu } from "../ui/ContextMenu";
import { gpuLayer, zTransform, EASE_OUT, SPRING, PRESS_TRANSITION } from "../../lib/motion";
import { useQueryClient } from "@tanstack/react-query";
import { prefetchPlaylist, prefetchAlbum } from "../../lib/prefetch";
import { Tooltip } from "../ui/Tooltip";
import { OverlayScrollbar } from "../ui/OverlayScrollbar";
import { isMac } from "../../lib/platform";
import { usePrefsStore } from "../../store/prefs.store";
import { chromePx } from "../../lib/zoom";

/* collapsed rail geometry: one 40px square target per row, centred in the
rail, 10px corners. Every icon and pinned cover sits on this same grid, so the
rail reads as one column rather than a stack of odd-sized pieces. */
const RAIL_ITEM = 40;
// the gap between the rail and the page card (the card's own left margin)
const GUTTER = 4;
const RAIL_RADIUS = 10;
// a pinned cover inset in its target, so the selected pill shows as an even
// 4px ring around it; its corners follow the pill's (10 - 4 = 6)
const RAIL_COVER = 32;

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
        gap:           collapsed ? 0 : 10,
        height:        collapsed ? RAIL_ITEM : 34,
        /* collapsed: a fixed square, centred in the column, so the active
         pill (inset: 0) is a perfect square */
        width:         collapsed ? RAIL_ITEM : "100%",
        margin:        collapsed ? "0 auto" : undefined,
        padding:       collapsed ? 0 : "0 8px",
        borderRadius:  collapsed ? RAIL_RADIUS : 8,
        border:        "none",
        fontSize:      13.5,
        fontWeight:    active ? 600 : 500,
        color:         active ? "#ffffff" : "var(--color-text)",
        background:    "transparent",
        cursor:        "pointer",
        textAlign:     collapsed ? "center" : "left",
      }}
    >
      {active && !collapsed && (
        <motion.div
          layoutId="activeNavPill"
          transition={SPRING}
          style={{
            position: "absolute",
            inset: 0,
            borderRadius: collapsed ? RAIL_RADIUS : 8,
            background: "var(--color-accent)",
            boxShadow: "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))",
            zIndex: 0,
          }}
        />
      )}
      <span className="nav-icon" data-active={active || undefined} style={{
        position: "relative",
        zIndex: 1,
        /* a 26px slot, the same as a playlist cover: icons are centred on the
         covers' centre line and every label starts on one text edge */
        width: collapsed ? 20 : 26, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
      }}>
        {/* a touch larger alone in the rail, where the icon is the only label */}
        <Icon size={collapsed ? 19 : 18} strokeWidth={1.7} active={active} />
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

/* The collapsed rail's selection highlight: one element for the whole rail
rather than a pill inside each item handed over by layoutId, which slid as a
rigid tile over every icon in between.

Its two edges ride separate critically damped springs. The leading edge (the
one on the side of the item you picked) is quick, the trailing edge follows a
beat later, so the highlight stretches toward your choice and draws itself in
behind - the motion points at where it's going. No overshoot: this answers a
click, and nothing that was merely clicked should bounce. */
const EDGE_LEAD  = { type: "spring" as const, stiffness: 620, damping: 50 };  // ~critical (2*sqrt(620) = 49.8)
const EDGE_TRAIL = { type: "spring" as const, stiffness: 260, damping: 33 };  // ~critical (2*sqrt(260) = 32.2)

function useRailIndicator(containerRef: React.RefObject<HTMLDivElement | null>, enabled: boolean, deps: unknown[]) {
  const reduceMotion = useReducedMotion();
  const top = useMotionValue(0);
  const bottom = useMotionValue(0);
  const height = useTransform([top, bottom], ([t, b]: number[]) => Math.max(0, b - t));
  const [visible, setVisible] = useState(false);
  const shown = useRef(false);

  const measure = useCallback(() => {
    const c = containerRef.current;
    const el = enabled ? c?.querySelector<HTMLElement>('.sb-item[data-active="true"]') : null;
    if (!c || !el) {
      shown.current = false;
      setVisible(false);
      return;
    }
    // measure from the item's centre and untransformed height, so a press
    // scale still running on the clicked item doesn't skew the target
    const cr = c.getBoundingClientRect();
    const r = el.getBoundingClientRect();
    const mid = r.top + r.height / 2 - cr.top + c.scrollTop;
    const t = mid - el.offsetHeight / 2;
    const b = mid + el.offsetHeight / 2;

    // first appearance (rail just collapsed, or arriving from a page with no
    // rail item) lands in place and fades in; only item-to-item moves travel
    if (!shown.current || reduceMotion) {
      top.jump(t);
      bottom.jump(b);
      shown.current = true;
      setVisible(true);
      return;
    }
    if (t === top.get() && b === bottom.get()) return;
    const down = t > top.get();
    animate(top, t, down ? EDGE_TRAIL : EDGE_LEAD);
    animate(bottom, b, down ? EDGE_LEAD : EDGE_TRAIL);
  }, [containerRef, enabled, reduceMotion, top, bottom]);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  useLayoutEffect(measure, deps);

  // zoom or window changes move the items without a route change
  useEffect(() => {
    const c = containerRef.current;
    if (!c || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => measure());
    ro.observe(c);
    return () => ro.disconnect();
  }, [containerRef, measure]);

  return { top, height, visible };
}

function RailIndicator({ top, height, visible }: { top: MotionValue<number>; height: MotionValue<number>; visible: boolean }) {
  return (
    <motion.div
      aria-hidden
      initial={false}
      animate={{ opacity: visible ? 1 : 0 }}
      transition={{ duration: 0.15, ease: EASE_OUT }}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        right: 0,
        margin: "0 auto",
        width: RAIL_ITEM,
        y: top,
        height,
        borderRadius: RAIL_RADIUS,
        background: "var(--color-accent)",
        boxShadow: "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))",
        pointerEvents: "none",
        zIndex: 0,
      }}
    />
  );
}

// collapsible section

function Section({
  label, expanded, onToggle, children, collapsed, first,
}: {
  label: string; expanded: boolean; onToggle: () => void; children: React.ReactNode; collapsed?: boolean;
  // the top group: nothing above it to separate from, so no divider
  first?: boolean;
}) {
  if (collapsed) {
    /* groups are split by a short centred hairline, not a rule across the
       whole rail: it separates without boxing the column into strips. an
       empty group (no pins yet) draws nothing, divider included. */
    const hasItems = Array.isArray(children) ? children.some(Boolean) : Boolean(children);
    if (!hasItems) return null;
    return (
      <div role="group" aria-label={label} style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {!first && (
          <div
            aria-hidden
            style={{ width: 20, height: 1, borderRadius: 1, background: "rgba(255, 255, 255, 0.12)", margin: "8px auto" }}
          />
        )}
        {children}
      </div>
    );
  }

  /* section headers read as quiet labels: small capitals, a lighter weight
     than the old 700, set a step dimmer than the rows so the rows lead. The
     chevron only shows when it has something to say - on hover or keyboard
     focus, or while the section is folded so a hidden group is never lost.
     Groups after the first get air above them instead of a rule. */
  return (
    <div style={{ marginTop: first ? 0 : 10 }}>
      <button
        onClick={onToggle}
        aria-expanded={expanded}
        className="sb-section-head focus-ring"
        style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          // 8px in like every row, so a label starts on the edge the icons and
          // covers below it start on
          width: "100%", height: 28, padding: "0 8px", border: "none", background: "transparent",
          borderRadius: 6,
          font: "inherit",
          // capitals at 11px want open tracking or they clump into a block
          color: "rgba(255, 255, 255, 0.5)", fontSize: 11, fontWeight: 600,
          letterSpacing: "0.06em", textTransform: "uppercase", cursor: "pointer",
        }}
      >
        <span>{label}</span>
        <motion.span
          className="sb-chev"
          animate={{ rotate: expanded ? 0 : -90 }}
          transition={{ duration: 0.18, ease: EASE_OUT }}
          style={{ display: "flex" }}
        >
          <ChevronDown size={12} strokeWidth={2.4} />
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
  const sidebarMode = usePrefsStore((s) => s.sidebarMode);
  const showStats   = usePrefsStore((s) => s.showStats);
  const { data: myPlaylists = [], isLoading: playlistsLoading } = useMyPlaylists();
  const qc          = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const path = location.pathname;
  const tab  = new URLSearchParams(location.search).get("tab") ?? "songs";
  const onLibrary = path === "/library";

  /* the last section lists either your pins or every playlist you have
   (Settings -> Sidebar). both are the same kind of row, so one list drives it */
  const showAllPlaylists = sidebarMode === "playlists";
  /* with the rootlist, "all playlists" follows spotify's own order and folder
   nesting (folders collapse; the collapsed rail just keeps the order) */
  const { data: rootTree } = usePlaylistFolders();
  const [closedFolders, setClosedFolders] = useState<Set<string>>(() => {
    try {
      return new Set(JSON.parse(localStorage.getItem("sidebar-closed-folders") ?? "[]") as string[]);
    } catch {
      return new Set();
    }
  });
  const toggleFolder = useCallback((id: string) => {
    setClosedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      try { localStorage.setItem("sidebar-closed-folders", JSON.stringify([...next])); } catch { /* private mode */ }
      return next;
    });
  }, []);
  const syncedById = new Map(myPlaylists.map((pl) => [pl.id, pl]));
  const rootNames = new Map<string, { name: string | null; image_url: string | null }>();
  (function walk(items: RootItem[]) {
    for (const it of items) {
      if (it.kind === "playlist") rootNames.set(it.id, { name: it.name, image_url: it.image_url });
      else walk(it.children);
    }
  })(rootTree ?? []);
  const useTree = showAllPlaylists && !!rootTree && rootTree.length > 0;
  // every playlist (what "is the open page in the sidebar" checks against),
  // and the rows actually shown with closed folders folded away
  const allRows: SidebarRow[] = useTree ? flattenRows(rootTree!, new Set()) : [];
  const openRows: SidebarRow[] = useTree ? flattenRows(rootTree!, closedFolders) : [];
  const asEntry = (id: string): PinnedItem => {
    const pl = syncedById.get(id);
    const meta = rootNames.get(id);
    return { id, name: pl?.name ?? meta?.name ?? "Playlist", image_url: pl?.image_url ?? meta?.image_url ?? null, type: "playlist" };
  };
  const entries: PinnedItem[] = useTree
    ? allRows.filter((r) => r.kind === "playlist").map((r) => asEntry(r.id))
    : showAllPlaylists
    ? myPlaylists.map((pl) => ({ id: pl.id, name: pl.name, image_url: pl.image_url, type: "playlist" as const }))
    : pins;
  const depthOf = new Map(allRows.filter((r) => r.kind === "playlist").map((r) => [r.id, r.depth]));

  /* which library item (if any) is open + is it in that list. lets the
   sidebar light up the specific row when it's open, and only fall back to
   lighting the "Playlists" button for playlists that have no row */

  const openMatch      = path.match(/^\/(playlist|album)\/(.+)$/);
  const openType       = openMatch?.[1] ?? null;   // "playlist" | "album"
  const openId         = openMatch?.[2] ?? null;
  const openInSidebar  = openId != null && entries.some((p) => p.id === openId && p.type === openType);
  const onPlaylistWithoutRow = openType === "playlist" && !openInSidebar;

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
  // the lights don't scale with the app's zoom, so their room is sized in
  // physical pixels (chromePx), but never below the normal 64 CSS px rail
  // that the nav icons need when zoomed in
  const zoom = usePrefsStore((s) => s.uiZoom);
  const px = (n: number) => chromePx(n, zoom);
  const railWidth = isCollapsed ? (macChrome ? Math.max(64, px(72)) : 64) : 232;

  const railRef = useRef<HTMLDivElement>(null);
  const rail = useRailIndicator(railRef, isCollapsed, [isCollapsed, path, location.search, entries.length, sidebarMode]);

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
        /* clips like overflow:hidden, but 4px wider on the right so the
           scrollbar can sit in the gap and touch the page card */
        clipPath:      `inset(0 -${GUTTER}px 0 0)`,
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
          style={{ height: px(32), display: "flex", alignItems: "center", gap: px(8), padding: `0 ${px(14)}px`, flexShrink: 0 }}
          title={!isMac ? "Mac traffic lights preview (Ctrl+Shift+M to toggle)" : undefined}
        >
          {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
            <div
              key={c}
              style={{
                width: px(12), height: px(12), flexShrink: 0, borderRadius: "50%",
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
      {/* the list scrolls under a floating overlay scrollbar rather than a
          native one: a long playlist list no longer gives up a 10px strip
          of the rail's width to a gutter. the right inset is 4px, not 8:
          the page card's own 4px margin makes up the rest, so rows reach
          the same distance from the card as from the window's left edge */}
      <div className="ovs-host" style={{ flex: 1, minHeight: 0, display: "flex", flexDirection: "column" }}>
      <div ref={railRef} className="ovs-hide" style={{ position: "relative", flex: 1, minHeight: 0, overflowY: "auto", overflowX: "hidden", padding: isCollapsed ? "6px 6px 12px" : "4px 4px 4px 8px" }}>
        {isCollapsed && <RailIndicator top={rail.top} height={rail.height} visible={rail.visible} />}
        <Section label="Discover" first expanded={spotifyOpen} onToggle={() => setSpotifyOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={Home} label="Home"      active={path === "/"}                                          onClick={() => navigate("/")} collapsed={isCollapsed} />
          <NavItem icon={ListMusic} label="Playlists" active={path === "/playlists" || onPlaylistWithoutRow}           onClick={() => navigate("/playlists")} collapsed={isCollapsed} />
          {showStats && <NavItem icon={BarChart} label="Stats" active={path === "/stats"} onClick={() => navigate("/stats")} collapsed={isCollapsed} />}
        </Section>

        <Section label="Library" expanded={libraryOpen} onToggle={() => setLibraryOpen(v => !v)} collapsed={isCollapsed}>
          <NavItem icon={Music} label="Songs"   active={onLibrary && tab === "songs"}   onClick={() => navigate("/library?tab=songs")} collapsed={isCollapsed} />
          <NavItem icon={Disc3} label="Albums"  active={onLibrary && tab === "albums"}  onClick={() => navigate("/library?tab=albums")} collapsed={isCollapsed} />
          <NavItem icon={User} label="Artists" active={onLibrary && tab === "artists"} onClick={() => navigate("/library?tab=artists")} collapsed={isCollapsed} />
          <NavItem icon={Mic} label="Podcasts" active={(onLibrary && tab === "podcasts") || path.startsWith("/show/")} onClick={() => navigate("/library?tab=podcasts")} collapsed={isCollapsed} />
        </Section>

        <Section label={showAllPlaylists ? "Your playlists" : "Pins"} expanded={pinsOpen} onToggle={() => setPinsOpen(v => !v)} collapsed={isCollapsed}>
          {entries.length === 0 ? (
            // nothing to say while the library is still loading
            !isCollapsed && !(showAllPlaylists && playlistsLoading) ? (
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
                  {showAllPlaylists ? (
                    <>
                      <ListMusic size={13} strokeWidth={2} style={{ flexShrink: 0, marginTop: 1 }} />
                      <span>No playlists yet. Ones you make or follow show up here.</span>
                    </>
                  ) : (
                    <>
                      <Pin size={13} strokeWidth={2} style={{ flexShrink: 0, marginTop: 1 }} />
                      <span>No pins yet. Right-click a playlist to pin it.</span>
                    </>
                  )}
                </div>
              </div>
            ) : null
          ) : (
            (useTree
              ? (isCollapsed ? allRows.filter((r) => r.kind === "playlist") : openRows).map((r) => (r.kind === "folder" ? r : asEntry(r.id)))
              : entries
            ).map((row) => {
              if ("kind" in row && row.kind === "folder") {
                const fIndent = row.depth * 14;
                return (
                  <button
                    key={`folder-${row.id}`}
                    type="button"
                    className="sb-item focus-ring"
                    onClick={() => toggleFolder(row.id)}
                    onDoubleClick={() => navigate(`/playlists?folder=${encodeURIComponent(row.id)}`)}
                    aria-expanded={row.open}
                    title={`${row.name} · ${row.count} playlists (double-click to open)`}
                    style={{
                      position: "relative", display: "flex", alignItems: "center", gap: 10, height: 34,
                      width: `calc(100% - ${fIndent}px)`, marginLeft: fIndent,
                      padding: "0 8px", border: "none", background: "transparent",
                      borderRadius: 8, cursor: "pointer", color: "var(--color-text)", fontSize: 13, fontWeight: 600, textAlign: "left",
                    }}
                  >
                    {row.depth > 0 && <span aria-hidden className="sb-guide" />}
                    <span style={{ width: 26, height: 26, borderRadius: 5, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "var(--color-surface-2)", color: "var(--color-text-dim)" }}>
                      <Folder size={14} strokeWidth={1.9} />
                    </span>
                    <span style={{ flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{row.name}</span>
                    <span className="tnum" style={{ fontSize: 11, fontWeight: 600, color: "var(--color-text-dim)" }}>{row.count}</span>
                    <ChevronDown size={13} style={{ flexShrink: 0, color: "var(--color-text-dim)", transform: row.open ? "none" : "rotate(-90deg)", transition: "transform 0.18s ease" }} />
                  </button>
                );
              }
              const p = row as PinnedItem;
              const indent = isCollapsed ? 0 : (depthOf.get(p.id) ?? 0) * 14;
              const active = openId === p.id && openType === p.type;
              const coverSize = isCollapsed ? RAIL_COVER : 26;
              const coverRadius = isCollapsed ? RAIL_RADIUS - (RAIL_ITEM - RAIL_COVER) / 2 : 5;
              const btn = (
                <motion.button
                  key={p.id}
                  onClick={() => navigate(`/${p.type}/${p.id}`)}
                  // pinning means nothing while the sidebar lists every playlist,
                  // so those rows have no menu; a pin row can be unpinned
                  onContextMenu={showAllPlaylists ? undefined : openMenu([
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
                    height:        isCollapsed ? RAIL_ITEM : 34,
                    width:         isCollapsed ? RAIL_ITEM : `calc(100% - ${indent}px)`,
                    margin:        isCollapsed ? "0 auto" : `0 0 0 ${indent}px`,
                    padding:       isCollapsed ? 0 : "0 8px",
                    borderRadius:  isCollapsed ? RAIL_RADIUS : 8,
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
                  {indent > 0 && <span aria-hidden className="sb-guide" />}
                  {active && !isCollapsed && (
                    <motion.div
                      layoutId="activeNavPill"
                      transition={SPRING}
                      style={{
                        position: "absolute",
                        inset: 0,
                        borderRadius: isCollapsed ? RAIL_RADIUS : 8,
                        background: "var(--color-accent)",
                        boxShadow: "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))",
                        zIndex: 0,
                      }}
                    />
                  )}
                  <span style={{ position: "relative", zIndex: 1, display: "flex", flexShrink: 0 }}>
                    {p.image_url ? (
                      <img src={coverUrl(p.image_url, coverSize) ?? p.image_url} alt="" style={{ width: coverSize, height: coverSize, borderRadius: coverRadius, objectFit: "cover", boxShadow: isCollapsed ? "0 2px 6px rgba(0, 0, 0, 0.35)" : undefined }} />
                    ) : (
                      <div style={{ width: coverSize, height: coverSize, borderRadius: coverRadius, background: active ? "rgba(255,255,255,0.2)" : "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center" }}>
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
      <OverlayScrollbar target={railRef} edge={-GUTTER} />
      </div>
      {menuEl}
    </motion.nav>
  );
}
