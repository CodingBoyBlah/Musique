import { useEffect, useRef, useState, memo } from "react";
import { Link } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { prefetchAlbum, prefetchArtist } from "../../lib/prefetch";
import {
  Play, Pause, SkipBack, SkipForward,
  Shuffle, Repeat, Repeat1,
  Volume2, Volume1, VolumeX,
  Devices, Maximize2,
} from "@/lib/icons";
import { useShallow } from "zustand/react/shallow";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import {
  setVolume as apiSetVolume, setMuted as apiSetMuted,
} from "../../api/playback";
import { remoteSetVolume } from "../../api/connect";
import { useDevices } from "../../hooks/useDevices";
import { CoverArt } from "../ui/CoverArt";
import { Tooltip } from "../ui/Tooltip";
import { fmtMs } from "../../utils/fmt";
import { usePlayerControls } from "../../hooks/usePlayerControls";
import { gpuLayer, zTransform, EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";

/* Transport buttons. These are pressed dozens of times a day, so the feedback
   is small and immediate: a 0.96 press, no hover growth (the colour change is
   enough), no bounce. Hover is CSS (.pb-btn in styles/layout.css) - the old
   onMouseEnter wrote node.style, which any re-render wiped. */
function IconBtn({
  children, onClick, active, large, title, disabled, ariaLabel, pressed,
}: {
  children:   React.ReactNode;
  onClick?:   () => void;
  active?:    boolean;
  large?:     boolean;
  title?:     string;
  disabled?:  boolean;
  ariaLabel?: string;
  // for toggles (shuffle, repeat): exposes on/off to assistive tech
  pressed?:   boolean;
}) {
  return (
    <motion.button
      onClick={disabled ? undefined : onClick}
      title={title}
      aria-label={ariaLabel}
      aria-pressed={pressed}
      disabled={disabled}
      className="pb-btn focus-ring"
      data-on={active || undefined}
      data-large={large || undefined}
      whileTap={disabled ? undefined : PRESS}
      transition={PRESS_TRANSITION}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        flexShrink:     0,
        width:          large ? 32 : 28,
        height:         large ? 32 : 28,
        borderRadius:   "50%",
        border:         "none",
        background:     large ? "rgba(242,238,233,0.90)" : "transparent",
        color:          large
          ? "#0a0f0c"
          : active
            ? "var(--color-accent)"
            : disabled
              ? "rgba(242,238,233,0.18)"
              : undefined,
        cursor:         disabled ? "default" : "pointer",
      }}
    >
      {children}
    </motion.button>
  );
}

function PlayPauseButton({
  isPlaying, onClick,
}: {
  isPlaying: boolean; onClick: () => void;
}) {
  return (
    <motion.button
      onClick={onClick}
      aria-label={isPlaying ? "Pause" : "Play"}
      className="focus-ring"
      whileTap={PRESS}
      transition={PRESS_TRANSITION}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        position: "relative",
        display: "flex", alignItems: "center", justifyContent: "center",
        width: 38, height: 38, flexShrink: 0,
        borderRadius: "50%",
        border: "none",
        background: "transparent",
        color: "var(--color-text-hi, #ffffff)",
        cursor: "pointer",
      }}
    >
      <AnimatePresence initial={false}>
        <motion.span
          key={isPlaying ? "pause" : "play"}
          initial={{ opacity: 0, scale: 0.55, filter: "blur(5px)" }}
          animate={{ opacity: 1, scale: 1,    filter: "blur(0px)" }}
          exit={{    opacity: 0, scale: 0.55, filter: "blur(5px)" }}
          transition={{ duration: 0.19, ease: EASE_OUT }}
          style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}
        >
          {isPlaying
            ? <Pause size={19} strokeWidth={2.6} fill="currentColor" />
            : <Play  size={19} strokeWidth={2.6} fill="currentColor" style={{ marginLeft: 2 }} />
          }
        </motion.span>
      </AnimatePresence>
    </motion.button>
  );
}

const SEEK_STEP_MS = 5000;

const PlayerScrubber = memo(function PlayerScrubber({
  durationMs,
  doSeek,
  showTimes,
}: {
  durationMs: number;
  doSeek: (ms: number) => void;
  // elapsed / remaining either side of the line. dropped when the dock is too
  // narrow to give the line itself a useful length.
  showTimes: boolean;
}) {
  const positionMs = usePlayerStore((s) => s.positionMs);
  const [isScrubHovered, setIsScrubHovered] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [dragFrac, setDragFrac] = useState<number | null>(null);
  const [hoverFrac, setHoverFrac] = useState<number | null>(null);
  const scrubBarRef = useRef<HTMLDivElement>(null);
  // the pointer handlers read these, not state: pointerup can land before the
  // render that would have carried the last pointermove's value
  const draggingRef = useRef(false);
  const dragFracRef = useRef<number | null>(null);

  const activeFrac = isDragging && dragFrac !== null
    ? dragFrac
    : durationMs > 0
      ? Math.min(positionMs / durationMs, 1)
      : 0;

  const pct = activeFrac * 100;

  function fracFromClientX(clientX: number) {
    const el = scrubBarRef.current;
    if (!el) return 0;
    const rect = el.getBoundingClientRect();
    if (rect.width <= 0) return 0;
    return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  }

  function setDrag(f: number | null) {
    dragFracRef.current = f;
    setDragFrac(f);
  }

  function endDrag(commit: boolean) {
    if (!draggingRef.current) return;
    const f = dragFracRef.current;
    if (commit && f !== null && durationMs > 0) {
      doSeek(Math.floor(f * durationMs));
    }
    draggingRef.current = false;
    setIsDragging(false);
    setDrag(null);
  }

  function handlePointerDown(e: React.PointerEvent<HTMLDivElement>) {
    if (!durationMs) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    const f = fracFromClientX(e.clientX);
    draggingRef.current = true;
    setIsDragging(true);
    setDrag(f);
    setHoverFrac(f);
  }

  function handlePointerMove(e: React.PointerEvent<HTMLDivElement>) {
    if (!durationMs) return;
    const f = fracFromClientX(e.clientX);
    setHoverFrac(f);
    if (draggingRef.current) setDrag(f);
  }

  function handlePointerUp(e: React.PointerEvent<HTMLDivElement>) {
    endDrag(true);
    try {
      e.currentTarget.releasePointerCapture(e.pointerId);
    } catch {}
  }

  // the OS or browser took the pointer away (alt-tab, touch cancelled): drop
  // the drag where it was rather than leaving the bar stuck in drag mode
  function handlePointerCancel() {
    endDrag(false);
    setHoverFrac(null);
    setIsScrubHovered(false);
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    if (!durationMs) return;
    const pos = usePlayerStore.getState().positionMs;
    let target: number | null = null;
    if (e.key === "ArrowRight" || e.key === "ArrowUp") target = pos + SEEK_STEP_MS;
    else if (e.key === "ArrowLeft" || e.key === "ArrowDown") target = pos - SEEK_STEP_MS;
    else if (e.key === "PageUp") target = pos + durationMs * 0.1;
    else if (e.key === "PageDown") target = pos - durationMs * 0.1;
    else if (e.key === "Home") target = 0;
    else if (e.key === "End") target = durationMs - 1000;
    if (target === null) return;
    e.preventDefault();
    e.stopPropagation();
    doSeek(Math.max(0, Math.min(durationMs, Math.floor(target))));
  }

  const showTooltip = (hoverFrac !== null || isDragging) && durationMs > 0;
  const tooltipFrac = isDragging && dragFrac !== null ? dragFrac : (hoverFrac ?? activeFrac);
  const isLineExpanded = isScrubHovered || isDragging;

  const shownMs = Math.floor(activeFrac * durationMs);
  const remainingMs = Math.max(0, durationMs - shownMs);

  const timeStyle: React.CSSProperties = {
    flexShrink: 0,
    minWidth: 30,
    fontSize: 10.5,
    fontWeight: 500,
    lineHeight: 1,
    color: "var(--color-text-dim)",
  };

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, width: "100%" }}>
      {showTimes && durationMs > 0 && (
        <span className="tnum t-caption" aria-hidden style={{ ...timeStyle, textAlign: "right" }}>
          {fmtMs(shownMs)}
        </span>
      )}
      <div
        ref={scrubBarRef}
        className="scrub"
        role="slider"
        tabIndex={durationMs > 0 ? 0 : -1}
        aria-label="Seek"
        aria-valuemin={0}
        aria-valuemax={Math.round(durationMs / 1000)}
        aria-valuenow={Math.round(shownMs / 1000)}
        aria-valuetext={durationMs > 0 ? `${fmtMs(shownMs)} of ${fmtMs(durationMs)}` : "Nothing playing"}
        aria-disabled={durationMs > 0 ? undefined : true}
        onKeyDown={handleKeyDown}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerCancel}
        onLostPointerCapture={() => endDrag(false)}
        onPointerEnter={() => setIsScrubHovered(true)}
        onPointerLeave={() => {
          if (!draggingRef.current) {
            setIsScrubHovered(false);
            setHoverFrac(null);
          }
        }}
        style={{
          position: "relative",
          flex: 1,
          minWidth: 0,
          height: 12,
          display: "flex",
          alignItems: "center",
          cursor: durationMs > 0 ? "pointer" : "default",
          touchAction: "none",
        }}
      >
        {/* Expanding hairline track */}
        <div
          className="scrub-track"
          style={{
            position: "relative",
            width: "100%",
            height: 5,
            borderRadius: 9999,
            background: isLineExpanded ? "rgba(255, 255, 255, 0.22)" : "rgba(255, 255, 255, 0.14)",
            overflow: "hidden",
            transform: isLineExpanded ? "scaleY(1)" : "scaleY(0.45)",
            transformOrigin: "center",
            transition: isDragging ? "none" : "transform 0.20s var(--ease-out), background 0.20s",
          }}
        >
          <div
            style={{
              position: "absolute",
              inset: 0,
              transformOrigin: "left",
              borderRadius: 9999,
              background: "#ffffff",
              transform: `scaleX(${pct / 100})`,
              transition: isDragging ? "none" : "transform 0.12s linear",
              boxShadow: isLineExpanded ? "0 0 8px rgba(255, 255, 255, 0.45)" : "none",
            }}
          />
        </div>

        {/* Floating Tooltip displaying the scrubbed timestamp on interaction */}
        <AnimatePresence>
          {showTooltip && (
            <motion.div
              initial={{ opacity: 0, y: 3, scale: 0.94 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 3, scale: 0.94 }}
              transition={{ duration: 0.12, ease: EASE_OUT }}
              className="tnum"
              style={{
                position: "absolute",
                left: `${tooltipFrac * 100}%`,
                bottom: "calc(100% + 5px)",
                // framer composes its own transform from x/y/scale, so the
                // centring has to be one of its values - a transform string
                // here would be thrown away and the label would hang off the
                // cursor's right-hand side
                x: "-50%",
                transformOrigin: "bottom center",
                pointerEvents: "none",
                whiteSpace: "nowrap",
                padding: "2px 6px",
                borderRadius: 5,
                background: "rgba(22, 22, 26, 0.96)",
                border: "1px solid rgba(255, 255, 255, 0.14)",
                boxShadow: "0 4px 14px rgba(0, 0, 0, 0.55)",
                fontSize: 10.5,
                fontWeight: 600,
                letterSpacing: "var(--type-caption-track)",
                color: "#ffffff",
                zIndex: 60,
              }}
            >
              {fmtMs(Math.floor(tooltipFrac * durationMs))}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
      {showTimes && durationMs > 0 && (
        <span className="tnum t-caption" aria-hidden style={timeStyle}>
          -{fmtMs(remainingMs)}
        </span>
      )}
    </div>
  );
});

/* `immersive` means this bar is floating on the immersive backdrop rather
   than docked in the app card. It drops the 44px sleeve, which is already
   filling the screen behind it, and takes the same translucent shade as the
   lyrics card so the two read as one set of surfaces over the artwork. */
export function PlayerBar({ immersive = false }: { immersive?: boolean }) {
  const qc              = useQueryClient();
  const isPlaying       = usePlayerStore((s) => s.isPlaying);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);
  const durationMs      = usePlayerStore((s) => s.durationMs);
  const volume          = usePlayerStore((s) => s.volume);
  const muted           = usePlayerStore((s) => s.muted);
  const storeSetVolume  = usePlayerStore((s) => s.setVolume);
  const storeSetMuted   = usePlayerStore((s) => s.setMuted);
  const setImmersiveOpen = usePlayerStore((s) => s.setImmersiveOpen);

  const { togglePlay, next: handleNext, prev: handlePrev, seek: doSeek } = usePlayerControls();
  const { activeDevice, isRemotePlayback, toggleDevices } = useDevices();

  const { shuffle, repeat, toggleShuffle, cycleRepeat } = useQueueStore(
    useShallow((s) => ({
      shuffle: s.shuffle,
      repeat:  s.repeat,
      toggleShuffle: s.toggleShuffle,
      cycleRepeat:   s.cycleRepeat,
    }))
  );

  /* The position ticker used to live here. It does not any more - see
     hooks/usePositionTicker, which Layout owns. Two PlayerBars on screen at
     once meant two intervals and a playhead running at double speed. */

  const barContainerRef = useRef<HTMLDivElement>(null);
  const [layoutMode, setLayoutMode] = useState({ showSecondaryControls: true, showInlineVolume: true });

  useEffect(() => {
    const el = barContainerRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const w = entry.contentRect.width;
      const sec = w >= 560;
      const vol = w >= 660;
      setLayoutMode((prev) => {
        if (prev.showSecondaryControls === sec && prev.showInlineVolume === vol) {
          return prev;
        }
        return { showSecondaryControls: sec, showInlineVolume: vol };
      });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const { showSecondaryControls, showInlineVolume } = layoutMode;

  // Volume controls & popup
  const [volPopupOpen, setVolPopupOpen] = useState(false);
  const volPopupRef = useRef<HTMLDivElement>(null);
  const volBtnRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!volPopupOpen) return;
    function handleClickOutside(e: MouseEvent) {
      if (
        volPopupRef.current &&
        !volPopupRef.current.contains(e.target as Node) &&
        volBtnRef.current &&
        !volBtnRef.current.contains(e.target as Node)
      ) {
        setVolPopupOpen(false);
      }
    }
    function handleKey(e: KeyboardEvent) {
      if (e.key !== "Escape") return;
      e.stopPropagation();
      setVolPopupOpen(false);
      volBtnRef.current?.querySelector("button")?.focus();
    }
    window.addEventListener("mousedown", handleClickOutside);
    // on document, not window: stopPropagation here keeps Escape from also
    // reaching window-level handlers (Immersive closes on Escape)
    document.addEventListener("keydown", handleKey);
    return () => {
      window.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKey);
    };
  }, [volPopupOpen]);

  useEffect(() => {
    if (showInlineVolume && volPopupOpen) {
      setVolPopupOpen(false);
    }
  }, [showInlineVolume, volPopupOpen]);

  const volDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  function handleVolumeChange(e: React.ChangeEvent<HTMLInputElement>) {
    const v = Number(e.target.value);
    storeSetVolume(v);
    if (volDebounce.current) clearTimeout(volDebounce.current);
    volDebounce.current = setTimeout(() => {
      if (usePlayerStore.getState().isRemotePlayback) {
        remoteSetVolume(v).catch(() => {});
      } else {
        apiSetVolume(v).catch(() => {});
      }
    }, 80);
  }

  function handleMuteToggle() {
    const next = !muted;
    storeSetMuted(next);
    if (usePlayerStore.getState().isRemotePlayback) {
      remoteSetVolume(next ? 0 : volume).catch(() => {});
    } else {
      apiSetMuted(next).catch(() => {});
    }
  }

  function handleVolumeButtonClick() {
    if (showInlineVolume) {
      handleMuteToggle();
    } else {
      setVolPopupOpen((v) => !v);
    }
  }

  const VolumeIcon = (muted || volume === 0) ? VolumeX : volume < 50 ? Volume1 : Volume2;
  const RepeatIcon = repeat === "one" ? Repeat1 : Repeat;
  const repeatLabel =
    repeat === "one" ? "Repeat current track"
    : repeat === "all" ? "Repeat all"
    : "Repeat off";

  return (
    <div
      style={{
        /* Docked in the app card it pins itself to the bottom. In the immersive
           view it is a block inside the right-hand column instead, so that it
           and the lyrics card get their width from the same parent and can
           never drift out of alignment. */
        position: immersive ? "relative" : "absolute",
        bottom: immersive ? undefined : 12,
        left: immersive ? undefined : 0,
        right: immersive ? undefined : 0,
        width: immersive ? "100%" : undefined,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        pointerEvents: "none",
        zIndex: 40,
      }}
    >
      {/* Remote playback banner */}
      <AnimatePresence>
        {isRemotePlayback && activeDevice && (
          /* No height animation: this column is pinned to the bottom, so the
             banner grows upward from the dock without pushing anything - it
             only needs to rise and fade in, which is compositor-only work. */
          <motion.button
            type="button"
            initial={{ opacity: 0, y: 8, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 6, scale: 0.98, transition: { duration: 0.14, ease: EASE_OUT } }}
            transition={{ duration: 0.22, ease: EASE_OUT }}
            whileTap={PRESS}
            onClick={toggleDevices}
            data-devices-trigger="true"
            aria-label={`Listening on ${activeDevice.name}. Change device`}
            className="glass-solid-fallback focus-ring t-caption"
            style={{
              pointerEvents: "auto",
              height: 28,
              marginBottom: 6,
              font: "inherit",
              background: "rgba(22, 22, 26, 0.90)",
              backdropFilter: "blur(20px)",
              WebkitBackdropFilter: "blur(20px)",
              border: "1px solid var(--color-glass-border)",
              borderRadius: 20,
              boxShadow: "0 8px 24px rgba(0, 0, 0, 0.4)",
              color: "var(--color-text-hi)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: 8,
              padding: "0 14px",
              fontSize: 11.5,
              fontWeight: 500,
              lineHeight: 1,
              cursor: "pointer",
              userSelect: "none",
              overflow: "hidden",
            }}
          >
            <Devices size={13} strokeWidth={2} active style={{ color: "var(--color-text-dim)" }} />
            <span>Listening on <strong style={{ fontWeight: 600 }}>{activeDevice.name}</strong></span>
            <span style={{ fontSize: 10.5, color: "var(--color-text-dim)", textDecoration: "underline", marginLeft: 4 }}>Change</span>
          </motion.button>
        )}
      </AnimatePresence>

      {/* Dock Container: Scaled down from full page width to a tailored floating island */}
      <div
        ref={barContainerRef}
        className={immersive ? undefined : "glass-solid-fallback"}
        style={{
          pointerEvents: "auto",
          width: immersive ? "100%" : "min(740px, calc(100% - 32px))",
          height: 74,
          /* Over the immersive backdrop: the lyrics card's exact shade, and no
             backdrop-filter. Dropping it is not only for the match - a
             backdrop-filter's input is whatever is behind it, and behind this
             the artwork is drifting, so it could never be cached and re-ran a
             24px gaussian under the bar on every frame. It was the last
             per-frame blur left on the page. Docked in the app card the bar
             sits over scrolling content instead, where it has to stay opaque
             to be readable, so that path is untouched. */
          background: immersive
            ? "linear-gradient(180deg, rgba(10, 8, 14, 0.30) 0%, rgba(10, 8, 14, 0.42) 100%)"
            : "var(--color-dock-bg, rgba(20, 20, 24, 0.88))",
          backdropFilter: immersive ? undefined : "blur(24px) saturate(160%)",
          WebkitBackdropFilter: immersive ? undefined : "blur(24px) saturate(160%)",
          borderRadius: 18,
          border: immersive
            ? "1px solid rgba(255, 255, 255, 0.11)"
            : "1px solid var(--color-dock-border, rgba(255, 255, 255, 0.09))",
          boxShadow: immersive
            ? "inset 0 1px 0 0 rgba(255, 255, 255, 0.09), 0 20px 50px rgba(0, 0, 0, 0.30)"
            : "inset 0 1px 0 0 rgba(255, 255, 255, 0.08), 0 12px 32px rgba(0, 0, 0, 0.45)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 18px",
          gap: 14,
          flexShrink: 0,
        }}
      >
        {/* Left: Playback Controls */}
        <div style={{ display: "flex", alignItems: "center", gap: "clamp(4px, 0.8vw, 8px)", flexShrink: 0 }}>
          {showSecondaryControls && (
            <Tooltip label={shuffle ? "Shuffle on" : "Shuffle off"}>
              <IconBtn active={shuffle} pressed={shuffle} ariaLabel="Shuffle" onClick={toggleShuffle}>
                <Shuffle size={15} strokeWidth={1.75} />
              </IconBtn>
            </Tooltip>
          )}

          <Tooltip label="Previous">
            <IconBtn ariaLabel="Previous" onClick={handlePrev}>
              <SkipBack size={15} strokeWidth={1.75} fill="currentColor" />
            </IconBtn>
          </Tooltip>

          <Tooltip label={isPlaying ? "Pause" : "Play"}>
            <PlayPauseButton
              isPlaying={isPlaying}
              onClick={togglePlay}
            />
          </Tooltip>

          <Tooltip label="Next">
            <IconBtn ariaLabel="Next" onClick={handleNext}>
              <SkipForward size={15} strokeWidth={1.75} fill="currentColor" />
            </IconBtn>
          </Tooltip>

          {showSecondaryControls && (
            <Tooltip label={repeatLabel}>
              <IconBtn active={repeat !== "none"} pressed={repeat !== "none"} ariaLabel={repeatLabel} onClick={cycleRepeat}>
                <RepeatIcon size={15} strokeWidth={1.75} />
              </IconBtn>
            </Tooltip>
          )}
        </div>

        {/* Center: Bigger Album Art + Bigger Titles + Progress Line Directly Underneath */}
        <div
          style={{
            flex: "1 1 auto",
            minWidth: immersive ? 140 : 160,
            maxWidth: immersive ? 360 : 420,
            display: "flex",
            alignItems: "center",
            gap: 12,
            overflow: "hidden",
          }}
        >
          {/* Bigger Album Art (44x44px). The immersive view hides it: the
              sleeve is already filling the left of the screen behind this bar,
              and a 44px copy of it next to the title is just noise. */}
          {!immersive && (
          <div
            className="group pressable"
            role={currentTrack ? "button" : undefined}
            tabIndex={currentTrack ? 0 : undefined}
            aria-label={currentTrack ? "Open full-screen player" : undefined}
            style={{
              position: "relative",
              width: 44,
              height: 44,
              flexShrink: 0,
              borderRadius: 8,
              overflow: "hidden",
              cursor: currentTrack ? "pointer" : "default",
              boxShadow: "0 2px 8px rgba(0, 0, 0, 0.4)",
            }}
            onClick={() => { if (currentTrack) setImmersiveOpen(true); }}
            onKeyDown={(e) => {
              if (currentTrack && (e.key === "Enter" || e.key === " ")) {
                e.preventDefault();
                setImmersiveOpen(true);
              }
            }}
            title={currentTrack ? "Open full-screen player" : undefined}
          >
            <AnimatePresence initial={false}>
              <motion.div
                key={currentTrack?.album?.image_url ?? currentTrack?.id ?? "none"}
                initial={{ opacity: 0, filter: "blur(10px)", scale: 1.06 }}
                animate={{ opacity: 1, filter: "blur(0px)", scale: 1 }}
                exit={{ opacity: 0, filter: "blur(8px)", scale: 1.04 }}
                transition={{ duration: 0.3, ease: EASE_OUT }}
                style={{ position: "absolute", inset: 0 }}
              >
                <CoverArt url={currentTrack?.album?.image_url ?? null} alt={currentTrack?.name ?? ""} size={44} />
              </motion.div>
            </AnimatePresence>
            {currentTrack && (
              <div
                className="queue-btn"
                style={{
                  position: "absolute",
                  inset: 0,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  background: "rgba(0,0,0,0.45)",
                  transition: "opacity 0.12s",
                }}
              >
                <Maximize2 size={14} strokeWidth={1.8} style={{ color: "#fff" }} />
              </div>
            )}
          </div>

          )}

          {/* Titles + Expanding Progress Line Directly Below */}
          <div style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", justifyContent: "center", gap: 5, overflow: "hidden" }}>
            <div style={{ minWidth: 0, overflow: "hidden" }}>
              <p style={{ margin: 0, fontSize: 14.5, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: currentTrack ? "var(--color-text-hi)" : "var(--color-text-dim)", lineHeight: 1.25, letterSpacing: "-0.01em" }}>
                {currentTrack?.album?.id ? (
                  <Link
                    to={`/album/${currentTrack.album.id}`}
                    onClick={() => setImmersiveOpen(false)}
                    className="pb-link"
                    style={{ color: "inherit" }}
                    // hover only prefetches now; the underline is CSS
                    onPointerEnter={() => prefetchAlbum(qc, currentTrack.album?.id)}
                    title={`Go to album: ${currentTrack.album.name}`}
                  >
                    {currentTrack.name}
                  </Link>
                ) : (
                  currentTrack?.name ?? "Not playing"
                )}
              </p>
              <p className="t-caption" style={{ margin: "2px 0 0", fontSize: 12.5, fontWeight: 400, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "var(--color-text-dim)", lineHeight: 1.2 }}>
                {currentTrack ? (
                  currentTrack.artists.map((a, i) => (
                    <span key={a.id || i}>
                      {i > 0 && ", "}
                      {a.id ? (
                        <Link
                          to={`/artist/${a.id}`}
                          onClick={() => setImmersiveOpen(false)}
                          className="pb-link"
                          style={{ color: "inherit" }}
                          onPointerEnter={() => prefetchArtist(qc, a.id)}
                        >
                          {a.name}
                        </Link>
                      ) : (
                        <span>{a.name}</span>
                      )}
                    </span>
                  ))
                ) : (
                  ""
                )}
              </p>
            </div>

            {/* Progress line under album title: isolated leaf scrubber so PlayerBar does not re-render every second */}
            <PlayerScrubber durationMs={durationMs} doSeek={doSeek} showTimes={showSecondaryControls} />
          </div>
        </div>

        {/* Right: Volume with responsive slider */}
        <div style={{ display: "flex", alignItems: "center", gap: 8, justifyContent: "flex-end", position: "relative", flexShrink: 0 }}>
          <Tooltip label={showInlineVolume ? (muted ? "Unmute" : `Mute (${volume}%)`) : (volPopupOpen ? "Close volume" : `Volume (${volume}%)`)}>
            <span ref={volBtnRef}>
              <IconBtn
                active={!showInlineVolume && volPopupOpen}
                ariaLabel={showInlineVolume ? (muted ? "Unmute" : "Mute") : "Volume"}
                onClick={handleVolumeButtonClick}
              >
                <VolumeIcon size={16} strokeWidth={1.75} />
              </IconBtn>
            </span>
          </Tooltip>
          {showInlineVolume && (
            <Tooltip label={muted ? "Muted" : `Volume ${volume}%`}>
              <input
                className="vol"
                aria-label="Volume"
                type="range" min={0} max={100}
                value={muted ? 0 : volume}
                onChange={handleVolumeChange}
                onMouseDown={() => { if (muted) { storeSetMuted(false); apiSetMuted(false).catch(() => {}); } }}
                style={{ width: "clamp(75px, 10vw, 110px)", ["--vol" as string]: `${muted ? 0 : volume}%` } as React.CSSProperties}
              />
            </Tooltip>
          )}

          {/* Floating volume popup for compact widths */}
          <AnimatePresence>
            {!showInlineVolume && volPopupOpen && (
              <motion.div
                ref={volPopupRef}
                className="glass-solid-fallback"
                role="dialog"
                aria-label="Volume"
                // rises out of the volume button, which sits under its
                // bottom-right corner
                initial={{ opacity: 0, y: 6, scale: 0.96 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 4, scale: 0.97, transition: { duration: 0.12, ease: EASE_OUT } }}
                transition={{ duration: 0.16, ease: EASE_OUT }}
                style={{
                  transformOrigin: "bottom right",
                  position: "absolute",
                  bottom: 44,
                  right: 0,
                  zIndex: 60,
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  padding: "8px 12px",
                  borderRadius: 12,
                  background: "rgba(19, 19, 22, 0.96)",
                  backdropFilter: "blur(24px) saturate(1.4)",
                  WebkitBackdropFilter: "blur(24px) saturate(1.4)",
                  border: "1px solid rgba(255, 255, 255, 0.12)",
                  boxShadow: "0 12px 32px rgba(0, 0, 0, 0.6), inset 0 1px 0 rgba(255, 255, 255, 0.08)",
                  pointerEvents: "auto",
                  whiteSpace: "nowrap",
                }}
              >
                <button
                  onClick={handleMuteToggle}
                  className="btn-icon"
                  aria-label={muted ? "Unmute" : "Mute"}
                  style={{
                    width: 24,
                    height: 24,
                    borderRadius: 6,
                    color: muted ? "var(--color-accent)" : "var(--color-text-hi)",
                  }}
                  title={muted ? "Unmute" : "Mute"}
                >
                  <VolumeIcon size={16} strokeWidth={2} fill="currentColor" />
                </button>
                <input
                  className="vol"
                  aria-label="Volume"
                  autoFocus
                  type="range"
                  min={0}
                  max={100}
                  value={muted ? 0 : volume}
                  onChange={handleVolumeChange}
                  onMouseDown={() => {
                    if (muted) {
                      storeSetMuted(false);
                      apiSetMuted(false).catch(() => {});
                    }
                  }}
                  style={{
                    width: 100,
                    ["--vol" as string]: `${muted ? 0 : volume}%`,
                  } as React.CSSProperties}
                />
                <span
                  className="tnum t-caption"
                  style={{
                    fontSize: 11,
                    fontWeight: 600,
                    color: "rgba(255, 255, 255, 0.7)",
                    minWidth: 32,
                    textAlign: "right",
                  }}
                >
                  {muted ? "0%" : `${volume}%`}
                </span>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
