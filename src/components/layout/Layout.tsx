import { useEffect } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { TitleBar } from "./TitleBar";
import Sidebar from "./Sidebar";
import { PlayerBar } from "./PlayerBar";
import { QueuePanel } from "./QueuePanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm } from "../ui/QuitConfirm";
import { Toaster } from "../ui/Toaster";
import { Immersive } from "./Immersive";
import { AddToPlaylistModal } from "../ui/AddToPlaylistModal";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";

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

  // ask rust whether a native material actually took this launch. until it
  // answers (and anywhere it didn't - Linux, or a failed Mica/vibrancy) the
  // root stays opaque dark so we never get white-on-white.
  useEffect(() => {
    getBackdropActive().then(setBackdropActive).catch(() => setBackdropActive(false));
  }, [setBackdropActive]);

  // backdrop strategy:
  //  - no live material (Linux, or Mica/vibrancy failed) -> paint the solid app
  //    bg, otherwise the transparent window shows white (or the desktop)
  //  - windows acrylic -> OS ignores the tint, so darken with a CSS scrim
  //  - windows Mica / macOS vibrancy -> stay transparent, OS material shows
  const scrim = backdropScrim(backdropActive, effect, materialTransparency, isMac);

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

        <div
          style={{
            flex: 1,
            // minWidth:0 lets this column shrink below its content's intrinsic
            // width instead of overflowing, so it stays responsive as side
            // panels (queue/lyrics/whatever) open and the window narrows. no
            // visual change while there's room.
            minWidth: 0,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            background: "var(--color-content)",
            position: "relative",
          }}
        >
          {/* fixed artwork tint behind the content (not the sidebar). lives in
              the non-scrolling column so it stays put while <main> scrolls. an
              inverse-vignette radial mask blooms the colour out of the cover
              (top-left) and fades it to nothing - no hard edges, no top/bottom
              gradient cuts. */}
          <AnimatePresence>
            {pageTint && (
              <motion.div
                key={pageTint}
                aria-hidden
                initial={{ opacity: 0 }}
                animate={{ opacity: 0.42 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.7, ease: "easeOut" }}
                style={{
                  position: "absolute",
                  inset: 0,
                  zIndex: 0,
                  pointerEvents: "none",
                }}
              >
                {/* blurred artwork bloom, centred horizontally, emerging from
                    the top-centre. scale pushes the blurred edges off-frame so
                    no visible seams; saturate keeps the colour alive. */}
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
                {/* fine fractal-noise dither over the bloom. a smooth blurred
                    gradient bands into visible steps on flat panels; a faint
                    noise layer breaks them up so the falloff stays clean. */}
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

          <div
            style={{
              position: "relative",
              zIndex: 1,
              flex: 1,
              minHeight: 0,
              minWidth: 0,
              display: "flex",
              flexDirection: "column",
              overflow: "hidden",
            }}
          >
            <TitleBar />

            {/* main region + right rail sit in ONE horizontal row BELOW the title
                bar, so the rail (lyrics/queue) can never cover the window's
                caption/close buttons. The rail is a real sibling of <main>: an
                in-flow spacer reserves its width (instantly, no transition) so
                the content grid reflows in one step on open AND close, and the
                panel just slides into that reserved slot. */}
            <div style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden", display: "flex" }}>
              <div style={{ position: "relative", flex: 1, minWidth: 0, overflow: "hidden" }}>
                <main
                  data-selectable
                  style={{
                    position: "absolute",
                    inset: 0,
                    overflowY: "auto",
                    overflowX: "hidden",
                    padding: "clamp(6px, 1.4vw, 12px) clamp(14px, 3vw, 32px) clamp(16px, 3vw, 32px)",
                  }}
                >
                  {/* page-load motion: content rises gently from below on each
                    nav. subtle (14px / 0.34s), respects reduced-motion. */}
                  <motion.div
                    key={location.pathname}
                    initial={reduceMotion ? false : { opacity: 0, y: 14 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.34, ease: [0.23, 1, 0.32, 1] }}
                  >
                    <Outlet />
                  </motion.div>
                </main>
              </div>

              {/* reserves the rail's width instantly (no transition) so the grid
                  reflows once; the panel below slides over this exact slot. */}
              <div aria-hidden style={{ width: lyricsOpen ? 366 : queueOpen ? 272 : 0, flexShrink: 0 }} />

              {/* right rail - lyrics or queue, mutually exclusive. positioned
                  against THIS row (below the title bar), sliding in via a transform. */}
              <AnimatePresence initial={false}>
                {lyricsOpen && <LyricsPanel key="lyrics" />}
              </AnimatePresence>
              <AnimatePresence initial={false}>
                {queueOpen && <QueuePanel key="queue" />}
              </AnimatePresence>
            </div>
          </div>
        </div>
      </div>

      <PlayerBar />
      <Immersive />
      <QuitConfirm />
      <AddToPlaylistModal />
      <Toaster />
    </div>
  );
}
