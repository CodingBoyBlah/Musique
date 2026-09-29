import { memo, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion, useMotionValue, useTransform, animate } from "framer-motion";
import { TitleBar as TitleBarView, WindowCaptionControls as CaptionControlsView } from "./TitleBar";
import SidebarView from "./Sidebar";
import { PlayerBar as PlayerBarView } from "./PlayerBar";
import { usePositionTicker } from "../../hooks/usePositionTicker";
import { QueuePanel } from "./QueuePanel";
import { FriendsPanel } from "./FriendsPanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm as QuitConfirmView } from "../ui/QuitConfirm";
import { Toaster as ToasterView } from "../ui/Toaster";
import { Immersive as ImmersiveView } from "./Immersive";
import { AddToPlaylistModal as AddToPlaylistView } from "../ui/AddToPlaylistModal";
import { CreditsModal as CreditsView } from "../ui/CreditsModal";
import { YtMatchModal as YtMatchView } from "../ui/YtMatchModal";
import { DevicesPopover as DevicesView } from "./DevicesPopover";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";
import { usePrefsStore } from "../../store/prefs.store";
import { chromePx } from "../../lib/zoom";
import { EASE_OUT, RAIL_CLOSE, RAIL_CLOSE_S, RAIL_OPEN, RAIL_OPEN_S } from "../../lib/motion";
import { pinRailGrids } from "../../lib/railFlip";
import "../../styles/layout.css";

/* The shell's fixed pieces take no props from Layout - each reads the stores
   it needs itself - so Layout re-rendering (every panel toggle, every resize
   that crosses the crush line) has nothing to tell them. Memoised, a panel
   toggle renders Layout and the panel, not the whole window, which is most of
   the work in the frame the slide starts on. */
const Sidebar = memo(SidebarView);
const TitleBar = memo(TitleBarView);
const WindowCaptionControls = memo(CaptionControlsView);
const PlayerBar = memo(PlayerBarView);
const QuitConfirm = memo(QuitConfirmView);
const Toaster = memo(ToasterView);
const Immersive = memo(ImmersiveView);
const AddToPlaylistModal = memo(AddToPlaylistView);
const CreditsModal = memo(CreditsView);
const YtMatchModal = memo(YtMatchView);
const DevicesPopover = memo(DevicesView);

/* collapsed sidebar. on mac the native traffic lights live in this column at
   their fixed OS positions (12px dots, 20px pitch, first centre at x=20), so
   the collapsed rail has to stay wide enough to hold all three. */
const COLLAPSED_SIDEBAR_W = 64;
const MAC_COLLAPSED_SIDEBAR_W = 72;

export default function Layout() {
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  // the panel can be switched off in settings; a stale open flag must not keep it
  const friendsFlag = usePlayerStore((s) => s.friendsOpen);
  const friendsOn = usePrefsStore((s) => s.showFriends);
  const friendsOpen = friendsFlag && friendsOn;
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
      const rpw = lyricsOpen ? 366 : queueOpen || friendsOpen ? 272 : 0;
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
  }, [sidebarCollapsed, collapsedSidebarW, lyricsOpen, queueOpen, friendsOpen]);

  const rawPanelWidth = lyricsOpen ? 366 : queueOpen || friendsOpen ? 272 : 0;
  const spacerWidth = willCrushMain ? 0 : rawPanelWidth;

  /* The right rail slides, and the page flows with it.

     frameRail is the rail's width, and with it the card's right edge and the
     dock. It runs on the same curve as the panel's sheet, so the edge and the
     panel travel as one piece, and the page - headings, text, track rows -
     reflows with the edge as it moves instead of jumping to its new width.

     Tile grids are pinned to their landing width on the first frame and their
     cards FLIPped there on the compositor, in step with the edge - see
     lib/railFlip. Nothing in a slide renders through React after the first
     frame: the edge is a motion value and the cards are Web Animations, which
     is what keeps the close from stalling on the frame it lands. */
  const frameRail = useMotionValue(spacerWidth);
  const railW = useTransform(frameRail, (v) => Math.max(0, v));
  const tintRight = useTransform(railW, (v) => -v);
  const tintShift = useTransform(railW, (v) => -v / 2);

  useLayoutEffect(() => {
    const from = frameRail.get();
    if (from === spacerWidth) return;
    if (reduceMotion || !mainRef.current) {
      frameRail.jump(spacerWidth);
      return;
    }
    const main = mainRef.current;
    const opening = spacerWidth > from;
    const seconds = opening ? RAIL_OPEN_S : RAIL_CLOSE_S;
    let live = true;
    let unpin = () => {};
    // after the commit but before the frame is painted: the JS-columned grids
    // re-render synchronously inside the pin, which React refuses mid-commit
    queueMicrotask(() => {
      if (live) unpin = pinRailGrids(main, from - spacerWidth, seconds * 1000);
    });
    const ctl = animate(frameRail, spacerWidth, opening ? RAIL_OPEN : RAIL_CLOSE);
    ctl.then(() => { if (live) unpin(); });
    // the pins hold the landing width, so releasing them moves nothing - it
    // only hands the grids back to the page for the next resize. An
    // interrupted slide releases them too; the next one re-pins from where
    // the page actually is.
    return () => {
      live = false;
      ctl.stop();
      unpin();
    };
  }, [spacerWidth, frameRail, reduceMotion]);

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
                      /* Blurred, so it must never be resized mid-slide - that
                         re-runs the blur every frame. It stays the width of
                         card + rail (a constant), overhangs the card's edge
                         where the card clips it, and is moved back to the
                         card's centre by a transform, which costs nothing. */
                      top: 0, left: 0, bottom: 0,
                      right: tintRight,
                      x: tintShift,
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
            <motion.div
              style={{
                width: railW,
                flexShrink: 0,
                position: "relative",
                overflow: "hidden",
                height: "100%",
                display: "flex",
              }}
            >
              <AnimatePresence initial={false}>
                {lyricsOpen && <LyricsPanel key="lyrics" />}
              </AnimatePresence>
              <AnimatePresence initial={false}>
                {queueOpen && <QueuePanel key="queue" />}
              </AnimatePresence>
              <AnimatePresence initial={false}>
                {friendsOpen && <FriendsPanel key="friends" />}
              </AnimatePresence>
            </motion.div>
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
