import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useCanvas } from "../../hooks/useCanvas";
import { useVideoAmbient } from "../../hooks/useVideoAmbient";
import { PodcastStage } from "../podcast/PodcastStage";
import { TranscriptView } from "../podcast/TranscriptView";
import { isEpisodeId } from "../../utils/episode";
import type { Canvas } from "../../api/internal";
import { coverUrl } from "../../lib/coverUrl";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { Minimize2 } from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { usePlayerControls } from "../../hooks/usePlayerControls";
import { useLyrics } from "../../hooks/useLyrics";
import { useAmbient } from "../../hooks/useAmbient";
import { useWindowActive } from "../../hooks/useWindowActive";
import { useIdleControls } from "../../hooks/useIdleControls";
import { useLyricFollow } from "../../hooks/useLyricFollow";
import { ReturnPill } from "./LyricReturnPill";
import { usePrefsStore } from "../../store/prefs.store";
import { EASE_OUT } from "../../lib/motion";
import { type Ambient } from "../../lib/ambient";
import "../../styles/lyrics.css";
import { PlayerBar } from "./PlayerBar";
import { MeshRoom } from "./MeshRoom";
import {
  LyricInterlude,
  LyricRowText,
  buildRows,
  withInterludes,
  rowDistances,
  voiceLayout,
  lyricTone,
  lyricWords,
  useActiveLyrics,
  useLyricClock,
  useMoreContrast,
} from "../../lib/lyrics";

/* The room the record is playing in: a moving mesh of the cover's colours
 * (MeshRoom), with the sharp sleeve drawn over it by Sleeve below, feathered at
 * its right edge and corners, so what shows through around the artwork - the
 * corners, and the whole right side the lyrics sit on - is the mesh moving.
 *
 * A playing canvas video lends its palette too (useVideoAmbient, re-read every
 * couple of seconds) and the mesh glides to it. It used to be the video itself,
 * blurred live, which brought back exactly what the mesh is for getting rid
 * of: shapes you could make out behind the lyrics.
 *
 * The first version of this room blurred two full-window copies of the cover
 * live, on every frame, under a `backdrop-filter` panel; a later one laid a
 * `mix-blend-mode: overlay` grain tile over the whole window. Both made the
 * compositor re-blend the full window every frame. The mesh is one small
 * canvas, stretched. */
function AmbientBg({ url, override }: { url: string | null | undefined; override?: Ambient | null }) {
  const cover = useAmbient(url);
  const reduceMotion = useReducedMotion();
  // a playing canvas lights the room from its own frames
  const ambient = override ? { ...override, ready: true } : cover;
  const { base, glow, ready } = ambient;
  const awake = useWindowActive();
  // Settings > Animated background. still, not gone: the room keeps its colour
  const drift = usePrefsStore((s) => s.ambientMotion);
  // the room moves with the music: paused, it holds still where it is
  const playing = usePlayerStore((s) => s.isPlaying);

  /* Nothing is drawn until the first cover has actually been read - lighting
     the room off the fallback palette for a frame read as a purple flash on
     every open. */
  return (
    <div
      aria-hidden
      style={{ position: "absolute", inset: 0, overflow: "hidden", background: base, pointerEvents: "none", contain: "strict" }}
    >
      <MeshRoom mesh={ambient.mesh} ready={ready} moving={awake && drift && playing && !reduceMotion} />

      {/* One element, two stacked gradients: the light the sleeve pools into
          the room, and the scrim over the strips the window chrome and the dock
          actually sit on. The scrim stays off the middle of the frame - an
          earlier pass laid a vignette and a full-height wash over this and
          turned a hot pink sleeve into dead maroon. */}
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

const ROOM_FADE_S = 0.6;

// the dock leaving, and coming back: out unhurried, back at once
const HIDE_S = 0.4;
const HIDE_EASE = "cubic-bezier(0.4, 0, 0.2, 1)";
const SHOW_S = 0.12;

/* How much shade the lyric side needs, from how bright the room is there.
 *
 * Aims the backdrop at a relative luminance of 0.15, about where white type
 * clears 5:1 and the dimmed lines either side of it still read. Shading by `a`
 * scales each sRGB channel by (1 - a), which is roughly (1 - a)^2.2 in linear
 * light, hence the inverse power. Capped so a white sleeve deepens rather than
 * turning into a black wall. */
const SHADE_TARGET = 0.15;
function lyricShade(light: number): number {
  if (light <= SHADE_TARGET) return 0;
  return Math.min(0.62, 1 - (SHADE_TARGET / light) ** (1 / 2.2));
}

/* Smoothstep from 28% to 66% of the window, written out as stops - a plain two
   stop gradient is linear, and its start and end read as edges. */
const SHADE_RAMP = Array.from({ length: 9 }, (_, i) => {
  const t = i / 8;
  return { at: 28 + 38 * t, a: t * t * (3 - 2 * t) };
});

/* Shade under the lyrics, only as deep as this record needs.
 *
 * A dark sleeve gets none and looks exactly as it did; a pale one is brought
 * down to where white type reads, and only on the lyric side, so the artwork
 * keeps its full brightness. Tinted with the room's own floor colour rather
 * than black, so a yellow room deepens to amber instead of going grey.
 *
 * It sits OVER the sleeve, not in the room under it. Under it, the sleeve's
 * feathered edge stayed bright while the room behind went dark, and the point
 * where the feather ran out showed as a hard step. Over it, sleeve and room are
 * darkened by the same eased ramp, so there is one transition and no seam. One
 * static layer, faded on opacity alone. */
function LyricShade({ base, light, ready }: { base: string; light: number; ready: boolean }) {
  const n = parseInt(base.slice(1), 16);
  const rgb = Number.isNaN(n) ? "0, 0, 0" : `${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}`;
  const stops = SHADE_RAMP.map((s) => `rgba(${rgb}, ${s.a.toFixed(3)}) ${s.at.toFixed(1)}%`).join(", ");
  return (
    <div
      aria-hidden
      style={{
        position: "absolute", inset: 0, pointerEvents: "none",
        background: `linear-gradient(90deg, ${stops})`,
        opacity: ready ? lyricShade(light) : 0,
        transition: `opacity ${ROOM_FADE_S}s cubic-bezier(0.4, 0, 0.2, 1)`,
      }}
    />
  );
}

/* The sleeve, full bleed down the left.
 *
 * A single radial mask does both jobs the reference does: it holds the artwork
 * solid through the left edge and the middle, then feathers it out across the
 * right and softens all four corners, so the drifting blur underneath comes
 * through exactly where the lyrics are. One mask on a static image rasterises
 * once - it is not a filter and it costs nothing per frame. */
function Sleeve({
  url, alt, canvas, onVideo,
}: {
  url: string | null | undefined;
  alt: string;
  canvas?: Canvas | null;
  onVideo?: (v: HTMLVideoElement | null) => void;
}) {
  const src = coverUrl(url, 900) ?? url;
  const reduceMotion = useReducedMotion();
  /* the canvas cdn sends CORS headers, so the video loads as anonymous and its
     frames can be read for the room's colours. if one ever doesn't, the load
     fails - retry without it (the video still plays, the colours stay the
     cover's) */
  const [cors, setCors] = useState(true);
  useEffect(() => setCors(true), [canvas?.url]);
  if (!src && !canvas) return null;
  // a still canvas is just a better picture; a moving one respects reduced motion
  const video = canvas && canvas.kind !== "image" && !reduceMotion ? canvas.url : null;
  const still = canvas?.kind === "image" ? canvas.url : src;

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
     stays solid to about 29% and has dissolved completely by about 60%. */
  /* eased out, not linear: a straight ramp from solid to clear reads as a
     band where it starts and ends, worst on a pale sleeve over a dark room */
  "radial-gradient(ellipse 90% 150% at 11.5% 50%, #000 0%, #000 38%, rgba(0,0,0,0.86) 48%, rgba(0,0,0,0.62) 58%, rgba(0,0,0,0.38) 67%, rgba(0,0,0,0.18) 76%, rgba(0,0,0,0.06) 85%, transparent 94%)",
  // the window's top-left corner
  "radial-gradient(ellipse 44% 36% at 0% 0%, transparent 6%, #000 94%)",
  // and its bottom-left
  "radial-gradient(ellipse 44% 36% at 0% 100%, transparent 6%, #000 94%)",
].join(", ");

  return (
    <div aria-hidden style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: "62%", overflow: "hidden", pointerEvents: "none" }}>
      <AnimatePresence initial={false}>
        {video ? (
          <motion.video
            key={`${video}-${cors}`}
            ref={onVideo}
            src={video}
            crossOrigin={cors ? "anonymous" : undefined}
            onError={() => { if (cors) setCors(false); }}
            autoPlay
            loop
            muted
            playsInline
            poster={src ?? undefined}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
            style={{
              position: "absolute", inset: 0,
              width: "100%", height: "100%", objectFit: "cover", objectPosition: "center",
              WebkitMaskImage: SLEEVE_MASK,
              maskImage: SLEEVE_MASK,
              maskComposite: "intersect",
            }}
          />
        ) : (
        <motion.img
          key={still ?? undefined}
          src={still ?? undefined}
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
        )}
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
  const autoHideSublyrics = usePrefsStore((s) => s.autoHideSublyrics);
  const { data, isLoading } = useLyrics(track);
  const { seek } = usePlayerControls();

  const rows = useMemo(() => withInterludes(buildRows(data)), [data]);
  const rowStarts = useMemo(() => rows.map((r) => r.startMs), [rows]);
  const synced = !!data?.lines.length;

  const { getClock, resync } = useLyricClock();
  const { active, overlapping, singing } = useActiveLyrics(rows, getClock, synced);
  const distances = useMemo(() => rowDistances(rows, active), [rows, active]);

  const followTrackId = usePlayerStore((s) => s.currentTrack?.id);
  const scrollRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLDivElement | null)[]>([]);
  const moreContrast = useMoreContrast();

  /* Where the active line rests: its middle about 30% down the column.
     Centring rather than top-aligning is what makes it size-aware: a line
     that wraps to three rows grows in both directions instead of pushing its
     tail off the bottom. The clamp is for the extreme case, where a line long
     enough to fill half the column would still ride up under the header.

     Measured from the column itself now. The old maths read `offsetTop`
     against a positioned ancestor about a card-header above the scroller, so
     its nominal "42%" (and "16%" clamp) actually landed the line near 28-30%
     of the column - which is where it has been sitting, and where it stays.
     The column's wrapper is positioned (the return pill hangs off it), which
     makes offsetTop exact. */
  const targetFor = useCallback((el: HTMLDivElement, cont: HTMLDivElement) => {
    const H = cont.clientHeight;
    const top = el.offsetTop - cont.offsetTop;
    return Math.min(
      top - H * 0.3 + el.clientHeight / 2,
      top - H * 0.04,
    );
  }, []);

  // first placement jumps, line-to-line springs, and the reader can take over
  const { detached, recenter } = useLyricFollow({
    scrollRef, rowRefs, active, overlapping, resetKey: rows, trackKey: followTrackId, targetFor, reduceMotion, layoutKey: autoHideSublyrics,
  });

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
    <div style={{ position: "relative", height: "100%" }}>
    <div ref={scrollRef} className="lyr-scroll gpu-scroll" style={lyricsScroll}>
      {rows.map((row, ri) => {
        const isActive = synced && ri === active;
        const tone = synced ? lyricTone(distances[ri], ri < active, moreContrast) : { blur: 0, alpha: 0.9 };
        const layout = voiceLayout(row);
        const ease = "0.45s cubic-bezier(0.22, 1, 0.36, 1)";
        return (
          <div
            key={ri}
            ref={(el) => { rowRefs.current[ri] = el; }}
            onClick={() => seekTo(ri)}
            className="lyr-row"
            data-seekable={synced}
            data-active={isActive}
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
              // an interlude carries its own padding inside the fold
              padding: row.interlude ? 0 : "11px 0",
              // Voice emphasis animates below; `scale` here is only the press.
              transition: reduceMotion ? undefined : "scale 0.12s cubic-bezier(0.23, 1, 0.32, 1)",
              display: "flex", flexDirection: "column", ["--lyr-voice-gap" as string]: "6px",
            }}
          >
            {row.interlude ? (
              <LyricInterlude
                startMs={row.startMs}
                endMs={row.endMs}
                active={isActive}
                getClock={getClock}
                size="clamp(2rem, 0.4rem + 2.75vw, 4.15rem)"
                pad="14px"
                inkRgb={ink}
                glowRgb={glow}
              />
            ) : row.voices.map((voice, vi) => {
              // lead left, duet right, a backing vocal on the side of the
              // singer it's under (lib/lyricsRows voiceLayout)
              const { side: align, secondary: isSecondary } = layout[vi];
              const voiceActive = voice.words.length || voice.end_ms != null
                ? singing.some((v) => v.row === ri && v.voice === vi)
                : isActive;
              const voiceTone = voiceActive ? lyricTone(0, false, moreContrast) : tone;
              const collapsed = synced && isSecondary && autoHideSublyrics && !voiceActive;
              /* Each voice shrinks toward its OWN side. The whole row used to
                 scale from one origin - left unless the row was a lone duet
                 line - so a right-aligned duet line with a backing vocal
                 shrank away from the right edge and floated mid-screen.
                 Vertically, a stacked pair draws toward the seam between them,
                 as the row-level scale did, instead of spreading apart. */
              const n = row.voices.length;
              const originY = n === 1 ? "center" : vi === 0 ? "bottom" : vi === n - 1 ? "top" : "center";

              /* rem-anchored, so the user's text size carries through; the vw
                 term still lets the column breathe with the window. The lead
                 tops out at ~58px, where Cider sets it on a wide window. */
              const size = isSecondary
                ? "clamp(1.5rem, 0.4rem + 1.95vw, 2.9rem)"
                : "clamp(2rem, 0.4rem + 2.75vw, 4.15rem)";
              const weight = isSecondary ? 700 : 800;
              // built for every row, not just the lit one: both states render
              // the same spans so the line can never re-wrap when it lights up
              const words = lyricWords(voice, row.startMs, row.endMs);
              return (
                <div
                  key={vi}
                  className="lyr-voice"
                  data-collapsed={collapsed}
                  aria-hidden={collapsed || undefined}
                  data-lyric-voice={vi}
                  data-lyric-secondary={isSecondary}
                  data-active={voiceActive}
                  style={{ opacity: collapsed ? 0 : voiceTone.alpha * (isSecondary ? 0.78 : 1) }}
                >
                  <div
                    style={{
                      // a backing vocal hangs off its singer's side: indented
                      // from that edge, with the rule on that edge
                      ...(isSecondary ? secondaryInset(align, 18, 12) : null),
                      textAlign: align,
                      transform: voiceActive ? "scale(1)" : "scale(0.79)",
                      transformOrigin: `${align} ${originY}`,
                      transition: reduceMotion ? undefined : `transform ${ease}`,
                    }}
                  >
                    <LyricRowText
                      words={words}
                      active={voiceActive}
                      getClock={getClock}
                      tone={voiceTone}
                      size={size}
                      weight={weight}
                      glowRgb={glow}
                      inkRgb={ink}
                      align={align}
                      tracking={isSecondary ? "-0.01em" : "-0.022em"}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
    <ReturnPill show={synced && detached} onClick={recenter} />
    </div>
  );
}

// the backing-vocal rule and indent, on whichever side the voice sits
function secondaryInset(side: "left" | "right", indent: number, pad: number): React.CSSProperties {
  const rule = "2px solid rgba(255,255,255,0.18)";
  return side === "right"
    ? { marginRight: indent, borderRight: rule, paddingRight: pad }
    : { marginLeft: indent, borderLeft: rule, paddingLeft: pad };
}

const LYRIC_MASK =
  "linear-gradient(to bottom, transparent 0, #000 9%, #000 calc(84% - var(--lyr-dock, 0px)), transparent calc(100% - var(--lyr-dock, 0px)))";

/* The room is set in Inter (self-hosted in fonts.css) while the rest of the app
   uses Host Grotesk - a heavier, tighter face for big lyrics. cv11/ss01 are
   Inter's single-storey a and open digits; tracking matches the lyric lines. */
const IMMERSIVE_TYPE: React.CSSProperties = {
  fontFamily: '"Inter", "Segoe UI Variable", "Segoe UI", system-ui, -apple-system, sans-serif',
  fontFeatureSettings: '"cv11", "ss01"',
  letterSpacing: "-0.011em",
};

const lyricsScroll: React.CSSProperties = {
  height: "100%", overflowY: "auto", overflowX: "hidden",
  /* Real gutters. The text used to sit 8px off the panel edge and then get
     scaled 12% wider from that edge, so long lines ran off the right of the
     window entirely. */
  padding: "10vh clamp(16px, 1.8vw, 30px) 32vh clamp(14px, 1.6vw, 26px)",
  /* The bottom fade ends at the dock's top edge, or the window's bottom while
     the dock is hidden (--lyr-dock, set by Immersive). */
  WebkitMaskImage: LYRIC_MASK,
  maskImage: LYRIC_MASK,
};


function Centered({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ height: "100%", display: "flex", alignItems: "center", justifyContent: "center", textAlign: "center", color: "rgba(255,255,255,0.66)", fontSize: "1.07rem", padding: 24 }}>
      {children}
    </div>
  );
}

// immersive view

export function Immersive() {
  const open     = usePlayerStore((s) => s.immersiveOpen);
  const setOpen  = usePlayerStore((s) => s.setImmersiveOpen);
  const track    = usePlayerStore((s) => s.currentTrack);
  // the same cached read AmbientBg does: the lyric ink and glow come off the
  // cover, so the type is lit by the record it belongs to
  const coverAmbient = useAmbient(track?.album?.image_url);
  // only fetch while the view is actually up
  const canvas = useCanvas(open ? track?.id : null);
  const [videoEl, setVideoEl] = useState<HTMLVideoElement | null>(null);
  const podcast = isEpisodeId(track?.id);
  const liveVideo = podcast || (canvas && canvas.kind !== "image") ? videoEl : null;
  // colours for the lyric ink/glow still come from periodic reads; the backdrop itself is live
  const videoAmbient = useVideoAmbient(liveVideo);
  // lyric ink and glow follow the video too, so type and room stay one hue
  const { glow, ink } = videoAmbient ?? coverAmbient;
  const reduceMotion = useReducedMotion();

  /* The controls step back while you just listen: the dock and the exit
     button fade out a moment after the mouse stops and come back the instant
     it moves. Paused, they stay - a stopped player with no visible way to
     start it again reads as broken. The cursor goes with them. */
  const playing = usePlayerStore((s) => s.isPlaying);
  const idle = useIdleControls(open);
  const hideControls = idle && playing;
  const controlsFade: React.CSSProperties = {
    opacity: hideControls ? 0 : 1,
    pointerEvents: hideControls ? "none" : undefined,
    // out slow, back fast: waking should feel instant, leaving should not be noticed
    transition: hideControls
      ? `opacity ${HIDE_S}s ${HIDE_EASE}, transform ${HIDE_S}s ${HIDE_EASE}`
      : `opacity ${SHOW_S}s ease-out, transform ${SHOW_S}s ease-out`,
  };

  /* The lyrics run the full height of the column, under the dock. Their fade
     out ends at the dock's top edge while it is up (--lyr-dock, the dock's
     height plus the gap) and slides down to the window's bottom once it hides,
     on the dock's own timing - so the space it leaves is filled by lyrics, not
     left empty. The scroller's own size never changes, so nothing reflows and
     the sung line does not move. */
  const dockRef = useRef<HTMLDivElement>(null);
  const [dockH, setDockH] = useState(0);
  useEffect(() => {
    const el = dockRef.current;
    if (!open || !el) return;
    const ro = new ResizeObserver(() => setDockH(el.offsetHeight));
    ro.observe(el);
    setDockH(el.offsetHeight);
    return () => ro.disconnect();
  }, [open, !!track]);

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
          // out faster than in: Esc is a keyboard exit, it should get out of the way
          exit={{ opacity: 0, transition: { duration: 0.18, ease: EASE_OUT } }}
          transition={{ duration: 0.26, ease: [0.22, 1, 0.36, 1] }}
          style={{ position: "fixed", inset: 0, zIndex: 900, overflow: "hidden", color: "#fff", background: "#07070b", cursor: hideControls ? "none" : undefined, ...IMMERSIVE_TYPE }}
        >
          <AmbientBg url={track.album?.image_url} override={videoAmbient} />
          {podcast ? (
            <PodcastStage track={track} onVideo={setVideoEl} />
          ) : (
            <Sleeve url={track.album?.image_url} alt={track.name} canvas={canvas} onVideo={setVideoEl} />
          )}
          <LyricShade
            base={(videoAmbient ?? coverAmbient).base}
            light={(videoAmbient ?? coverAmbient).light}
            ready={!!videoAmbient || coverAmbient.ready}
          />

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
                position: "relative",
                width: "min(50%, 860px)",
                minWidth: 360,
                display: "flex",
                flexDirection: "column",
                minHeight: 0,
                ["--lyr-dock" as string]: hideControls ? "0px" : `calc(${dockH}px + clamp(12px, 1.6vh, 20px))`,
                transition: hideControls ? `--lyr-dock ${HIDE_S}s ${HIDE_EASE}` : `--lyr-dock ${SHOW_S}s ease-out`,
              }}
            >
              {/* The lyrics sit straight on the room - no card, no tabs. The
                  exit button keeps its old spot at the top right of the column. */}
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                /* leaves the way the dock came in: settling back down toward
                   the bar the view was opened from */
                exit={{ opacity: 0, transform: "translateY(10px)", transition: { duration: 0.18, ease: EASE_OUT } }}
                transition={{ duration: 0.28, delay: 0.04, ease: [0.22, 1, 0.36, 1] }}
                style={{
                  pointerEvents: "auto",
                  flex: 1,
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                }}
              >
                <div data-idle-hold style={{ display: "flex", justifyContent: "flex-end", alignSelf: "flex-end", ...controlsFade }}>
                  <button
                    className="imm-close"
                    onClick={() => setOpen(false)}
                    title="Exit immersive view (Esc)"
                    aria-label="Exit immersive view"
                  >
                    <Minimize2 size={16} strokeWidth={2.2} />
                  </button>
                </div>
                {/* inset from the column's left edge, so the lyrics sit further
                    right than the dock below them */}
                <div style={{ flex: 1, minHeight: 0, position: "relative", marginLeft: "clamp(24px, 6vw, 120px)" }}>
                  {podcast ? (
                    <TranscriptView episodeId={track.id} size="stage" ink={ink} />
                  ) : (
                    <ImmersiveLyrics glow={glow} ink={ink} />
                  )}
                </div>
              </motion.div>

              {/* The app's own dock, minus the 44px sleeve - it is already
                  filling the screen behind this. Same component, so transport,
                  scrubber, volume, devices and queue all behave identically. */}
              <motion.div
                initial={{ opacity: 0, transform: "translateY(14px)" }}
                animate={{ opacity: 1, transform: "none" }}
                // the same path out as in
                exit={{ opacity: 0, transform: "translateY(14px)", transition: { duration: 0.18, ease: EASE_OUT } }}
                transition={{ duration: 0.3, delay: 0.06, ease: [0.22, 1, 0.36, 1] }}
                style={{ position: "absolute", left: 0, right: 0, bottom: 0 }}
              >
                <div
                  ref={dockRef}
                  data-idle-hold
                  style={{ ...controlsFade, transform: hideControls && !reduceMotion ? "translateY(10px)" : "none" }}
                >
                  <PlayerBar immersive />
                </div>
              </motion.div>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}


