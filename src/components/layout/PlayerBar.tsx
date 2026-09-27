import { useEffect, useRef, useState, memo } from "react";
import { Link } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence, animate, useMotionValue, useTransform } from "framer-motion";
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
import { gpuLayer, zTransform, EASE_OUT, PRESS } from "../../lib/motion";

/* Transport buttons: they lift a little under the pointer and squash on the
   press, on an underdamped spring so they land with a small bounce - the feel
   these had before the flat 0.96 press replaced it. Colour hover is CSS
   (.pb-btn in styles/layout.css); the old onMouseEnter wrote node.style, which
   any re-render wiped. */
const BTN_SPRING = { type: "spring" as const, stiffness: 420, damping: 24 };
const PLAY_SPRING = { type: "spring" as const, stiffness: 440, damping: 22 };
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
      whileHover={disabled ? undefined : { scale: 1.1 }}
      whileTap={disabled ? undefined : { scale: 0.92 }}
      transition={BTN_SPRING}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        position:       "relative",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        flexShrink:     0,
        width:          large ? 32 : 28,
        height:         large ? 32 : 28,
        borderRadius:   "50%",
        border:         "none",
        // left unset when not large, so the CSS hover plate can show
        background:     large ? "rgba(242,238,233,0.90)" : undefined,
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
      {/* toggles (shuffle, repeat) mark "on" with a dot as well as the
          accent, so the state doesn't rest on colour alone */}
      {pressed !== undefined && <span aria-hidden className="pb-on-dot" data-on={active || undefined} />}
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
      className="pb-play focus-ring"
      whileHover={{ scale: 1.14 }}
      whileTap={{ scale: 0.88 }}
      transition={PLAY_SPRING}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        position: "relative",
        display: "flex", alignItems: "center", justifyContent: "center",
        width: 38, height: 38, flexShrink: 0,
        borderRadius: "50%",
        border: "none",
        color: "var(--color-text-hi, #ffffff)",
        cursor: "pointer",
      }}
    >
      <AnimatePresence initial={false}>
        <motion.span
          key={isPlaying ? "pause" : "play"}
          initial={{ opacity: 0, scale: 0.5, filter: "blur(6px)" }}
          animate={{
            opacity: 1, scale: 1, filter: "blur(0px)",
            transition: {
              scale: { type: "spring", stiffness: 520, damping: 20 },
              opacity: { duration: 0.16, ease: EASE_OUT },
              filter: { duration: 0.2, ease: EASE_OUT },
            },
          }}
          exit={{ opacity: 0, scale: 0.5, filter: "blur(6px)", transition: { duration: 0.14, ease: EASE_OUT } }}
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

/* One line of dock text that ellipsises at rest and, while `active` (the
   pointer is over the titles), scrolls to show the rest of a name too long to
   fit - "The Mountain (feat. Dennis Hopper, Ajay…" could not be read at all.
   Linear, as constant motion should be, with a pause at each end, and at a
   reading pace set by how far it has to travel rather than a fixed time. Only
   hover starts it: a dock that moved on its own would pull the eye away from
   the page. Reduced motion keeps the ellipsis. */
function Marquee({
  active, style, className, children,
}: {
  active: boolean;
  style?: React.CSSProperties;
  className?: string;
  children: React.ReactNode;
}) {
  const outer = useRef<HTMLParagraphElement>(null);
  const [shift, setShift] = useState(0);

  useEffect(() => {
    if (!active) { setShift(0); return; }
    if (window.matchMedia?.("(prefers-reduced-motion: reduce)").matches) return;
    // measured on the clipping <p>: its scrollWidth includes what overflows,
    // while the inline span inside reports 0
    const o = outer.current;
    if (!o) return;
    const over = o.scrollWidth - o.clientWidth;
    setShift(over > 2 ? over : 0);
  }, [active, children]);

  const running = shift > 0;
  return (
    <p
      ref={outer}
      className={[className, running ? "pb-marquee" : ""].filter(Boolean).join(" ") || undefined}
      style={{ ...style, overflow: "hidden", whiteSpace: "nowrap", textOverflow: running ? "clip" : "ellipsis" }}
    >
      <span
        className={running ? "pb-marquee-run" : undefined}
        style={
          running
            ? ({
                ["--shift" as string]: `${-shift}px`,
                // ~35px/s, and never quicker than 3s end to end
                ["--dur" as string]: `${Math.max(3, shift / 35 + 1.6).toFixed(2)}s`,
              } as React.CSSProperties)
            : undefined
        }
      >
        {children}
      </span>
    </p>
  );
}

/* The volume popover, for docks too narrow to hold the inline slider.

   A vertical capsule, the way iOS and Control Center do volume: the fill rises
   from the bottom and follows the pointer 1:1 while you drag (pointer capture,
   so it keeps tracking off the capsule). Pull past either end and it
   rubber-bands - the capsule stretches with growing resistance instead of
   hitting a wall - then springs back when you let go. The icon at its foot
   mutes; the wheel and the keyboard both work. It used to be a small
   horizontal range input with a separate mute button and a number. */
const VOL_TRACK_H = 132;
const VOL_STEP = 5;
const clampVol = (v: number) => Math.max(0, Math.min(100, Math.round(v)));

// Apple's rubber-band: the further past the edge, the less it follows
function rubberband(over: number, dim: number, c = 0.55) {
  const a = Math.abs(over);
  return (Math.sign(over) * a * dim * c) / (dim + c * a);
}

function VolumePopover({
  popRef, volume, muted, onVolume, onToggleMute,
}: {
  popRef: React.RefObject<HTMLDivElement | null>;
  volume: number;
  muted: boolean;
  onVolume: (v: number) => void;
  onToggleMute: () => void;
}) {
  const capsuleRef = useRef<HTMLDivElement>(null);
  const rectRef = useRef<DOMRect | null>(null);
  const [dragging, setDragging] = useState(false);
  const shown = muted ? 0 : volume;
  // listeners below read the latest values without re-binding
  const live = useRef({ shown, onVolume });
  live.current = { shown, onVolume };

  // signed overshoot in px: + past the top, - past the bottom
  const stretch = useMotionValue(0);
  const scaleY = useTransform(stretch, (x) => 1 + Math.abs(x) / VOL_TRACK_H);
  // stretch away from the edge being pulled
  const originY = useTransform(stretch, (x) => (x >= 0 ? 1 : 0));

  useEffect(() => {
    capsuleRef.current?.focus({ preventScroll: true });
  }, []);

  useEffect(() => {
    const el = popRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) return; // that's zoom
      e.preventDefault();
      const { shown: v, onVolume: set } = live.current;
      set(clampVol(v + (e.deltaY < 0 ? VOL_STEP : -VOL_STEP)));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, [popRef]);

  function track(clientY: number) {
    const r = rectRef.current;
    if (!r) return;
    const raw = ((r.bottom - clientY) / r.height) * 100;
    const v = clampVol(raw);
    if (v !== live.current.shown) live.current.onVolume(v);
    const overPct = raw > 100 ? raw - 100 : raw < 0 ? raw : 0;
    stretch.set(rubberband((overPct / 100) * VOL_TRACK_H, VOL_TRACK_H));
  }

  function onPointerDown(e: React.PointerEvent<HTMLDivElement>) {
    if (e.button !== 0) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    // measured once, unstretched, so the stretch can't feed back into the value
    rectRef.current = e.currentTarget.getBoundingClientRect();
    stretch.stop();
    setDragging(true);
    track(e.clientY);
  }

  function release() {
    if (!rectRef.current) return;
    rectRef.current = null;
    setDragging(false);
    // released from a pull: it comes back with a little of that energy
    animate(stretch, 0, { type: "spring", stiffness: 520, damping: 28 });
  }

  function onKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    const v = live.current.shown;
    let next: number | null = null;
    if (e.key === "ArrowUp" || e.key === "ArrowRight") next = v + VOL_STEP;
    else if (e.key === "ArrowDown" || e.key === "ArrowLeft") next = v - VOL_STEP;
    else if (e.key === "PageUp") next = v + 10;
    else if (e.key === "PageDown") next = v - 10;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = 100;
    else if (e.key.toLowerCase() === "m") {
      e.preventDefault();
      onToggleMute();
      return;
    }
    if (next === null) return;
    e.preventDefault();
    onVolume(clampVol(next));
  }

  const Icon = shown === 0 ? VolumeX : shown < 50 ? Volume1 : Volume2;
  // the icon sits in the bottom of the capsule; once the fill covers it, it
  // goes dark so it stays legible on the white
  const iconOnFill = shown >= 16;

  return (
    <motion.div
      ref={popRef}
      className="glass-solid-fallback"
      role="dialog"
      aria-label="Volume"
      // rises out of the volume button beneath it
      initial={{ opacity: 0, scale: 0.88, y: 10, filter: "blur(6px)" }}
      animate={{
        opacity: 1, scale: 1, y: 0, filter: "blur(0px)",
        transition: {
          scale: { type: "spring", stiffness: 480, damping: 26 },
          y: { type: "spring", stiffness: 480, damping: 26 },
          opacity: { duration: 0.14, ease: EASE_OUT },
          filter: { duration: 0.18, ease: EASE_OUT },
        },
      }}
      // leaves the way it came, and quicker than it arrived
      exit={{ opacity: 0, scale: 0.94, y: 6, filter: "blur(4px)", transition: { duration: 0.12, ease: EASE_OUT } }}
      style={{
        position: "absolute",
        bottom: "calc(100% + 30px)",
        left: "50%",
        x: "-50%",
        transformOrigin: "bottom center",
        zIndex: 60,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 7,
        padding: "8px 5px 5px",
        borderRadius: 17,
        background: "var(--color-popover)",
        backdropFilter: "blur(24px) saturate(1.4)",
        WebkitBackdropFilter: "blur(24px) saturate(1.4)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        boxShadow: "0 16px 40px rgba(0, 0, 0, 0.55), inset 0 1px 0 rgba(255, 255, 255, 0.08)",
        pointerEvents: "auto",
      }}
    >
      <span
        className="tnum t-caption"
        aria-hidden
        style={{ fontSize: 10, fontWeight: 600, lineHeight: 1, color: "rgba(255, 255, 255, 0.72)", textAlign: "center", whiteSpace: "nowrap" }}
      >
        {muted ? "Muted" : `${volume}%`}
      </span>

      <motion.div
        ref={capsuleRef}
        role="slider"
        tabIndex={0}
        aria-label="Volume"
        aria-orientation="vertical"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={shown}
        aria-valuetext={muted ? "Muted" : `${volume}%`}
        className="focus-ring"
        onPointerDown={onPointerDown}
        onPointerMove={(e) => { if (rectRef.current) track(e.clientY); }}
        onPointerUp={release}
        onPointerCancel={release}
        onLostPointerCapture={release}
        onKeyDown={onKeyDown}
        style={{
          position: "relative",
          // slim: a level, not a panel
          width: 24,
          height: VOL_TRACK_H,
          borderRadius: 12,
          overflow: "hidden",
          background: "rgba(255, 255, 255, 0.12)",
          cursor: dragging ? "grabbing" : "pointer",
          touchAction: "none",
          scaleY,
          originY,
        }}
      >
        <div
          aria-hidden
          style={{
            position: "absolute",
            inset: 0,
            background: "rgba(255, 255, 255, 0.92)",
            transformOrigin: "bottom",
            transform: `scaleY(${shown / 100})`,
            // 1:1 under the pointer; a short glide for wheel and keyboard steps
            transition: dragging ? "none" : "transform 0.18s cubic-bezier(0.23, 1, 0.32, 1)",
          }}
        />
        <button
          type="button"
          aria-label={muted ? "Unmute" : "Mute"}
          // its own target: a press here mutes rather than setting a level
          onPointerDown={(e) => e.stopPropagation()}
          onClick={onToggleMute}
          className="pb-vol-mute"
          style={{
            position: "absolute",
            left: 0,
            right: 0,
            bottom: 0,
            height: 28,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            border: "none",
            background: "transparent",
            padding: 0,
            cursor: "pointer",
            color: iconOnFill ? "rgba(0, 0, 0, 0.72)" : "rgba(255, 255, 255, 0.85)",
            transition: "color 0.15s ease",
          }}
        >
          <Icon size={13} strokeWidth={2.1} />
        </button>
      </motion.div>
    </motion.div>
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
    color: "rgba(255, 255, 255, 0.5)",
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
            // ~3px at rest (was ~2): early in a song the played part was a speck
            transform: isLineExpanded ? "scaleY(1)" : "scaleY(0.6)",
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
                background: "var(--color-popover)",
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
  // hovering the title block scrolls any name too long to show
  const [titlesHovered, setTitlesHovered] = useState(false);
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

  // any change of level, from the slider, the popover, the wheel or keys.
  // raising it from muted unmutes, as turning a knob up would
  function applyVolume(v: number) {
    storeSetVolume(v);
    const st = usePlayerStore.getState();
    if (st.muted && v > 0) {
      storeSetMuted(false);
      if (!st.isRemotePlayback) apiSetMuted(false).catch(() => {});
    }
    if (volDebounce.current) clearTimeout(volDebounce.current);
    volDebounce.current = setTimeout(() => {
      if (usePlayerStore.getState().isRemotePlayback) {
        remoteSetVolume(v).catch(() => {});
      } else {
        apiSetVolume(v).catch(() => {});
      }
    }, 80);
  }

  function handleVolumeChange(e: React.ChangeEvent<HTMLInputElement>) {
    applyVolume(Number(e.target.value));
  }

  // the wheel over the volume button nudges the level, popover open or not
  const applyVolumeRef = useRef(applyVolume);
  applyVolumeRef.current = applyVolume;
  useEffect(() => {
    const el = volBtnRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) return; // that's zoom
      e.preventDefault();
      const st = usePlayerStore.getState();
      const cur = st.muted ? 0 : st.volume;
      applyVolumeRef.current(clampVol(cur + (e.deltaY < 0 ? VOL_STEP : -VOL_STEP)));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

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
            {/* a hairline inside the edge, so a dark sleeve keeps its shape
                against the dark dock */}
            <div
              aria-hidden
              style={{
                position: "absolute",
                inset: 0,
                borderRadius: "inherit",
                boxShadow: "inset 0 0 0 1px rgba(255, 255, 255, 0.08)",
                pointerEvents: "none",
              }}
            />
          </div>

          )}

          {/* Titles + Expanding Progress Line Directly Below */}
          <div
            style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", justifyContent: "center", gap: 5, overflow: "hidden" }}
            onPointerEnter={() => setTitlesHovered(true)}
            onPointerLeave={() => setTitlesHovered(false)}
          >
            <div style={{ minWidth: 0, overflow: "hidden" }}>
              <Marquee active={titlesHovered} style={{ margin: 0, fontSize: 14.5, fontWeight: 600, color: currentTrack ? "var(--color-text-hi)" : "var(--color-text-dim)", lineHeight: 1.25, letterSpacing: "-0.01em" }}>
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
              </Marquee>
              <Marquee active={titlesHovered} className="t-caption" style={{ margin: "2px 0 0", fontSize: 12.5, fontWeight: 400, color: "var(--color-text-dim)", lineHeight: 1.2 }}>
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
              </Marquee>
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

          {/* the volume popover, for docks too narrow for the inline slider.
              this group holds only the button then, so left: 50% centres the
              popover over it */}
          <AnimatePresence>
            {!showInlineVolume && volPopupOpen && (
              <VolumePopover
                popRef={volPopupRef}
                volume={volume}
                muted={muted}
                onVolume={applyVolume}
                onToggleMute={handleMuteToggle}
              />
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
