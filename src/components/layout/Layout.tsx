import { useEffect, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { TitleBar, WindowCaptionControls, WindowCaptionHitboxOverlay } from "./TitleBar";
import { SearchPalette } from "./SearchPalette";
import Sidebar from "./Sidebar";
import { PlayerBar } from "./PlayerBar";
import { QueuePanel } from "./QueuePanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm } from "../ui/QuitConfirm";
import { Toaster } from "../ui/Toaster";
import { Immersive } from "./Immersive";
import { AddToPlaylistModal } from "../ui/AddToPlaylistModal";
import { DevicesPopover } from "./DevicesPopover";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";
import { invoke } from "@tauri-apps/api/core";

export default function Layout() {
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const effect = useUIStore((s) => s.windowEffect);
  const materialTransparency = useUIStore((s) => s.materialTransparency);
  const pageTint = useUIStore((s) => s.pageTint);
  const backdropActive = useUIStore((s) => s.backdropActive);
  const setBackdropActive = useUIStore((s) => s.setBackdropActive);
  const location = useLocation();
  const reduceMotion = useReducedMotion();
  const mainRef = useRef<HTMLElement>(null);

  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const [willCrushMain, setWillCrushMain] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const checkCrush = () => {
      const w = window.innerWidth;
      const isCollapsed = sidebarCollapsed || w < 768;
      const sw = isCollapsed ? 64 : 232;
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
  }, [sidebarCollapsed, lyricsOpen, queueOpen]);

  const rawPanelWidth = lyricsOpen ? 366 : queueOpen ? 272 : 0;
  const spacerWidth = willCrushMain ? 0 : rawPanelWidth;
  const macSimulated = useUIStore((s) => s.macSimulated);
  const hasRightRail = (lyricsOpen || queueOpen) && spacerWidth > 0;
  const isWindowsDocked = !isMac && !macSimulated && !hasRightRail;

  useEffect(() => {
    if (mainRef.current) mainRef.current.scrollTop = 0;
    const t = setTimeout(() => {
      invoke("trim_memory").catch(() => {});
    }, 1500);
    return () => clearTimeout(t);
  }, [location.pathname]);

  useEffect(() => {
    const handleVis = () => {
      if (document.visibilityState === "hidden") {
        invoke("trim_memory").catch(() => {});
      }
    };
    document.addEventListener("visibilitychange", handleVis);
    return () => document.removeEventListener("visibilitychange", handleVis);
  }, []);

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
      <div style={{ display: "flex", flex: 1, overflow: "hidden", position: "relative" }}>
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
            background: cardBg,
            boxShadow: isWindowsDocked
              ? "0 10px 30px rgba(0, 0, 0, 0.5)"
              : "0 10px 30px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.08)",
            backdropFilter: isTranslucent ? "blur(32px) saturate(140%)" : "blur(24px)",
            WebkitBackdropFilter: isTranslucent ? "blur(32px) saturate(140%)" : "blur(24px)",
            position: "relative",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
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

          {/* Title bar at top of card */}
          <TitleBar />

          {/* Top border extending to caption controls when docked */}
          {isWindowsDocked && (
            <div
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                right: 137,
                height: 1,
                background: "rgba(255, 255, 255, 0.08)",
                borderTopLeftRadius: 12,
                pointerEvents: "none",
                zIndex: 15,
              }}
            />
          )}

          {/* Right border below caption controls when docked */}
          {isWindowsDocked && (
            <div
              style={{
                position: "absolute",
                top: 39,
                right: 0,
                bottom: 0,
                width: 1,
                background: "rgba(255, 255, 255, 0.08)",
                borderBottomRightRadius: 12,
                pointerEvents: "none",
                zIndex: 15,
              }}
            />
          )}

          {/* Window Caption Controls docked into top-right notch of Island Card */}
          {isWindowsDocked && <WindowCaptionControls dockedToCard />}

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

          {/* PlayerBar docked inside card */}
          <PlayerBar />
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

        {/* Invisible Hitbox Overlay extending to top and right edges/corners of window */}
        {isWindowsDocked && <WindowCaptionHitboxOverlay />}
      </div>

      <SearchPalette />
      <DevicesPopover />
      <Immersive />
      <QuitConfirm />
      <AddToPlaylistModal />
      <Toaster />
      {!isWindowsDocked && <WindowCaptionControls />}
    </div>
  );
}
