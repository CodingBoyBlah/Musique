import { useEffect, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { TitleBar, WindowCaptionControls } from "./TitleBar";
import Sidebar from "./Sidebar";
import { PlayerBar } from "./PlayerBar";
import { usePositionTicker } from "../../hooks/usePositionTicker";
import { QueuePanel } from "./QueuePanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm } from "../ui/QuitConfirm";
import { Toaster } from "../ui/Toaster";
import { Immersive } from "./Immersive";
import { AddToPlaylistModal } from "../ui/AddToPlaylistModal";
import { CreditsModal } from "../ui/CreditsModal";
import { YtMatchModal } from "../ui/YtMatchModal";
import { DevicesPopover } from "./DevicesPopover";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";
import { usePrefsStore } from "../../store/prefs.store";
import { chromePx } from "../../lib/zoom";
import { EASE_OUT } from "../../lib/motion";
import "../../styles/layout.css";

/* collapsed sidebar. on mac the native traffic lights live in this column at
   their fixed OS positions (12px dots, 20px pitch, first centre at x=20), so
   the collapsed rail has to stay wide enough to hold all three. */
const COLLAPSED_SIDEBAR_W = 64;
const MAC_COLLAPSED_SIDEBAR_W = 72;

export default function Layout() {
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const immersiveOpen = usePlayerStore((s) => s.immersiveOpen);
  // one owner for the playhead clock, whatever else is on screen
  usePositionTicker();
  const effect = useUIStore((s) => s.windowEffect);
  const materialTransparency = useUIStore((s) => s.materialTransparency);
  const pageTint = useUIStore((s) => s.pageTint);
  const backdropActive = useUIStore((s) => s.backdropActive);
  const setBackdropActive = useUIStore((s) => s.setBackdropActive);
  const location = useLocation();
  const reduceMotion = useReducedMotion();
  const mainRef = useRef<HTMLElement>(null);

  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const macSimulated = useUIStore((s) => s.macSimulated);
  const uiZoom = usePrefsStore((s) => s.uiZoom);
  // same sum as the Sidebar's rail: the traffic lights' room in physical px
  const collapsedSidebarW = (isMac || macSimulated)
    ? Math.max(COLLAPSED_SIDEBAR_W, chromePx(MAC_COLLAPSED_SIDEBAR_W, uiZoom))
    : COLLAPSED_SIDEBAR_W;
  const [willCrushMain, setWillCrushMain] = useState(false);
  const [shellHidden, setShellHidden] = useState(false);
  const immersiveCovered = useUIStore((s) => s.immersiveCovered);
  const setImmersiveCovered = useUIStore((s) => s.setImmersiveCovered);

  /* Hide (and inert) the shell once the immersive overlay fully covers it.
     The signal is the overlay's own animation completing (Immersive sets
     immersiveCovered), so the two can never drift apart. The timer is only a
     backstop in case that signal never arrives - it is well past the 260ms
     fade, so it never races the real one. */
  useEffect(() => {
    if (!immersiveOpen) {
      setShellHidden(false);
      if (immersiveCovered) setImmersiveCovered(false);
      return;
    }
    if (immersiveCovered) {
      setShellHidden(true);
      return;
    }
    const t = setTimeout(() => setShellHidden(true), 700);
    return () => clearTimeout(t);
  }, [immersiveOpen, immersiveCovered, setImmersiveCovered]);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const checkCrush = () => {
      const w = window.innerWidth;
      const isCollapsed = sidebarCollapsed || w < 768;
      const sw = isCollapsed ? collapsedSidebarW : 232;
      const rpw = lyricsOpen ? 366 : queueOpen ? 272 : 0;
      const nextCrush = w - sw - rpw < 340;
      setWillCrushMain((prev) => (prev === nextCrush ? prev : nextCrush));
    };

    checkCrush();

    let rId = 0;
    const onResizeThrottled = () => {
      if (rId) return;
      rId = requestAnimationFrame(() => {
        rId = 0;
        checkCrush();
      });
    };
    window.addEventListener("resize", onResizeThrottled, { passive: true });

    return () => {
      window.removeEventListener("resize", onResizeThrottled);
      if (rId) cancelAnimationFrame(rId);
    };
  }, [sidebarCollapsed, collapsedSidebarW, lyricsOpen, queueOpen]);

  const rawPanelWidth = lyricsOpen ? 366 : queueOpen ? 272 : 0;
  const spacerWidth = willCrushMain ? 0 : rawPanelWidth;

  /* Right rail width, held through a close.

     The rail used to drop to display:none on the same frame the panel was told
     to close, so its slide-out never showed and the page card snapped wider.
     Now a closing panel keeps its rail until AnimatePresence reports the exit
     complete; only then does the rail give its width back. That moment bumps
     railSettleTick in the UI store, which the grid cards subscribe to through
     useReflowPulse - so they re-render (and framer re-measures them) on the
     very render the column widens, and glide into the space.

     Refs rather than state: the hold has to be decided during the render that
     closes the panel, or there is one painted frame with no rail at all. */
  const railSettleTick = useUIStore((s) => s.railSettleTick);
  const bumpRailSettle = useUIStore((s) => s.bumpRailSettle);
  const lastRailW = useRef(spacerWidth);
  const holdTick = useRef<number | null>(null);
  if (rawPanelWidth > 0) {
    if (spacerWidth > 0) lastRailW.current = spacerWidth;
    holdTick.current = null;
  } else if (holdTick.current === null && lastRailW.current > 0) {
    holdTick.current = railSettleTick;
  }
  const holdingRail = rawPanelWidth === 0 && holdTick.current === railSettleTick;
  const railWidth = willCrushMain
    ? 0
    : rawPanelWidth > 0
    ? spacerWidth
    : holdingRail
    ? lastRailW.current
    : 0;
  if (rawPanelWidth === 0 && !holdingRail) lastRailW.current = 0;

  const onPanelExitComplete = () => {
    const s = usePlayerStore.getState();
    if (!s.lyricsOpen && !s.queueOpen) bumpRailSettle();
  };

  /* `trim_memory` used to fire 1.5s after every navigation. That call empties
     the working set of this process and of every WebView2 child, so it landed
     right as the user was reading and scrolling the page they had just opened,
     and everything had to be faulted back in. Genuine idle trimming now happens
     in the native layer, only once the window has actually been out of sight
     for a while (see mem_trim.rs). */
  useEffect(() => {
    if (mainRef.current) mainRef.current.scrollTop = 0;
  }, [location.pathname]);

  useEffect(() => {
    getBackdropActive().then(setBackdropActive).catch(() => setBackdropActive(false));
  }, [setBackdropActive]);

  const toggleMacSimulated = useUIStore((s) => s.toggleMacSimulated);

  // Ctrl+K lives with the search field in the top bar (TopSearch)
  useEffect(() => {
    function onGlobalKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "m") {
        e.preventDefault();
        toggleMacSimulated();
      }
    }
    window.addEventListener("keydown", onGlobalKey);
    return () => window.removeEventListener("keydown", onGlobalKey);
  }, [toggleMacSimulated]);

  // backdrop strategy:
  //  - no live material (Linux, or Mica/vibrancy failed) -> paint the solid app
  //    bg, otherwise the transparent window shows white (or the desktop)
  //  - windows acrylic -> OS ignores the tint, so darken with a CSS scrim
  //  - windows Mica / macOS vibrancy -> stay transparent, OS material shows
  const scrim = backdropScrim(backdropActive, effect, materialTransparency, isMac);
  const isTranslucent = backdropActive && effect !== "none";
  const cardBg = isTranslucent
    ? `rgba(18, 18, 20, ${Math.max(0.66, Math.min(0.85, 0.78 * (1 - materialTransparency * 0.28)))})`
    : "rgba(19, 19, 19, 0.94)";

  return (
    /*
     * window root is fully transparent - the OS Mica material (applied in rust)
     * is the background. no backdrop-filter here, that'd blur the desktop a
     * second time and fight Mica. win11 rounds the frame.
     */
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        overflow: "hidden",
        background: scrim,
        transition: "background 0.2s",
      }}
    >
      <div
        style={{
          display: "flex",
          flex: 1,
          minHeight: 0,
          overflow: "hidden",
          position: "relative",
          visibility: shellHidden ? "hidden" : "visible",
        }}
        inert={shellHidden ? true : undefined}
      >
        {/* the sidebar runs the full height of the window */}
        <Sidebar />

        {/* everything right of the sidebar: the top bar on the OS material,
            and under it the page card plus the lyrics / queue rail */}
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
          <div style={{ flexShrink: 0, position: "relative", zIndex: 20 }}>
            <TitleBar />
          </div>

          <div style={{ flex: 1, minHeight: 0, display: "flex", position: "relative" }}>
            {/* The Main Window Island Card */}
            <div
              style={{
                flex: 1,
                minWidth: 0,
                margin: "0 4px 4px",
                borderRadius: 12,
                border: "1px solid rgba(255, 255, 255, 0.08)",
                background: "transparent",
                /* negative spread keeps the lift underneath the card instead of
                   letting it bleed up into the top bar */
                boxShadow: "0 14px 30px -8px rgba(0, 0, 0, 0.55)",
                position: "relative",
                overflow: "hidden",
                display: "flex",
                flexDirection: "column",
              }}
            >
              {/* the card's material, on its own layer under the content.

                  NO backdrop-filter here, deliberately. nothing is ever painted
                  between the window root and this card, so it would only blur
                  transparency (or, with no live material, a flat scrim colour)
                  while costing a compositing surface of its own. */}
              <div
                aria-hidden
                style={{
                  position: "absolute",
                  inset: 0,
                  zIndex: 0,
                  pointerEvents: "none",
                  borderRadius: 12,
                  background: cardBg,
                  boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.08)",
                }}
              />

              {/* Cover art bloom contained within the island card */}
              <AnimatePresence>
                {pageTint && (
                  <motion.div
                    key={pageTint}
                    aria-hidden
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 0.18 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.7, ease: "easeOut" }}
                    style={{
                      position: "absolute",
                      inset: 0,
                      zIndex: 0,
                      pointerEvents: "none",
                      overflow: "hidden",
                      borderRadius: 12,
                    }}
                  >
                    <div
                      style={{
                        position: "absolute",
                        inset: 0,
                        backgroundImage: `url(${pageTint})`,
                        backgroundSize: "cover",
                        backgroundPosition: "center top",
                        filter: "blur(72px) saturate(1.7)",
                        transform: "scale(1.6)",
                        transformOrigin: "center top",
                        maskImage:
                          "radial-gradient(75% 70% at 50% 0%, #000 0%, rgba(0,0,0,0.5) 42%, transparent 78%)",
                        WebkitMaskImage:
                          "radial-gradient(75% 70% at 50% 0%, #000 0%, rgba(0,0,0,0.5) 42%, transparent 78%)",
                      }}
                    />
                    <div
                      style={{
                        position: "absolute",
                        inset: 0,
                        opacity: 0.05,
                        mixBlendMode: "overlay",
                        backgroundRepeat: "repeat",
                        backgroundImage:
                          "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E\")",
                      }}
                    />
                  </motion.div>
                )}
              </AnimatePresence>

              {/* Scrolling page view inside card */}
              <div style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden" }}>
                <main
                  ref={mainRef}
                  style={{
                    position: "absolute",
                    inset: 0,
                    overflowY: "auto",
                    overflowX: "hidden",
                    paddingTop: 20,
                    paddingLeft: "clamp(14px, 2.5vw, 32px)",
                    paddingRight: "clamp(14px, 2.5vw, 32px)",
                    paddingBottom: "90px",
                  }}
                >
                  <motion.div
                    key={location.pathname}
                    initial={reduceMotion ? false : { opacity: 0, y: 6, scale: 0.995 }}
                    animate={{ opacity: 1, y: 0, scale: 1 }}
                    transition={{ duration: 0.18, ease: EASE_OUT }}
                  >
                    <Outlet />
                  </motion.div>
                </main>
              </div>

              {/* PlayerBar docked inside card. The immersive view floats its own
                  copy over the top, so this one stands down rather than render and
                  paint underneath an opaque overlay. */}
              {!immersiveOpen && <PlayerBar />}
            </div>

            {/* Right rail - Lyrics or Queue on base layer */}
            <div
              style={{
                width: railWidth,
                flexShrink: 0,
                position: "relative",
                overflow: "hidden",
                height: "100%",
                display: railWidth > 0 ? "flex" : "none",
              }}
            >
              <AnimatePresence initial={false} onExitComplete={onPanelExitComplete}>
                {lyricsOpen && <LyricsPanel key="lyrics" />}
              </AnimatePresence>
              <AnimatePresence initial={false} onExitComplete={onPanelExitComplete}>
                {queueOpen && <QueuePanel key="queue" />}
              </AnimatePresence>
            </div>
          </div>
        </div>

      </div>

      <DevicesPopover />
      <Immersive />
      <QuitConfirm />
      <AddToPlaylistModal />
      <CreditsModal />
      <YtMatchModal />
      <Toaster />
      {/* always at the window's own top-right corner, in the top bar's strip,
          so the buttons sit on the OS material and the close button owns the
          corner pixel */}
      <WindowCaptionControls />
    </div>
  );
}
