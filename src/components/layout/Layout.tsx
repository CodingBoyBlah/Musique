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

  
  const scrim = backdropScrim(backdropActive, effect, materialTransparency, isMac);

  return (
    
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
            
            minWidth: 0,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            background: "var(--color-content)",
            position: "relative",
          }}
        >
          
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

          <div
            style={{
              position: "relative",
              zIndex: 1,
              flex: 1,
              minHeight: 0,
              minWidth: 0,
              display: "flex",
              overflow: "hidden",
            }}
          >
            
            <div style={{ position: "absolute", top: 0, left: 0, right: 0, zIndex: 50, pointerEvents: "none" }}>
              <TitleBar />
            </div>
            {/* main region + right rail sit in ONE horizontal row */}
            <div style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden", display: "flex" }}>
              <div style={{ position: "relative", flex: 1, minWidth: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}>
                {/* Spacer blocks main scrolling content but lets page tint through */}
                <div style={{ height: 48, flexShrink: 0 }} />
                <main
                  data-selectable
                  style={{
                    position: "relative",
                    flex: 1,
                    overflowY: "auto",
                    overflowX: "hidden",
                    paddingTop: "clamp(10px, 1.4vw, 16px)",
                    paddingLeft: "clamp(14px, 3vw, 32px)",
                    paddingRight: "clamp(16px, 3vw, 32px)",
                    paddingBottom: "clamp(16px, 3vw, 32px)",
                  }}
                >
                  
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

              
              <div aria-hidden style={{ width: lyricsOpen ? 366 : queueOpen ? 272 : 0, flexShrink: 0 }} />

              
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
