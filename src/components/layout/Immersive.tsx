import { useEffect, useMemo, useRef } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { Minimize2, Captions, Queue, Music } from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { usePlayerControls } from "../../hooks/usePlayerControls";
import { useLyrics } from "../../hooks/useLyrics";
import { useAmbient } from "../../hooks/useAmbient";
import { useWindowActive } from "../../hooks/useWindowActive";
import { PlayerBar } from "./PlayerBar";
import { playTrack } from "../../api/playback";
import {
  LyricRowText,
  buildRows,
  lyricTone,
  lyricWords,
  useActiveRow,
  useLyricClock,
} from "../../lib/lyrics";

/* The room the record is playing in.
 *
 * One copy of the cover, blurred once into a bitmap by lib/ambient, drifting on
 * transform alone. The sharp sleeve is drawn over it by Sleeve below, feathered
 * at its right edge and corners, so what shows through around the artwork - the
 * corners, and the whole right side the lyrics sit on - is this layer moving.
 *
 * Everything expensive that used to live here is gone, and none of it was ever
 * needed for the look. The first version blurred two full-window copies of the
 * cover live, on every frame, under a `backdrop-filter` panel that re-blurred
 * half the window again - a backdrop-filter's input is whatever is behind it,
 * and that was moving, so it could never be cached. A later pass added a
 * full-window `mix-blend-mode: overlay` grain tile, which forces the whole
 * subtree onto a render surface that has to be re-blended on every frame the
 * ambient moves. Blurring once into a bitmap and then only transforming it
 * costs nothing per frame, and the artwork now supplies its own texture. */
function AmbientBg({ url }: { url: string | null | undefined }) {
  const { art, blobs, base, glow } = useAmbient(url);
  const awake = useWindowActive();

  return (
    <div
      aria-hidden
      style={{ position: "absolute", inset: 0, overflow: "hidden", background: base, pointerEvents: "none", contain: "strict" }}
    >
      <div
        className={awake ? "amb-layer amb-1" : "amb-layer amb-1 amb-parked"}
        style={
          art
            ? { backgroundImage: `url("${art}")` }
            : // unreadable artwork: light the room off the palette instead
              { background: `radial-gradient(circle at 34% 40%, rgba(${blobs[0]}, 0.75) 0%, rgba(${blobs[1]}, 0.35) 52%, rgba(${blobs[2]}, 0) 78%)` }
        }
      />

{/* One element, two stacked gradients: the light the sleeve pools into
          the room, and the scrim over the strips the window chrome and the dock
          actually sit on. Kept on a single div because every full-window layer
          here is another full-window blend on every frame the backdrop moves.
          The scrim stays off the middle of the frame - an earlier pass laid a
          vignette and a full-height wash over this and turned a hot pink sleeve
          into dead maroon. */}
      <div
        style={{
          position: "absolute", inset: 0,
          background:
            `radial-gradient(ellipse 52% 60% at 26% 50%, rgba(${glow}, 0.18) 0%, rgba(${glow}, 0) 74%), ` +
            "linear-gradient(180deg, rgba(4,4,8,0.42) 0%, rgba(4,4,8,0.04) 16%, rgba(4,4,8,0) 52%, rgba(4,4,8,0.14) 76%, rgba(4,4,8,0.50) 100%)",
        }}
      />
    </div>
  );
}

/* The sleeve, full bleed down the left.
 *
 * A single radial mask does both jobs the reference does: it holds the artwork
 * solid through the left edge and the middle, then feathers it out across the
 * right and softens all four corners, so the drifting blur underneath comes
 * through exactly where the lyrics are. One mask on a static image rasterises
 * once - it is not a filter and it costs nothing per frame. */
function Sleeve({ url, alt }: { url: string | null | undefined; alt: string }) {
  const src = coverUrl(url, 900) ?? url;
  if (!src) return null;

/* Three masks, multiplied together.
 *
 * The first is the falloff across the right of the sleeve, where the lyrics
 * sit. The other two open up the window's top-left and bottom-left corners, so
 * the drift shows through there as well - one ellipse cannot do both jobs,
 * because widening it until the left corners dissolve also thins the artwork
 * along the whole top and bottom edge. Squeezing the first mask's vertical
 * radius was that mistake: it took the corners with it but hollowed out the
 * middle of the sleeve on the way.
 *
 * The intersect composite is what makes them multiply. The default is add,
 * which unions instead - each corner gradient is opaque everywhere the other
 * one is cut out, so a union would simply cancel both holes. */
const SLEEVE_MASK = [
  /* Falloff across the sleeve. These percentages are relative to the 62%-wide
     container, which makes them easy to misread - in window terms the artwork
     stays solid to about 31% and has dissolved completely by 57%. */
  "radial-gradient(ellipse 90% 150% at 11.5% 50%, #000 0%, #000 42%, transparent 89%)",
  // the window's top-left corner
  "radial-gradient(ellipse 44% 36% at 0% 0%, transparent 6%, #000 94%)",
  // and its bottom-left
  "radial-gradient(ellipse 44% 36% at 0% 100%, transparent 6%, #000 94%)",
].join(", ");

  return (
    <div aria-hidden style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: "62%", overflow: "hidden", pointerEvents: "none" }}>
      <AnimatePresence initial={false}>
        <motion.img
          key={src}
          src={src}
          alt={alt}
          referrerPolicy="no-referrer"
          decoding="async"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
          style={{
            position: "absolute", inset: 0,
            width: "100%", height: "100%", objectFit: "cover", objectPosition: "center",
            WebkitMaskImage: SLEEVE_MASK,
            maskImage: SLEEVE_MASK,
            maskComposite: "intersect",
          }}
        />
      </AnimatePresence>
    </div>
  );
}

// synced lyrics same wordby word engine as the side LyricsPanel too


/* IMMERSIVE - the sleeve fills the left, the lyrics sit on the drifting blur to
the right of it, and the app's own dock floats over the bottom */
function ImmersiveLyrics({ glow, ink }: { glow: string; ink: string }) {
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

    /* The active line rests with its middle at 42% of the column, not 34% -
       it was sitting close enough to the tabs to feel pinned under them.
       Centring rather than top-aligning is what makes it size-aware: a line
       that wraps to three rows grows in both directions instead of pushing its
       tail off the bottom. The clamp is for the extreme case, where a line long
       enough to fill half the column would still ride up under the header. */
    const H = cont.clientHeight;
    const target = Math.min(
      el.offsetTop - H * 0.42 + el.clientHeight / 2,
      el.offsetTop - H * 0.16,
    );
    if (reduceMotion) {
      cont.scrollTop = target;
      return;
    }

    const from = cont.scrollTop;
    const delta = target - from;
    if (Math.abs(delta) < 1) return;

    // ease-out-quart, the curve the row's scale is on
    const DURATION = 560;
    const start = performance.now();
    let raf = 0;
    const step = (now: number) => {
      const t = Math.min(1, (now - start) / DURATION);
      cont.scrollTop = from + delta * (1 - Math.pow(1 - t, 4));
      if (t < 1) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
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
    <div ref={scrollRef} data-selectable className="lyr-scroll" style={lyricsScroll}>
      {rows.map((row, ri) => {
        const isActive = synced && ri === active;
        const tone = synced ? lyricTone(Math.abs(ri - active), ri < active) : { blur: 0, alpha: 0.9 };
        const multi = row.voices.length > 1;
        return (
          <div
            key={ri}
            ref={(el) => { rowRefs.current[ri] = el; }}
            onClick={() => seekTo(ri)}
            style={{
              cursor: synced ? "pointer" : "default",
              /* Constant box: the padding never changes. Growing the active
                 row reflowed the whole list on every line and moved the scroll
                 target out from under the tween chasing it, so size is carried
                 by transform, which does not relayout.

                 The list is laid out at the ACTIVE size and everything else is
                 scaled DOWN to 0.79 - 1/1.27, the ratio measured off the
                 reference. Scaling the active line up instead is the obvious
                 way round and the wrong one: a scale about `left center` grows
                 the box rightward without the layout knowing, so the effect has
                 to be paid for in reserved width that every other line wraps
                 early to fund. This way the widest thing on screen is always
                 the thing the layout measured, and nothing can clip however
                 large the ratio gets. */
              padding: "5px 0",
              opacity: tone.alpha,
              transform: isActive ? "scale(1)" : "scale(0.79)",
              transformOrigin:
                row.voices.length === 1 && row.voices[0].role === "duet"
                  ? "right center"
                  : "left center",
              transition: reduceMotion
                ? "none"
                : "opacity 0.44s cubic-bezier(0.22, 1, 0.36, 1), transform 0.56s cubic-bezier(0.22, 1, 0.36, 1)",
              display: "flex", flexDirection: "column", gap: multi ? 6 : 0,
            }}
          >
            {row.voices.map((voice, vi) => {
              const isDuet = voice.role === "duet";
              const isSecondary = vi > 0 && !isDuet;
              // Apple Music: lead voice stays left, duet voice gets opposite horizontal alignment (right).
              // If duet is solo in row (vi === 0 && isDuet), it also aligns right.
              // If duet is concurrent (vi > 0 && isDuet), it aligns right opposite lead.
              const align: "left" | "right" = isDuet ? "right" : "left";

              const size = isSecondary
                ? "clamp(21px, 2.15vw, 34px)"
                : isDuet && vi > 0
                ? "clamp(24px, 2.5vw, 40px)"
                : "clamp(28px, 2.9vw, 46px)";
              const weight = isSecondary ? 700 : 800;
              // built for every row, not just the lit one: both states render
              // the same spans so the line can never re-wrap when it lights up
              const words = lyricWords(voice, row.startMs, row.endMs);
              return (
                <div
                  key={vi}
                  style={{
                    marginLeft: isSecondary ? 18 : 0,
                    borderLeft: isSecondary ? "2px solid rgba(255,255,255,0.18)" : "none",
                    paddingLeft: isSecondary ? 12 : 0,
                    opacity: isSecondary ? 0.78 : 1,
                    textAlign: align,
                  }}
                >
                  <LyricRowText
                    words={words}
                    active={isActive}
                    getClock={getClock}
                    tone={tone}
                    size={size}
                    weight={weight}
                    glowRgb={glow}
                    inkRgb={ink}
                    align={align}
                  />
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
  height: "100%", overflowY: "auto", overflowX: "hidden",
  /* Real gutters. The text used to sit 8px off the panel edge and then get
     scaled 12% wider from that edge, so long lines ran off the right of the
     window entirely. */
  padding: "10vh clamp(16px, 1.8vw, 30px) 32vh clamp(14px, 1.6vw, 26px)",
  WebkitMaskImage: "linear-gradient(to bottom, transparent 0, #000 9%, #000 84%, transparent 100%)",
  maskImage: "linear-gradient(to bottom, transparent 0, #000 9%, #000 84%, transparent 100%)",
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
    usePlayerStore.getState().setPlaying(true);
    usePlayerStore.getState().setTargetState("playing");
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
            ? <img src={coverUrl(t.album.image_url, 44) ?? t.album.image_url} alt="" style={{ width: 44, height: 44, borderRadius: 6, objectFit: "cover", flexShrink: 0 }} />
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


// immersive view

export function Immersive() {
  const open     = usePlayerStore((s) => s.immersiveOpen);
  const setOpen  = usePlayerStore((s) => s.setImmersiveOpen);
  const panel    = usePlayerStore((s) => s.immersivePanel);
  const setPanel = usePlayerStore((s) => s.setImmersivePanel);
  const track    = usePlayerStore((s) => s.currentTrack);
  // the same cached read AmbientBg does: the lyric ink and glow come off the
  // cover, so the type is lit by the record it belongs to
  const { glow, ink } = useAmbient(track?.album?.image_url);

  // esc closes
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, setOpen]);

  return (
    <AnimatePresence>
      {open && track && (
        <motion.div
          /* Opacity only.
           *
           * This first animated `filter: blur(16px)` to zero, which re-rasterises
           * every pixel of the window on every frame of the transition. Swapping
           * that for a scale was better but still wrong: scaling a full-screen
           * element re-rasters its whole subtree at each new scale, which is the
           * spike you see on open and close. Opacity is the one property the
           * compositor can animate without touching raster at all. */
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.26, ease: [0.22, 1, 0.36, 1] }}
          style={{ position: "fixed", inset: 0, zIndex: 900, overflow: "hidden", color: "#fff", background: "#07070b" }}
        >
          <AmbientBg url={track.album?.image_url} />
          <Sleeve url={track.album?.image_url} alt={track.name} />

          {/* window drag strip immersive covers the whole window (titlebar
              included) so without this you couldnt drag the window here. BUT this sits OVER
               the empty top padding and  close button (higher z) stays
              clickable. macOS native traffic lights render ABOVe the webview */}
          <div
            data-tauri-drag-region
            style={{ position: "absolute", top: 0, left: 0, right: 0, height: 40, zIndex: 4 }}
          />

          {/* The sleeve owns the left of the window; the lyrics and the dock
              share a single column down the right.

              They are stacked in one flex column rather than positioned
              separately so their width comes from the same parent - there is no
              pair of numbers to keep in sync, and they cannot fall out of
              alignment at any window size. It also puts the dock over the part
              of the frame the drift already softens, which is why it can stay
              transparent without a backdrop-filter of its own. */}
          <div
            style={{
              position: "absolute", inset: 0, zIndex: 5,
              display: "flex", justifyContent: "flex-end",
              padding: "clamp(24px, 4vh, 52px) clamp(20px, 3vw, 50px) clamp(16px, 2.6vh, 28px)",
              pointerEvents: "none",
            }}
          >
            <div
              style={{
                width: "min(50%, 760px)",
                minWidth: 360,
                display: "flex",
                flexDirection: "column",
                gap: "clamp(12px, 1.6vh, 20px)",
                minHeight: 0,
              }}
            >
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.28, delay: 0.04, ease: [0.22, 1, 0.36, 1] }}
                style={{
                  pointerEvents: "auto",
                  flex: 1,
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                  borderRadius: 26,
                  /* Dark, not pale. The reference card is a shade dropped over the
                     ambient, which is what keeps white type readable while the
                     field behind it keeps moving and changing colour. */
                  background: "linear-gradient(180deg, rgba(10, 8, 14, 0.30) 0%, rgba(10, 8, 14, 0.42) 100%)",
                  border: "1px solid rgba(255, 255, 255, 0.11)",
                  boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 30px 80px rgba(0, 0, 0, 0.30)",
                  padding: "20px clamp(20px, 2vw, 36px)",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12, marginBottom: 10 }}>
                  <div className="lyr-tabs">
                    <PanelTab active={panel === "lyrics"} onClick={() => setPanel("lyrics")} icon={<Captions size={14} active={panel === "lyrics"} />} label="Lyrics" />
                    <PanelTab active={panel === "queue"} onClick={() => setPanel("queue")} icon={<Queue size={14} active={panel === "queue"} />} label="Queue" />
                  </div>
                  <button
                    className="imm-close"
                    onClick={() => setOpen(false)}
                    title="Exit immersive view (Esc)"
                    aria-label="Exit immersive view"
                  >
                    <Minimize2 size={16} strokeWidth={2.2} />
                  </button>
                </div>
                <div style={{ flex: 1, minHeight: 0 }}>
                  <AnimatePresence mode="wait">
                    <motion.div
                      key={panel}
                      initial={{ opacity: 0, y: 8 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: -6 }}
                      transition={{ duration: 0.16, ease: [0.22, 1, 0.36, 1] }}
                      style={{ height: "100%" }}
                    >
                      {panel === "lyrics" ? <ImmersiveLyrics glow={glow} ink={ink} /> : <ImmersiveQueue />}
                    </motion.div>
                  </AnimatePresence>
                </div>
              </motion.div>

              {/* The app's own dock, minus the 44px sleeve - it is already
                  filling the screen behind this. Same component, so transport,
                  scrubber, volume, devices and queue all behave identically. */}
              <motion.div
                initial={{ opacity: 0, y: 14 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3, delay: 0.06, ease: [0.22, 1, 0.36, 1] }}
                style={{ flexShrink: 0 }}
              >
                <PlayerBar immersive />
              </motion.div>
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
      className={active ? "lyr-tab lyr-tab-on" : "lyr-tab"}
      onClick={onClick}
      aria-pressed={active}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

