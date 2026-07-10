import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Play, Pause, SkipBack, SkipForward,
  Shuffle, Repeat, Repeat1,
  Volume2, Volume1, VolumeX,
  ListMusic, Captions, Maximize2,
} from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import {
  pausePlayback, resumeOrPlay, seekPlayback, playTrack,
  setVolume as apiSetVolume, setMuted as apiSetMuted,
} from "../../api/playback";
import { CoverArt } from "../ui/CoverArt";
import { Tooltip } from "../ui/Tooltip";
import { fmtMs } from "../../utils/fmt";
import { startRadio } from "../../utils/radio";
import { gpuLayer, zTransform } from "../../lib/motion";

// icon button

function IconBtn({
  children, onClick, active, large, title, disabled,
}: {
  children:  React.ReactNode;
  onClick?:  () => void;
  active?:   boolean;
  large?:    boolean;
  title?:    string;
  disabled?: boolean;
}) {
  return (
    <motion.button
      onClick={onClick}
      title={title}
      disabled={disabled}
      whileHover={disabled ? {} : { scale: 1.10 }}
      whileTap={disabled   ? {} : { scale: 0.92 }}
      transition={{ type: "spring", stiffness: 400, damping: 25 }}
      /* TODO LOGIC - keep a permanent GPU layer (translateZ(0) always there) so scaling
       never prodemotes the layer - that promote/demote rounding is the
       1px "teleport" you get before/after the animation */

/* IMPLEMETED  DOMNE */
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
              : "var(--color-text)",
        cursor:         disabled ? "default" : "pointer",
      }}
    >
      {children}
    </motion.button>
  );
}

// play / pause (bare, (and also) morphing glyph, to be reused)

function PlayPauseButton({
  isPlaying, onClick,
}: {
  isPlaying: boolean; onClick: () => void;
}) {
  return (
    <motion.button
      onClick={onClick}
      whileHover={{ scale: 1.08 }}
      whileTap={{ scale: 0.86 }}
      transition={{ type: "spring", stiffness: 420, damping: 20 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        position: "relative",
        display: "flex", alignItems: "center", justifyContent: "center",
        width: 34, height: 34, flexShrink: 0,
        border: "none", background: "transparent",
        color: "var(--color-text-hi)", cursor: "pointer",
      }}
    >
      {/* two glyphs crossfade through a slight blur so the swap erads as one
          morph instead of a hard cut, pinned to inset:0 and centred so the
          swap/scale cant shift the glyphs position (Removes teleport) */}
      <AnimatePresence initial={false}>
        <motion.span
          key={isPlaying ? "pause" : "play"}
          initial={{ opacity: 0, scale: 0.55, filter: "blur(5px)" }}
          animate={{ opacity: 1, scale: 1,    filter: "blur(0px)" }}
          exit={{    opacity: 0, scale: 0.55, filter: "blur(5px)" }}
          transition={{ duration: 0.19, ease: [0.23, 1, 0.32, 1] }}
          style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}
        >
          {isPlaying
            ? <Pause size={22} strokeWidth={2.4} fill="currentColor" />
            : <Play  size={22} strokeWidth={2.4} fill="currentColor" style={{ marginLeft: 2 }} />
          }
        </motion.span>
      </AnimatePresence>
    </motion.button>
  );
}

// progress bar

function ProgressBar({
  positionMs, durationMs, onSeek,
}: {
  positionMs: number; durationMs: number; onSeek: (ms: number) => void;
}) {
  // hover scrub preview + drag-scrub. while dragging the bar follows the cursor until release, when we commit the seek.


  // (AND shows the wouldbe time)
  const [hoverFrac, setHoverFrac] = useState<number | null>(null);
  const [dragFrac,  setDragFrac]  = useState<number | null>(null);
  const barRef = useRef<HTMLDivElement>(null);

  const dragging  = dragFrac != null;
  // The fill would show drag position ?????
  const shownFrac = dragging ? dragFrac! : (durationMs > 0 ? Math.min(positionMs / durationMs, 1) : 0);
  const pct       = shownFrac * 100;
  const showBubble = (hoverFrac != null || dragging) && durationMs > 0;
  const bubbleFrac = dragging ? dragFrac! : (hoverFrac ?? 0);

  function fracFromClientX(clientX: number) {
    const el = barRef.current;
    if (!el) return 0;
    const rect = el.getBoundingClientRect();
    return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  }
  function onPointerDown(e: React.PointerEvent<HTMLDivElement>) {
    if (!durationMs) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    const f = fracFromClientX(e.clientX);
    setDragFrac(f);
    setHoverFrac(f);
  }
  function onPointerMove(e: React.PointerEvent<HTMLDivElement>) {
    if (!durationMs) return;
    const f = fracFromClientX(e.clientX);
    setHoverFrac(f);
    if (dragging) setDragFrac(f);
  }
  function onPointerUp(e: React.PointerEvent<HTMLDivElement>) {
    if (dragging) {
      onSeek(Math.floor(dragFrac! * durationMs));
      setDragFrac(null);
    }
    try { e.currentTarget.releasePointerCapture(e.pointerId); } catch { /* noop */ }
  }

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, width: "100%", maxWidth: 480 }}>
      <span style={{ fontSize: 11, fontWeight: 400, color: "var(--color-text-dim)", flexShrink: 0, fontVariantNumeric: "tabular-nums" }}>
        {fmtMs(dragging ? Math.floor(dragFrac! * durationMs) : positionMs)}
      </span>
      {/* tall transparent hitarea so it's easy to grab -- 4px track sits centred */}
      <div
        ref={barRef}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={() => { if (!dragging) setHoverFrac(null); }}
        className="group"
        style={{ flex: 1, height: 16, position: "relative", display: "flex", alignItems: "center", cursor: durationMs ? "pointer" : "default", touchAction: "none" }}
      >
        <div style={{ position: "relative", width: "100%", height: 4, borderRadius: 99 }}>
          <div style={{ position: "absolute", inset: 0, background: "rgba(255,255,255,0.10)", borderRadius: 99 }} />
          <motion.div
            style={{ position: "absolute", inset: "0 auto 0 0", background: "rgba(242,238,233,0.80)", borderRadius: 99 }}
            animate={{ width: `${pct}%` }}
            transition={{ duration: dragging ? 0 : 0.16, ease: [0.23, 1, 0.32, 1] }}
          />
          {/* draggable thumb --- shows on hover/drag, sits at the fill end to */}
          <motion.div
            style={{
              position: "absolute", top: "50%", left: `${pct}%`,
              width: 12, height: 12, borderRadius: "50%",
              background: "#fff", boxShadow: "0 1px 4px rgba(0,0,0,0.5)",
              pointerEvents: "none",
            }}
            animate={{
              x: "-50%", y: "-50%",
              opacity: durationMs && (hoverFrac != null || dragging) ? 1 : 0,
              scale: dragging ? 1.15 : 1,
            }}
            transition={{ opacity: { duration: 0.12 }, scale: { duration: 0.12 }, left: { duration: dragging ? 0 : 0.16 } }}
          />
        </div>

        <AnimatePresence>
          {showBubble && (
            <motion.span
              initial={{ opacity: 0, y: 3 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 3 }}
              transition={{ duration: 0.12 }}
              style={{
                position: "absolute",
                left: `${bubbleFrac * 100}%`,
                bottom: "calc(100% + 4px)",
                transform: "translateX(-50%)",
                pointerEvents: "none",
                whiteSpace: "nowrap",
                padding: "3px 7px",
                borderRadius: 6,
                background: "rgba(26,26,30,0.96)",
                border: "1px solid var(--color-glass-border)",
                boxShadow: "0 6px 16px rgba(0,0,0,0.45)",
                fontSize: 11,
                fontWeight: 600,
                fontVariantNumeric: "tabular-nums",
                color: "var(--color-text-hi)",
              }}
            >
              {fmtMs(Math.floor(bubbleFrac * durationMs))}
            </motion.span>
          )}
        </AnimatePresence>
      </div>
      <span style={{ fontSize: 11, fontWeight: 400, color: "var(--color-text-dim)", flexShrink: 0, fontVariantNumeric: "tabular-nums" }}>
        {durationMs > 0 ? fmtMs(durationMs) : "0:00"}
      </span>
    </div>
  );
}

// playerbar

export function PlayerBar() {
  const queueOpen       = usePlayerStore((s) => s.queueOpen);
  const toggleQueue     = usePlayerStore((s) => s.toggleQueue);
  const lyricsOpen      = usePlayerStore((s) => s.lyricsOpen);
  const toggleLyrics    = usePlayerStore((s) => s.toggleLyrics);
  const isPlaying       = usePlayerStore((s) => s.isPlaying);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);
  const positionMs      = usePlayerStore((s) => s.positionMs);
  const durationMs      = usePlayerStore((s) => s.durationMs);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const incrementPos    = usePlayerStore((s) => s.incrementPos);
  const setPosition     = usePlayerStore((s) => s.setPosition);
  const volume          = usePlayerStore((s) => s.volume);
  const muted           = usePlayerStore((s) => s.muted);
  const storeSetVolume  = usePlayerStore((s) => s.setVolume);
  const storeSetMuted   = usePlayerStore((s) => s.setMuted);
  const setPlaying      = usePlayerStore((s) => s.setPlaying);
  const sessionReady    = usePlayerStore((s) => s.sessionReady);
  const setImmersiveOpen = usePlayerStore((s) => s.setImmersiveOpen);
  const { shuffle, repeat, advance, previous, toggleShuffle, cycleRepeat } = useQueueStore(
    useShallow((s) => ({
      shuffle: s.shuffle,
      repeat:  s.repeat,
      advance: s.advance,
      previous: s.previous,
      toggleShuffle: s.toggleShuffle,
      cycleRepeat:   s.cycleRepeat,
    }))
  );

  useEffect(() => {
    if (!isPlaying) return;
    const timer = setInterval(incrementPos, 1000);
    return () => clearInterval(timer);
  }, [isPlaying, incrementPos]);

  const volDebounce = useRef<ReturnType<typeof setTimeout> | null>(null);

  function handleNext() {
    const next = advance(currentTrack);
    if (next) { setCurrentTrack(next); playTrack(next.id).catch(() => {}); }
    else if (currentTrack) { startRadio(currentTrack); }  // queue dry -> radio
  }
  function doSeek(ms: number) {
    setPosition(ms);                 // move the bar now
    seekPlayback(ms).catch(() => {}); // and tell librespot
  }
  function handlePrev() {
    if (positionMs > 3000) {
      doSeek(0);
    } else {
      const prev = previous(currentTrack);
      if (prev) { setCurrentTrack(prev); playTrack(prev.id).catch(() => {}); }
    }
  }
  function handleVolumeChange(e: React.ChangeEvent<HTMLInputElement>) {
    const v = Number(e.target.value);
    storeSetVolume(v);
    if (volDebounce.current) clearTimeout(volDebounce.current);
    volDebounce.current = setTimeout(() => { apiSetVolume(v).catch(() => {}); }, 80);
  }
  function handleMuteToggle() {
    const next = !muted;
    storeSetMuted(next);
    apiSetMuted(next).catch(() => {});
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
        height:     72,
        background: "var(--color-player)",
        borderTop:  "1px solid var(--color-border)",
        display:    "flex",
        alignItems: "center",
        gap:        16,
        padding:    "0 16px",
        flexShrink: 0,
      }}
    >
      {/* left: track info */}
      <div style={{ width: "28%", minWidth: 0, display: "flex", alignItems: "center", gap: 10, overflow: "hidden" }}>
        {/* art swaps (eg on skip) ease in through a slight blur so the change isn't an abrupt cut */}
        <div
          className="group"
          style={{ position: "relative", width: 44, height: 44, flexShrink: 0, borderRadius: 6, overflow: "hidden", cursor: currentTrack ? "pointer" : "default" }}
          onClick={() => { if (currentTrack) setImmersiveOpen(true); }}
          title={currentTrack ? "Open immersive view" : undefined}
        >
          {/* crossfade on track change: old + new art coexist (default
              AnimatePresence mode) and dissolve through a soft blur so the swap
              eases and not cuts. separate in/out blur+scale curves read as
              a gentle focus and also pull. */}
          <AnimatePresence initial={false}>
            <motion.div
              key={currentTrack?.album?.image_url ?? currentTrack?.id ?? "none"}
              initial={{ opacity: 0, filter: "blur(16px)", scale: 1.08 }}
              animate={{ opacity: 1, filter: "blur(0px)",  scale: 1 }}
              exit={{    opacity: 0, filter: "blur(12px)", scale: 1.04 }}
              transition={{
                opacity: { duration: 0.45, ease: [0.22, 1, 0.36, 1] },
                filter:  { duration: 0.5,  ease: [0.22, 1, 0.36, 1] },
                scale:   { duration: 0.55, ease: [0.22, 1, 0.36, 1] },
              }}
              style={{ position: "absolute", inset: 0, willChange: "filter, opacity, transform", backfaceVisibility: "hidden" }}
            >
              <CoverArt url={currentTrack?.album?.image_url ?? null} alt={currentTrack?.name ?? ""} size={44} />
            </motion.div>
          </AnimatePresence>
          {/* expand affordance on hover (only when a track is loaded) */}
          {currentTrack && (
            <div
              className="queue-btn"
              style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.45)", transition: "opacity 0.12s" }}
            >
              <Maximize2 size={16} strokeWidth={2.4} style={{ color: "#fff" }} />
            </div>
          )}
        </div>
        <div style={{ minWidth: 0 }}>
          <p style={{ margin: 0, fontSize: 13, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: currentTrack ? "var(--color-text-hi)" : "var(--color-text-dim)" }}>
            {currentTrack?.name ?? "Not playing"}
          </p>
          <p style={{ margin: 0, fontSize: 12, fontWeight: 400, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "var(--color-text-dim)" }}>
            {currentTrack?.artists.map((a) => a.name).join(", ") ?? "-"}
          </p>
        </div>
      </div>

      {/* center: controls + progress */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", gap: 7 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <Tooltip label={shuffle ? "Shuffle on" : "Shuffle off"}>
            <IconBtn active={shuffle} onClick={toggleShuffle}>
              <Shuffle size={14} strokeWidth={2.4} />
            </IconBtn>
          </Tooltip>
          <Tooltip label="Previous">
            <IconBtn onClick={handlePrev}>
              <SkipBack size={16} strokeWidth={2} fill="currentColor" />
            </IconBtn>
          </Tooltip>
          <Tooltip label={isPlaying ? "Pause" : "Play"}>
            <PlayPauseButton
              isPlaying={isPlaying}
              onClick={() => {
                if (isPlaying) {
                  setPlaying(false);                  // optimistic, flip icon now
                  pausePlayback().catch(console.error);
                } else if (currentTrack) {
                  setPlaying(true);                   // optimistic
                  const clickedAt = Date.now();
                  const id = currentTrack.id;
                  resumeOrPlay(currentTrack.id, sessionReady ? positionMs : 0)
                    .then(() => {
                      /* watchdog: if no real "playing" event lands within 5s the
                       load silently died (dead session / unavailable track),
                       revert so the UI never shows "playing" with no audio */

                      setTimeout(() => {
                        const s = usePlayerStore.getState();
                        if (s.isPlaying && s.currentId === id && s.lastPlayingAt < clickedAt) {
                          s.setPlaying(false);
                        }
                      }, 5000);
                    })
                    .catch((e) => { console.error(e); setPlaying(false); });
                }
              }}
            />
          </Tooltip>
          <Tooltip label="Next">
            <IconBtn onClick={handleNext}>
              <SkipForward size={16} strokeWidth={2} fill="currentColor" />
            </IconBtn>
          </Tooltip>
          <Tooltip label={repeatLabel}>
            <IconBtn active={repeat !== "none"} onClick={cycleRepeat}>
              <RepeatIcon size={14} strokeWidth={2.4} />
            </IconBtn>
          </Tooltip>
        </div>
        <ProgressBar positionMs={positionMs} durationMs={durationMs} onSeek={doSeek} />
      </div>

      {/* right: volume + queue */}
      <div style={{ width: "28%", display: "flex", alignItems: "center", gap: 8, justifyContent: "flex-end" }}>
        <Tooltip label={muted ? "Unmute" : "Mute"}>
          <IconBtn onClick={handleMuteToggle}>
            <VolumeIcon size={14} strokeWidth={2} fill="currentColor" />
          </IconBtn>
        </Tooltip>
        <Tooltip label={muted ? "Muted" : `Volume ${volume}%`}>
          <input
            className="vol"
            type="range" min={0} max={100}
            value={muted ? 0 : volume}
            onChange={handleVolumeChange}
            onMouseDown={() => { if (muted) { storeSetMuted(false); apiSetMuted(false).catch(() => {}); } }}
            style={{ width: 72, ["--vol" as string]: `${muted ? 0 : volume}%` } as React.CSSProperties}
          />
        </Tooltip>

      {/* TODO reveiew bottom tooltip (ai made, check the paddings) - DONE changed scale value to 1.08 */}
        <Tooltip label={lyricsOpen ? "Hide lyrics" : "Lyrics"}>
          <motion.button
            onClick={() => { if (currentTrack) toggleLyrics(); }}
            disabled={!currentTrack}
            whileHover={currentTrack ? { scale: 1.08 } : {}}
            whileTap={currentTrack ? { scale: 0.92 } : {}}
            transition={{ type: "spring", stiffness: 400, damping: 25 }}
            transformTemplate={zTransform}
            style={{
              ...gpuLayer,
              display:        "flex",
              alignItems:     "center",
              justifyContent: "center",
              width:          28,
              height:         28,
              borderRadius:   5,
              border:         "none",
              background:     lyricsOpen ? "var(--color-accent-dim, rgba(110,231,183,0.12))" : "transparent",
              color:          !currentTrack ? "rgba(242,238,233,0.18)" : lyricsOpen ? "var(--color-accent)" : "var(--color-text)",
              cursor:         currentTrack ? "pointer" : "default",
            }}
          >
            <Captions size={15} strokeWidth={2} />
          </motion.button>
        </Tooltip>
        <Tooltip label={queueOpen ? "Hide queue" : "Show queue"} align="end">
          <motion.button
            onClick={toggleQueue}
            whileHover={{ scale: 1.08 }}
            whileTap={{ scale: 0.92 }}
            transition={{ type: "spring", stiffness: 400, damping: 25 }}
            transformTemplate={zTransform}
            style={{
              ...gpuLayer,
              display:        "flex",
              alignItems:     "center",
              justifyContent: "center",
              width:          28,
              height:         28,
              borderRadius:   5,
              border:         "none",
              background:     queueOpen ? "var(--color-accent-dim, rgba(110,231,183,0.12))" : "transparent",
              color:          queueOpen ? "var(--color-accent)" : "var(--color-text)",
              cursor:         "pointer",
            }}
          >
            <ListMusic size={14} strokeWidth={2} />
          </motion.button>
        </Tooltip>
      </div>
    </div>
  );
}
