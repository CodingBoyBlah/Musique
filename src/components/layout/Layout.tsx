import { useEffect, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { TitleBar, WindowCaptionControls, CAPTION_W, CAPTION_H } from "./TitleBar";
import { SearchPalette } from "./SearchPalette";
import Sidebar from "./Sidebar";
import { PlayerBar } from "./PlayerBar";
import { usePositionTicker } from "../../hooks/usePositionTicker";
import { QueuePanel } from "./QueuePanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm } from "../ui/QuitConfirm";
import { Toaster } from "../ui/Toaster";
import { Immersive } from "./Immersive";
import { AddToPlaylistModal } from "../ui/AddToPlaylistModal";
import { YtMatchModal } from "../ui/YtMatchModal";
import { DevicesPopover } from "./DevicesPopover";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";

/* collapsed sidebar. on mac the native traffic lights live in this column at
   their fixed OS positions (12px dots, 20px pitch, first centre at x=20), so
   the collapsed rail has to stay wide enough to hold all three. */
const COLLAPSED_SIDEBAR_W = 64;
const MAC_COLLAPSED_SIDEBAR_W = 72;

/* the notch cut out of the island card for the windows caption cluster. the
   card is inset 4px from the window, the cluster sits flush in the corner. */
const NOTCH_W = CAPTION_W - 4;
const NOTCH_H = CAPTION_H - 4;
const NOTCH_R = 10;

/* the card is clipped, not just covered, so the caption buttons sit on the
   same OS material as the sidebar instead of on the card's own tint. the
   polygon walks the notch's one inner corner as a quarter round, matching the
   cluster's border-radius exactly. */
const NOTCH_CLIP = (() => {
  const arc = [0, 22.5, 45, 67.5, 90].map((deg) => {
    const a = (deg * Math.PI) / 180;
    const x = (NOTCH_W - NOTCH_R + NOTCH_R * Math.cos(a)).toFixed(2);
    const y = (NOTCH_H - NOTCH_R + NOTCH_R * Math.sin(a)).toFixed(2);
    return `calc(100% - ${x}px) ${y}px`;
  });
  return `polygon(0 0, calc(100% - ${NOTCH_W}px) 0, ${arc.join(", ")}, 100% ${NOTCH_H}px, 100% 100%, 0 100%)`;
})();

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
  const collapsedSidebarW = (isMac || macSimulated) ? MAC_COLLAPSED_SIDEBAR_W : COLLAPSED_SIDEBAR_W;
  const [willCrushMain, setWillCrushMain] = useState(false);
  const [shellHidden, setShellHidden] = useState(false);

  useEffect(() => {
    if (immersiveOpen) {
      const t = setTimeout(() => setShellHidden(true), 420);
      return () => clearTimeout(t);
    } else {
      setShellHidden(false);
    }
  }, [immersiveOpen]);

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
  const hasRightRail = (lyricsOpen || queueOpen) && spacerWidth > 0;
  const isWindowsDocked = !isMac && !macSimulated && !hasRightRail;

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
  const setSearchPaletteOpen = useUIStore((s) => s.setSearchPaletteOpen);

  useEffect(() => {
    function onGlobalKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "m") {
        e.preventDefault();
        toggleMacSimulated();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchPaletteOpen(true);
      }
    }
    window.addEventListener("keydown", onGlobalKey);
    return () => window.removeEventListener("keydown", onGlobalKey);
  }, [toggleMacSimulated, setSearchPaletteOpen]);

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
          overflow: "hidden",
          position: "relative",
          visibility: shellHidden ? "hidden" : "visible",
        }}
        inert={shellHidden ? true : undefined}
      >
        <Sidebar />

        {/* The Main Window Island Card */}
        <div
          style={{
            flex: 1,
            minWidth: 0,
            margin: 4,
            borderRadius: 12,
            border: isWindowsDocked ? "none" : "1px solid rgba(255, 255, 255, 0.08)",
            borderLeft: isWindowsDocked ? "1px solid rgba(255, 255, 255, 0.08)" : undefined,
            borderBottom: isWindowsDocked ? "1px solid rgba(255, 255, 255, 0.08)" : undefined,
            background: "transparent",
            /* negative spread keeps the lift underneath the card instead of
               letting it bleed up into the 4px gutter the caption buttons
               share with the OS material */
            boxShadow: "0 14px 30px -8px rgba(0, 0, 0, 0.55)",
            position: "relative",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          {/* the card's material, on its own layer so the caption notch can be
              clipped out of it and let the OS material through.

              NO backdrop-filter here, deliberately. it never did anything:
              nothing is ever painted between the window root and this card, so
              it was only ever blurring transparency (or, with no live
              material, a flat scrim colour). what it DID do was give the layer
              its own compositing surface, and a clip-path over that surface
              comes back opaque black instead of punched through - which is
              exactly what was filling the notch. */}
          <div
            aria-hidden
            style={{
              position: "absolute",
              inset: 0,
              zIndex: 0,
              pointerEvents: "none",
              borderRadius: 12,
              background: cardBg,
              boxShadow: isWindowsDocked ? undefined : "inset 0 1px 0 rgba(255, 255, 255, 0.08)",
              clipPath: isWindowsDocked ? NOTCH_CLIP : undefined,
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
                  clipPath: isWindowsDocked ? NOTCH_CLIP : undefined,
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

          {/* Title bar at top of card */}
          <TitleBar />

          {/* the card's hairline, drawn by hand so it can run around the
              caption notch: across the top, down the notch's left and along
              its underside, then down the right edge. nothing above or to the
              right of the buttons themselves. */}
          {isWindowsDocked && (
            <>
              <div
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  right: NOTCH_W,
                  height: 1,
                  background: "rgba(255, 255, 255, 0.08)",
                  borderTopLeftRadius: 12,
                  pointerEvents: "none",
                  zIndex: 15,
                }}
              />
              <div
                style={{
                  position: "absolute",
                  top: 0,
                  right: 0,
                  width: NOTCH_W,
                  height: NOTCH_H,
                  borderLeft: "1px solid rgba(255, 255, 255, 0.08)",
                  borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
                  borderBottomLeftRadius: NOTCH_R,
                  pointerEvents: "none",
                  zIndex: 15,
                }}
              />
              <div
                style={{
                  position: "absolute",
                  top: NOTCH_H,
                  right: 0,
                  bottom: 0,
                  width: 1,
                  background: "rgba(255, 255, 255, 0.08)",
                  borderBottomRightRadius: 12,
                  pointerEvents: "none",
                  zIndex: 15,
                }}
              />
            </>
          )}

          {/* Scrolling page view inside card */}
          <div style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden" }}>
            <main
              ref={mainRef}
              data-selectable
              style={{
                position: "absolute",
                inset: 0,
                overflowY: "auto",
                overflowX: "hidden",
                paddingTop: 8,
                paddingLeft: "clamp(14px, 2.5vw, 32px)",
                paddingRight: "clamp(14px, 2.5vw, 32px)",
                paddingBottom: "90px",
              }}
            >
              <motion.div
                key={location.pathname}
                initial={reduceMotion ? false : { opacity: 0, y: 6, scale: 0.995 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
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
            width: spacerWidth,
            flexShrink: 0,
            position: "relative",
            overflow: "hidden",
            height: "100%",
            display: spacerWidth > 0 ? "flex" : "none",
          }}
        >
          <AnimatePresence initial={false}>
            {lyricsOpen && <LyricsPanel key="lyrics" />}
          </AnimatePresence>
          <AnimatePresence initial={false}>
            {queueOpen && <QueuePanel key="queue" />}
          </AnimatePresence>
        </div>

      </div>

      <SearchPalette />
      <DevicesPopover />
      <Immersive />
      <QuitConfirm />
      <AddToPlaylistModal />
      <YtMatchModal />
      <Toaster />
      {/* always at the window's own top-right corner, never inside the card,
          so the buttons sit on the OS material and the close button owns the
          corner pixel */}
      <WindowCaptionControls />
    </div>
  );
}
