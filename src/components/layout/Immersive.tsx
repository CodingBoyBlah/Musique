import { useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import {
  X, Play, Pause, SkipBack, SkipForward, Shuffle, Repeat, Repeat1,
  Captions, ListMusic, Music,
} from "lucide-react";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { usePlayerControls } from "../../hooks/usePlayerControls";
import { useLyrics } from "../../hooks/useLyrics";
import { CoverArt } from "../ui/CoverArt";
import { fmtMs } from "../../utils/fmt";
import { gpuLayer, zTransform } from "../../lib/motion";
import { playTrack } from "../../api/playback";
import {
  ActiveLine,
  buildRows,
  mapWords,
  useActiveRow,
  useLyricClock,
} from "../../lib/lyrics";

// animated gradient bg derived from the cover art like apple music mobile

// {arc browser onboarindg graident inspo too}

function GradientBg({ url }: { url: string | null | undefined }) {
  const reduceMotion = useReducedMotion();
  // isolation:isolate + an opaque base = nothing behind ever bleeds through


// TODO done
  const base: React.CSSProperties = {
    position: "absolute", inset: 0, overflow: "hidden",
    isolation: "isolate", background: "#07070b",
  };
  if (!url) return <div aria-hidden style={base} />;
  /* perf: only TWO blurred layers, one moderate raidius and we animate
   only translate/rotate NOT scale or filter. animating scale on a blurred
   element rerasterizes the (giant) blur every frame, which is exactly what
   made this view crawl {Notes in obsidian} translate/rotate are pure compositor transforms the
   GPU slaps onto the already blurred bitmap for free. no mixBlendMode either
   (kills the compositor fast-paths). still get the slow churn of the song's
   colours, just cheap.
   note: no transform here, framer owns the transform property. scale is passed
   THROUGH framer ( ALWAYS constant, never animated) so the blur rasterizes only once. */

// we could do a third layer with a different blur radius and a different (TODO ??)
  const layer = (opacity: number): React.CSSProperties => ({
    position: "absolute", inset: "-35%",
    backgroundImage: `url(${url})`, backgroundSize: "cover", backgroundPosition: "center",
    filter: "blur(64px) saturate(1.8)", opacity,
    willChange: "transform", backfaceVisibility: "hidden",
  });
  return (
    <div aria-hidden style={base}>
      <motion.div
        initial={false}
        animate={reduceMotion ? { scale: 1.35 } : { scale: 1.35, x: [0, 70, -50, 0], y: [0, -54, 44, 0], rotate: [0, 6, -5, 0] }}
        transition={{ duration: 30, repeat: Infinity, ease: "easeInOut" }}
        style={layer(0.95)}
      />
      <motion.div
        initial={false}
        animate={reduceMotion ? { scale: 1.6 } : { scale: 1.6, x: [0, -60, 50, 0], y: [0, 48, -38, 0], rotate: [0, -7, 5, 0] }}
        transition={{ duration: 44, repeat: Infinity, ease: "easeInOut" }}
        style={layer(0.5)}
      />
      {/* darkening scrim so white text/controls stay legible over any artwork */}
      <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(4,4,8,0.55) 0%, rgba(4,4,8,0.32) 40%, rgba(4,4,8,0.62) 100%)" }} />
    </div>
  );
}

// synced lyrics same wordby word engine as the side LyricsPanel too


/* IMMERSIVE - background graident like arcs onboarding, panels: Album art, play, pause, and on the side EITHER
lyrics or queue, like cider TODO DONE*/
function ImmersiveLyrics() {
  const track        = usePlayerStore((s) => s.currentTrack);
  const setPosition  = usePlayerStore((s) => s.setPosition);
  const reduceMotion = useReducedMotion();
  const { data, isLoading } = useLyrics(track);
  const { seek } = usePlayerControls();

  const rows = useMemo(() => buildRows(data), [data]);
  const rowStarts = useMemo(() => rows.map((r) => r.startMs), [rows]);
  const synced = !!data?.lines.length;

  const { getClock, resync } = useLyricClock();
  const active = useActiveRow(rowStarts, getClock, synced);

  const scrollRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLDivElement | null)[]>([]);
  useEffect(() => {
    if (active < 0) return;
    const el = rowRefs.current[active];
    const cont = scrollRef.current;
    if (!el || !cont) return;
    cont.scrollTo({ top: el.offsetTop - cont.clientHeight * 0.42 + el.clientHeight / 2, behavior: reduceMotion ? "auto" : "smooth" });
  }, [active, reduceMotion]);

  function seekTo(i: number) {
    if (!synced) return;
    const ms = rowStarts[i];
    setPosition(ms);
    resync(ms);
    seek(ms);
  }

  if (isLoading) return <Centered>Finding lyrics…</Centered>;
  if (data?.instrumental) return <Centered>Instrumental - no lyrics.</Centered>;
  if (rows.length === 0) return <Centered>No lyrics found for this track.</Centered>;

  return (
    <div ref={scrollRef} data-selectable style={lyricsScroll}>
      {rows.map((row, ri) => {
        const isActive = synced && ri === active;
        const isPast = synced && ri < active;
        const multi = row.voices.length > 1;
        return (
          <div
            key={ri}
            ref={(el) => { rowRefs.current[ri] = el; }}
            onClick={() => seekTo(ri)}
            style={{
              cursor: synced ? "pointer" : "default", padding: "7px 4px", borderRadius: 10,
              transition: "opacity 0.3s ease, transform 0.4s cubic-bezier(0.23,1,0.32,1)",
              transformOrigin: "left center",
              transform: isActive ? "scale(1.03)" : "scale(1)",
              opacity: !synced ? 0.9 : isActive ? 1 : isPast ? 0.32 : 0.5,
              display: "flex", flexDirection: "column", gap: multi ? 4 : 0,
            }}
          >
            {row.voices.map((voice, vi) => {
              const size = vi === 0 ? 30 : 23;
              const weight = vi === 0 ? 800 : 700;
              const indent = vi === 0 ? 0 : 18;
              return (
                <div
                  key={vi}
                  style={{
                    marginLeft: indent,
                    borderLeft: vi === 0 ? "none" : "2px solid rgba(255,255,255,0.18)",
                    paddingLeft: vi === 0 ? 0 : 10,
                  }}
                >
                  {isActive && voice.words.length ? (
                    // word-by-word brightness sweep (real musixmatch/netease timings)
                    <ActiveLine words={mapWords(voice)} getClock={getClock} size={size} weight={weight} halo={0.18} />
                  ) : (
                    <p
                      style={{
                        margin: 0, fontSize: size, fontWeight: weight, lineHeight: 1.3, letterSpacing: "-0.01em",
                        color: isActive ? "#fff" : "rgba(255,255,255,0.82)",
                        textShadow: isActive ? "0 0 24px rgba(255,255,255,0.18)" : "none",
                      }}
                    >
                      {voice.text || "♪"}
                    </p>
                  )}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

const lyricsScroll: React.CSSProperties = {
  height: "100%", overflowY: "auto", padding: "8vh 8px 30vh",
  WebkitMaskImage: "linear-gradient(to bottom, transparent 0, #000 8%, #000 86%, transparent 100%)",
  maskImage: "linear-gradient(to bottom, transparent 0, #000 8%, #000 86%, transparent 100%)",
};

function Centered({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ height: "100%", display: "flex", alignItems: "center", justifyContent: "center", textAlign: "center", color: "rgba(255,255,255,0.6)", fontSize: 15, padding: 24 }}>
      {children}
    </div>
  );
}

// immersive queue

function ImmersiveQueue() {
  const queue = useQueueStore((s) => s.queue);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);

  if (queue.length === 0) return <Centered>Nothing queued.</Centered>;

  function jump(i: number) {
    /*drop everything before the picked track, then play it
     ( 827cd1 used to splice in place, this is simpler) */
    const picked = queue[i];
    useQueueStore.setState({ queue: queue.slice(i + 1) });
    setCurrentTrack(picked);
    playTrack(picked.id).catch(() => {});
  }

  return (
    <div data-selectable style={{ height: "100%", overflowY: "auto", padding: "12px 4px 30vh" }}>
      <p style={{ margin: "0 0 12px", fontSize: 12, fontWeight: 700, letterSpacing: "0.06em", textTransform: "uppercase", color: "rgba(255,255,255,0.5)" }}>Up next</p>
      {queue.map((t, i) => (
        <button
          key={`${t.id}-${i}`}
          onClick={() => jump(i)}
          style={{
            display: "flex", alignItems: "center", gap: 12, width: "100%", padding: "8px 10px",
            border: "none", background: "transparent", borderRadius: 10, cursor: "pointer", textAlign: "left",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.08)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
        >
          {t.album?.image_url
            ? <img src={t.album.image_url} alt="" style={{ width: 44, height: 44, borderRadius: 6, objectFit: "cover", flexShrink: 0 }} />
            : <div style={{ width: 44, height: 44, borderRadius: 6, background: "rgba(255,255,255,0.08)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}><Music size={18} style={{ color: "rgba(255,255,255,0.4)" }} /></div>}
          <div style={{ minWidth: 0, flex: 1 }}>
            <p style={{ margin: 0, fontSize: 14, fontWeight: 600, color: "#fff", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{t.name}</p>
            <p style={{ margin: 0, fontSize: 12.5, color: "rgba(255,255,255,0.55)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{t.artists.map((a) => a.name).join(", ")}</p>
          </div>
        </button>
      ))}
    </div>
  );
}

// lean progress bar

function Progress() {
  const positionMs = usePlayerStore((s) => s.positionMs);
  const durationMs = usePlayerStore((s) => s.durationMs);
  const { seek } = usePlayerControls();
  const barRef = useRef<HTMLDivElement>(null);
  const [drag, setDrag] = useState<number | null>(null);

  const frac = drag != null ? drag : durationMs > 0 ? Math.min(positionMs / durationMs, 1) : 0;

  function fracFrom(clientX: number) {
    const el = barRef.current; if (!el) return 0;
    const r = el.getBoundingClientRect();
    return Math.max(0, Math.min(1, (clientX - r.left) / r.width));
  }

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 10, width: "100%" }}>
      <span style={tmark}>{fmtMs(drag != null ? Math.floor(drag * durationMs) : positionMs)}</span>
      <div
        ref={barRef}
        onPointerDown={(e) => { if (!durationMs) return; e.currentTarget.setPointerCapture(e.pointerId); setDrag(fracFrom(e.clientX)); }}
        onPointerMove={(e) => { if (drag != null) setDrag(fracFrom(e.clientX)); }}
        onPointerUp={(e) => { if (drag != null) { seek(Math.floor(drag * durationMs)); setDrag(null); } try { e.currentTarget.releasePointerCapture(e.pointerId); } catch { /* noop */ } }}
        style={{ flex: 1, height: 16, display: "flex", alignItems: "center", cursor: durationMs ? "pointer" : "default", touchAction: "none" }}
      >
        <div style={{ position: "relative", width: "100%", height: 5, borderRadius: 99 }}>
          <div style={{ position: "absolute", inset: 0, background: "rgba(255,255,255,0.18)", borderRadius: 99 }} />
          <div style={{ position: "absolute", inset: "0 auto 0 0", width: `${frac * 100}%`, background: "#fff", borderRadius: 99 }} />
        </div>
      </div>
      <span style={tmark}>{durationMs > 0 ? fmtMs(durationMs) : "0:00"}</span>
    </div>
  );
}

const tmark: React.CSSProperties = { fontSize: 12, color: "rgba(255,255,255,0.6)", flexShrink: 0, fontVariantNumeric: "tabular-nums" };

// round control button

function Ctl({ children, onClick, active, big, title }: { children: React.ReactNode; onClick: () => void; active?: boolean; big?: boolean; title?: string }) {
  return (
    <motion.button
      onClick={onClick}
      title={title}
      whileHover={{ scale: 1.1 }}
      whileTap={{ scale: 0.9 }}
      transition={{ type: "spring", stiffness: 400, damping: 22 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0,
        width: big ? 64 : 44, height: big ? 64 : 44, borderRadius: "50%", border: "none",
        background: big ? "#fff" : "transparent",
        color: big ? "#0a0a0f" : active ? "#fff" : "rgba(255,255,255,0.75)",
        cursor: "pointer",
      }}
    >
      {children}
    </motion.button>
  );
}

// immersive view

export function Immersive() {
  const open        = usePlayerStore((s) => s.immersiveOpen);
  const setOpen     = usePlayerStore((s) => s.setImmersiveOpen);
  const panel       = usePlayerStore((s) => s.immersivePanel);
  const setPanel    = usePlayerStore((s) => s.setImmersivePanel);
  const track       = usePlayerStore((s) => s.currentTrack);
  const isPlaying   = usePlayerStore((s) => s.isPlaying);
  const { togglePlay, next, prev } = usePlayerControls();
  const { shuffle, repeat, toggleShuffle, cycleRepeat } = useQueueStore();

  // esc closes
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, setOpen]);

  const RepeatIcon = repeat === "one" ? Repeat1 : Repeat;

  return (
    <AnimatePresence>
      {open && track && (
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 24 }}
          transition={{ duration: 0.3, ease: [0.23, 1, 0.32, 1] }}
          style={{ position: "fixed", inset: 0, zIndex: 900, overflow: "hidden", color: "#fff", background: "#07070b" }}
        >
          <GradientBg url={track.album?.image_url} />

          {/* window drag strip immersive covers the whole window (titlebar
              included) so without this you couldnt drag the window here. BUT this sits OVER
               the empty top padding and  close button (higher z) stays
              clickable. macOS native traffic lights render ABOVe the webview */}
          <div
            data-tauri-drag-region
            style={{ position: "absolute", top: 0, left: 0, right: 0, height: 40, zIndex: 4 }}
          />

          {/* close */}
          <button
            onClick={() => setOpen(false)}
            title="Close (Esc)"
            style={{
              position: "absolute", top: 18, right: 18, zIndex: 5,
              width: 40, height: 40, borderRadius: "50%", border: "none", cursor: "pointer",
              background: "rgba(255,255,255,0.1)", color: "#fff",
              display: "flex", alignItems: "center", justifyContent: "center",
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.2)"; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.1)"; }}
          >
            <X size={18} strokeWidth={2.4} />
          </button>

          {/* two-column content */}
          <div
            style={{
              position: "relative", zIndex: 2, height: "100%",
              display: "flex", gap: "clamp(20px, 4vw, 64px)",
              padding: "clamp(40px, 6vh, 80px) clamp(24px, 5vw, 80px)",
              alignItems: "center", flexWrap: "wrap",
            }}
          >
            {/* left: cover + meta + controls */}
            <div style={{ flex: "1 1 420px", minWidth: 0, maxWidth: 620, display: "flex", flexDirection: "column", gap: "clamp(18px, 3vh, 32px)", margin: "0 auto" }}>
              <motion.div
                layout
                style={{ width: "min(46vh, 100%)", aspectRatio: "1 / 1", alignSelf: "center", borderRadius: 16, overflow: "hidden", boxShadow: "0 30px 80px rgba(0,0,0,0.55)" }}
              >
                <CoverArt url={track.album?.image_url} alt={track.name} size={520} style={{ width: "100%", height: "100%" }} />
              </motion.div>

              <div style={{ minWidth: 0 }}>
                <h1 style={{ margin: 0, fontSize: "clamp(24px, 3.4vw, 40px)", fontWeight: 800, letterSpacing: "-0.02em", lineHeight: 1.1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{track.name}</h1>
                <p style={{ margin: "6px 0 0", fontSize: "clamp(14px, 1.6vw, 18px)", color: "rgba(255,255,255,0.7)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {track.artists.map((a) => a.name).join(", ")}
                </p>
              </div>

              <Progress />

              <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "clamp(12px, 2vw, 26px)" }}>
                <Ctl active={shuffle} onClick={toggleShuffle} title="Shuffle"><Shuffle size={20} strokeWidth={2.4} /></Ctl>
                <Ctl onClick={prev} title="Previous"><SkipBack size={24} strokeWidth={2} fill="currentColor" /></Ctl>
                <Ctl big onClick={togglePlay} title={isPlaying ? "Pause" : "Play"}>
                  {isPlaying ? <Pause size={30} fill="currentColor" strokeWidth={0} /> : <Play size={30} fill="currentColor" strokeWidth={0} style={{ marginLeft: 3 }} />}
                </Ctl>
                <Ctl onClick={next} title="Next"><SkipForward size={24} strokeWidth={2} fill="currentColor" /></Ctl>
                <Ctl active={repeat !== "none"} onClick={cycleRepeat} title="Repeat"><RepeatIcon size={20} strokeWidth={2.4} /></Ctl>
              </div>
            </div>

            {/* right: lyrics / queue switcher */}
            <div style={{ flex: "1 1 380px", minWidth: 0, height: "100%", maxHeight: "82vh", display: "flex", flexDirection: "column", minHeight: 0 }}>
              <div style={{ display: "flex", gap: 4, padding: 4, borderRadius: 99, background: "rgba(255,255,255,0.08)", alignSelf: "flex-start", marginBottom: 8 }}>
                <PanelTab active={panel === "lyrics"} onClick={() => setPanel("lyrics")} icon={<Captions size={15} />} label="Lyrics" />
                <PanelTab active={panel === "queue"} onClick={() => setPanel("queue")} icon={<ListMusic size={15} />} label="Queue" />
              </div>
              <div style={{ flex: 1, minHeight: 0 }}>
                <AnimatePresence mode="wait">
                  <motion.div
                    key={panel}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -8 }}
                    transition={{ duration: 0.2, ease: [0.23, 1, 0.32, 1] }}
                    style={{ height: "100%" }}
                  >
                    {panel === "lyrics" ? <ImmersiveLyrics /> : <ImmersiveQueue />}
                  </motion.div>
                </AnimatePresence>
              </div>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function PanelTab({ active, onClick, icon, label }: { active: boolean; onClick: () => void; icon: React.ReactNode; label: string }) {
  return (
    <button
      onClick={onClick}
      style={{
        display: "flex", alignItems: "center", gap: 7, height: 32, padding: "0 16px", borderRadius: 99,
        border: "none", cursor: "pointer", fontSize: 13, fontWeight: 600, fontFamily: "inherit",
        background: active ? "#fff" : "transparent",
        color: active ? "#0a0a0f" : "rgba(255,255,255,0.7)",
        transition: "background 0.15s, color 0.15s",
      }}
    >
      {icon} {label}
    </button>
  );
}
