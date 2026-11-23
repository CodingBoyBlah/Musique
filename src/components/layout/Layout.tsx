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

/* The routed page, cut off from Layout's own renders. A bare <Outlet/>
   re-renders the whole page every time Layout does - a panel toggle, a
   play/pause, a new cover tint - because each Layout render hands the page a
   fresh route context. On an album that was every track row and tooltip
   rendering on the first frame of a panel slide. Memoised, the page renders
   when its route does and when its own state changes, never for the shell. */
const PageOutlet = memo(function PageOutlet() {
  return <Outlet />;
});

/* collapsed sidebar. on mac the native traffic lights live in this column at
   their fixed OS positions (12px dots, 20px pitch, first centre at x=20), so
   the collapsed rail has to stay wide enough to hold all three. */
const COLLAPSED_SIDEBAR_W = 64;

// what sequential focus navigation stops on
const TABBABLE =
  'a[href],button:not([disabled]),input:not([disabled]):not([type="hidden"]),select:not([disabled]),textarea:not([disabled]),[tabindex],[contenteditable]:not([contenteditable="false"]),audio[controls],video[controls],summary';
const MAC_COLLAPSED_SIDEBAR_W = 72;

/* the page's cover-art wash: concentrated behind the header (cover, title,
   buttons), then a long ease-out tail that thins to nothing toward the
   bottom. many stops on a quadratic curve so the fade has no visible edge. */
const BLOOM_MASK =
  "linear-gradient(180deg, #000 0%, rgba(0,0,0,0.96) 12%, rgba(0,0,0,0.8) 24%, " +
  "rgba(0,0,0,0.58) 34%, rgba(0,0,0,0.38) 44%, rgba(0,0,0,0.23) 54%, " +
  "rgba(0,0,0,0.12) 64%, rgba(0,0,0,0.05) 76%, rgba(0,0,0,0.015) 88%, transparent 100%)";

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
  const shellRef = useRef<HTMLDivElement>(null);

  const immersiveCovered = useUIStore((s) => s.immersiveCovered);
  const setImmersiveCovered = useUIStore((s) => s.setImmersiveCovered);

  /* The shell under the immersive overlay.

     Once the overlay has fully faded in, the shell stops rendering
     (content-visibility: hidden). It matters: the lyric sweep repaints a line
     every frame, and every repaint re-layerizes the whole document - with the
     shell still painted underneath that doubled the renderer's CPU for as
     long as the view stayed up.

     It used to be hidden with visibility (and inerted). Both are inherited,
     so each flip restyled all ~1500 elements of the shell, and inert changes
     the hit-test data recorded in paint, so it repainted it too: a 20-26ms
     frame on the way in and another on the frame the exit fade started.
     content-visibility is not inherited (one element restyles), keeps the
     shell's style, layout and scroll positions, and takes its contents out of
     focus order. It comes back in the same commit the exit fade starts in;
     the overlay is still opaque there, and its tiles raster under the fade.

     The timer is only a backstop in case the overlay's signal never arrives -
     it is well past the 260ms fade, so it never races the real one. */
  const shellHidden = immersiveOpen && immersiveCovered;
  useEffect(() => {
    if (!immersiveOpen) {
      if (immersiveCovered) setImmersiveCovered(false);
      return;
    }
    if (immersiveCovered) return;
    const t = setTimeout(() => setImmersiveCovered(true), 700);
    return () => clearTimeout(t);
  }, [immersiveOpen, immersiveCovered, setImmersiveCovered]);

  /* Tab, before the shell is hidden (the fade in) - content-visibility takes
     care of it after. Focus moves through the focusable elements outside the
     shell in document order, wrapping at either end: the order the browser
     gives once the shell is out of it. On document, in the capture phase, so a
     modal's own trap (on window) runs first; whatever it has already handled
     is left alone. Steering Tab costs nothing; inerting the shell for this
     restyled and repainted all of it. */
  useEffect(() => {
    const shell = shellRef.current;
    if (!immersiveOpen || !shell) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Tab" || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
      const order = Array.from(document.querySelectorAll<HTMLElement>(TABBABLE)).filter(
        (el) => el.tabIndex >= 0 && !shell.contains(el) && !el.closest("[inert]") && el.getClientRects().length > 0,
      );
      e.preventDefault();
      if (!order.length) return;
      const at = order.indexOf(document.activeElement as HTMLElement);
      const next = at === -1
        ? (e.shiftKey ? order[order.length - 1] : order[0])
        : order[(at + (e.shiftKey ? -1 : 1) + order.length) % order.length];
      next.focus();
    };
    document.addEventListener("keydown", onKey, true);
    return () => document.removeEventListener("keydown", onKey, true);
  }, [immersiveOpen]);

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

  // Ctrl+K lives with the search field in the top bar (TopSearch).
  // Ctrl+Shift+M (fake mac chrome) is a dev-only layout aid
  useEffect(() => {
    if (!import.meta.env.DEV) return;
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
          contentVisibility: shellHidden ? "hidden" : undefined,
        }}
        ref={shellRef}
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
                    animate={{ opacity: 0.4 }}
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
                        /* brightness pulls pale covers down so white type
                           on top keeps its contrast at the higher opacity */
                        filter: "blur(80px) saturate(1.6) brightness(0.85)",
                        transform: "scale(1.6)",
                        transformOrigin: "center top",
                        /* a full-width wash that eases out down the page
                           (many stops so the fade has no visible edge),
                           rather than a spotlight in the middle */
                        maskImage: BLOOM_MASK,
                        WebkitMaskImage: BLOOM_MASK,
                      }}
                    />
                    {/* grain breaks up the banding a smooth dark gradient
                        gets on 8-bit panels */}
                    <div
                      style={{
                        position: "absolute",
                        inset: 0,
                        maskImage: BLOOM_MASK,
                        WebkitMaskImage: BLOOM_MASK,
                        opacity: 0.07,
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
                  {/* `transform` as one string, not framer's y/scale: those
                      are composed in JS and written every frame on the main
                      thread - right while the new page mounts - where a
                      whole transform runs as a Web Animation on the
                      compositor. Same values, same curve. Back to "none" at
                      rest (transitionEnd) - framer reads a "none" target as
                      scale(0), and any leftover transform would make this the
                      containing block for the pages' unportalled modals. */}
                  <motion.div
                    key={location.pathname}
                    initial={reduceMotion ? false : { opacity: 0, transform: "translateY(6px) scale(0.995)" }}
                    animate={{ opacity: 1, transform: "translateY(0px) scale(1)", transitionEnd: { transform: "none" } }}
                    transition={{ duration: 0.18, ease: EASE_OUT }}
                  >
                    <PageOutlet />
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
