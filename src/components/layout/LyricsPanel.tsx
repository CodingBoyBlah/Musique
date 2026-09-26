import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import {
  Globe,
  Languages,
  Music2,
  RefreshCw,
} from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useLyrics } from "../../hooks/useLyrics";
import { sourceLabel } from "../../lib/lyricsSource";
import { useAmbient } from "../../hooks/useAmbient";
import { seekPlayback } from "../../api/playback";
import { Loader } from "../ui/Loader";
import { Tooltip } from "../ui/Tooltip";
import { zTransform, EASE_OUT, PRESS, SPRING_PANEL } from "../../lib/motion";
import { useLyricFollow } from "../../hooks/useLyricFollow";
import { ReturnPill } from "./LyricReturnPill";
import "../../styles/lyrics.css";
import {
  detectLyricScript,
  canRomanize,
  scriptLabel,
  romanizeLines,
} from "../../utils/romanize";
import {
  LyricRowText,
  buildRows,
  voiceLayout,
  lyricTone,
  lyricWords,
  useActiveRow,
  useLyricClock,
  useMoreContrast,
  type Row,
} from "../../lib/lyrics";

const WIDTH = 366;

// panel

export function LyricsPanel() {
  const track = usePlayerStore((s) => s.currentTrack);
  const setPosition = usePlayerStore((s) => s.setPosition);
  const reduceMotion = useReducedMotion();

  const { data, isLoading, isError, isFetching, refetch } = useLyrics(track);
  const { glow, ink } = useAmbient(track?.album?.image_url);

  /* provider-supplied translation / romanization. persisted, and only ever
     offered when the source actually carries them for this track. */
  const showTranslation = usePlayerStore((s) => s.lyricsShowTranslation);
  const setShowTranslation = usePlayerStore((s) => s.setLyricsShowTranslation);
  const showRoman = usePlayerStore((s) => s.lyricsShowRoman);
  const setShowRoman = usePlayerStore((s) => s.setLyricsShowRoman);

  const hasTranslation = !!data?.has_translation;
  const hasRoman = !!data?.has_roman;

  const synced = !!data?.lines.length;

  // build rows via the shared engine (same grouping the Immersive view uses)
  const rows = useMemo<Row[]>(() => buildRows(data), [data]);

  const rowStarts = useMemo(() => rows.map((r) => r.startMs), [rows]);

  // flattened texts (row order) for script detection + romanization indexing
  const flatTexts = useMemo(
    () => rows.flatMap((r) => r.voices.map((v) => v.text)),
    [rows],
  );
  const rowOffsets = useMemo(() => {
    let o = 0;
    return rows.map((r) => {
      const s = o;
      o += r.voices.length;
      return s;
    });
  }, [rows]);

  const script = useMemo(() => detectLyricScript(flatTexts), [flatTexts]);
  /* our own transliteration is a fallback. when the source ships a real
     romanization there is no reason to offer a guess beside it. */
  const canPron = canRomanize(script) && !hasRoman;

  // pronunciation (romaijin/pinyin) - has secondary toggle
  const [pron, setPron] = useState(false);
  const [romaji, setRomaji] = useState<string[] | null>(null);
  const [romanizing, setRomanizing] = useState(false);
  useEffect(() => {
    setRomaji(null);
    setPron(false);
  }, [track?.id]);
  useEffect(() => {
    if (!pron || !canPron || !flatTexts.length || romaji) return;
    let cancelled = false;
    setRomanizing(true);
    romanizeLines(flatTexts, script)
      .then((r) => {
        if (!cancelled) setRomaji(r);
      })
      .catch(() => {
        if (!cancelled) setRomaji(flatTexts.map(() => ""));
      })
      .finally(() => {
        if (!cancelled) setRomanizing(false);
      });
    return () => {
      cancelled = true;
    };
  }, [pron, canPron, flatTexts, script, romaji]);

  // interpolated clock AND active-row tracking shared with the Immersive view
  const { getClock, resync } = useLyricClock();
  const active = useActiveRow(rowStarts, getClock, synced);

  /* keep the active row at about 40%. the same follower the immersive view
     uses: a jump for the first placement, one velocity-keeping spring line to
     line (native smooth scroll picked its own curve and length), and it lets
     go the moment the reader scrolls */
  const followTrackId = usePlayerStore((s) => s.currentTrack?.id);
  const scrollRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLDivElement | null)[]>([]);
  const moreContrast = useMoreContrast();
  const targetFor = useCallback(
    (el: HTMLDivElement, cont: HTMLDivElement) =>
      el.offsetTop - cont.clientHeight * 0.4 + el.clientHeight / 2,
    [],
  );
  const { detached, recenter } = useLyricFollow({
    scrollRef, rowRefs, active, resetKey: rows, trackKey: followTrackId, targetFor, reduceMotion,
  });

  function seekTo(i: number) {
    if (!synced) return;
    const ms = rowStarts[i];
    setPosition(ms);
    resync(ms);
    seekPlayback(ms).catch(() => {});
  }

  const hasLyrics = rows.length > 0;

  return (
    <motion.div
      // Absolute OVERLAY that slides in/out via a transform (x). The layout space
      // is reserved by an in-flow spacer in Layout.tsx (which toggles instantly on
      // open AND close), so the grid reflows in one step and the cards glide via
      // framer `layout` both ways. This panel just slides over that region; its
      // width never animates, so nothing reflows per-frame.
      //
      // Slides along the edge it is docked to and nothing else: the scale it
      // used to carry pulled it off that edge, and the old spring (zeta ~0.89)
      // overshot on a toggle that had no momentum behind it. Critically damped,
      // and out along the same path it came in on.
      initial={{ opacity: 0, x: 60 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 60 }}
      transformTemplate={zTransform}
      transition={SPRING_PANEL}
      style={{
        position: "absolute", top: 0, right: 0, bottom: 0, zIndex: 5,
        width: WIDTH,
        maxWidth: "100vw",
        overflow: "hidden",
        borderLeft: "none",
        background: "transparent",
        boxShadow: "none",
        contain: "paint",
        willChange: "transform",
      }}
    >
      <div
        style={{
          width: WIDTH,
          height: "100%",
          position: "relative",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        {/* header - every control here is conditional on the track actually
            having something to offer, so a plain LRCLIB track shows none of
            them and the panel looks exactly as it always did */}
        {(canPron || hasRoman || hasTranslation) && (
          <div
            style={{
              position: "relative",
              zIndex: 2,
              flexShrink: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "flex-end",
              gap: 6,
              padding: "4px 12px 0 14px",
              height: 36,
            }}
          >

            {hasRoman && (
              <Tooltip
                label={showRoman ? "Hide romanization" : "Show romanization"}
                side="bottom"
              >
                <Pill on={showRoman} onClick={() => setShowRoman(!showRoman)}>
                  <Languages size={13} strokeWidth={2.4} />
                </Pill>
              </Tooltip>
            )}

            {hasTranslation && (
              <Tooltip
                label={showTranslation ? "Hide translation" : "Show translation"}
                side="bottom"
              >
                <Pill
                  on={showTranslation}
                  onClick={() => setShowTranslation(!showTranslation)}
                >
                  <Globe size={13} strokeWidth={2.4} active={showTranslation} />
                </Pill>
              </Tooltip>
            )}

            {canPron && (
              <Tooltip
                label={
                  pron ? "Hide pronunciation" : `Show ${scriptLabel(script)}`
                }
                side="bottom"
              >
                <Pill on={pron} onClick={() => setPron((v) => !v)}>
                  <Languages size={13} strokeWidth={2.4} />
                  <span>{scriptLabel(script)}</span>
                </Pill>
              </Tooltip>
            )}
          </div>
        )}

        {/* lyrics body */}
        <div
          ref={scrollRef}
          className="scroll-y"
          style={{
            position: "relative",
            zIndex: 1,
            flex: 1,
            overflowY: "auto",
            overflowX: "hidden",
            padding: "26px 18px 40vh",
            scrollbarWidth: "none",
            WebkitMaskImage:
              "linear-gradient(to bottom, transparent 0, #000 7%, #000 88%, transparent 100%)",
            maskImage:
              "linear-gradient(to bottom, transparent 0, #000 7%, #000 88%, transparent 100%)",
          }}
        >
          {isLoading ? (
            <Loader label="Finding lyrics" />
          ) : data?.instrumental ? (
            <CenterNote
              icon={<Music2 size={24} />}
              title="Instrumental"
              subtitle="No lyrics for this track."
            />
          ) : isError || !hasLyrics ? (
            <CenterNote
              title="No lyrics found"
              subtitle="LRCLIB has nothing for this track yet."
              action={<RetryBtn busy={isFetching} onClick={() => refetch()} />}
            />
          ) : (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 14,
                width: "100%",
                minWidth: 0,
              }}
            >
              {rows.map((row, ri) => {
                const isActive = synced && ri === active;
                const tone = synced
                  ? lyricTone(Math.abs(ri - active), ri < active, moreContrast)
                  : { blur: 0, alpha: 0.92 };
                const multi = row.voices.length > 1;
                const layout = voiceLayout(row);
                return (
                  <div
                    key={ri}
                    ref={(el) => {
                      rowRefs.current[ri] = el;
                    }}
                    onClick={() => seekTo(ri)}
                    // hover + press live in styles/lyrics.css
                    className="lyr-prow"
                    data-seekable={synced}
                    data-active={isActive}
                    style={{
                      /* Constant box. Growing the active row's padding reflowed
                         the whole list on every line, and scaling it from
                         `left center` grew it rightward without the layout
                         knowing - which is what ran long lines off the edge of
                         this panel. Emphasis is light only now, the same as the
                         immersive view. */
                      padding: "6px 10px",
                      cursor: synced ? "pointer" : "default",
                      transition: reduceMotion
                        ? "opacity 0.2s ease, background-color 0.16s ease"
                        : "opacity 0.45s cubic-bezier(0.22, 1, 0.36, 1), background-color 0.16s ease, scale 0.12s cubic-bezier(0.23, 1, 0.32, 1)",
                      opacity: synced ? tone.alpha : 0.92,
                      contentVisibility: "auto",
                      containIntrinsicSize: "0 40px",
                      display: "flex",
                      flexDirection: "column",
                      gap: multi ? 3 : 0,
                      width: "100%",
                      minWidth: 0,
                      boxSizing: "border-box",
                    }}
                  >
                    {row.voices.map((voice, vi) => {
                      const isDuet = voice.role === "duet";
                      // lead left, duet right, a backing vocal on the side of
                      // the singer it's under (lib/lyricsRows voiceLayout)
                      const { side: align, secondary: isSecondary } = layout[vi];

                      // Lead voice is primary; bg voice is visually subordinate (smaller, lower opacity, indented)
                      // rem (19 / 22 / 25px at the default size), so the user's text size carries through
                      const size = isSecondary ? "1.357rem" : isDuet && vi > 0 ? "1.571rem" : "1.786rem";
                      const weight = isSecondary ? 700 : 800;
                      const romIdx = rowOffsets[ri] + vi;
                      const words = lyricWords(voice, row.startMs, row.endMs);
                      return (
                        <div
                          key={vi}
                          style={{
                            // a backing vocal hangs off its singer's side:
                            // indented from that edge, with the rule on it.
                            // no width: 100% - with the indent as a margin
                            // that ran the box past the panel's edge
                            ...(isSecondary
                              ? align === "right"
                                ? { marginRight: 16, borderRight: "2px solid rgba(255,255,255,0.18)", paddingRight: 8 }
                                : { marginLeft: 16, borderLeft: "2px solid rgba(255,255,255,0.18)", paddingLeft: 8 }
                              : null),
                            opacity: isSecondary ? 0.78 : 1,
                            minWidth: 0,
                            boxSizing: "border-box",
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
                            tracking={isSecondary ? "-0.01em" : undefined}
                          />
                          {/* what the source itself shipped, in the same
                              quiet key as the pronunciation line that was
                              already here - they stack rather than compete */}
                          {showRoman && voice.roman && (
                            <p style={subText(isActive, align)}>{voice.roman}</p>
                          )}
                          {showTranslation && voice.translation && (
                            <p style={subText(isActive, align)}>{voice.translation}</p>
                          )}
                          {pron && (
                            <p style={subText(isActive, align)}>
                              {romaji ? romaji[romIdx] : romanizing ? "…" : ""}
                            </p>
                          )}
                        </div>
                      );
                    })}
                  </div>
                );
              })}
              <p
                style={{
                  margin: "22px 8px 0",
                  fontSize: 11,
                  fontWeight: 600,
                  letterSpacing: "0.04em",
                  textTransform: "uppercase",
                  // was 0.28 - about 2:1, unreadable at 11px over artwork
                  color: "rgba(255,255,255,0.48)",
                }}
              >
                {synced
                  ? data?.word_level
                    ? "Word-by-word"
                    : "Synced"
                  : "Lyrics"}{" "}
                ·{" "}
                {sourceLabel(data?.source)}
              </p>
            </div>
          )}
        </div>
        <ReturnPill show={synced && hasLyrics && detached} onClick={recenter} />
      </div>
    </motion.div>
  );
}


/* the quiet line under a lyric. pronunciation, romanization and translation
all read at the same weight, so stacking two of them never competes with the
lead text above. */
function subText(active: boolean, align: "left" | "right" = "left"): React.CSSProperties {
  return {
    margin: "2px 0 0",
    fontSize: 12.5,
    fontWeight: 600,
    color: active ? "rgba(255,255,255,0.74)" : "rgba(255,255,255,0.52)",
    letterSpacing: "0.004em",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
    overflowWrap: "break-word",
    textAlign: align,
  };
}

/* the panel's one button shape, shared by every header control */
function Pill({
  on,
  onClick,
  children,
}: {
  on: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <motion.button
      className="focus-ring"
      aria-pressed={on}
      whileHover={{ scale: 1.03 }}
      whileTap={PRESS}
      transition={{ duration: 0.12, ease: EASE_OUT }}
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 5,
        height: 28,
        padding: "0 10px",
        borderRadius: 99,
        cursor: "pointer",
        border: "none",
        background: on ? "var(--color-accent)" : "rgba(255, 255, 255, 0.14)",
        color: on ? "var(--color-accent-text, #ffffff)" : "#ffffff",
        fontSize: 11.5,
        fontWeight: 650,
        letterSpacing: "0.004em",
        boxShadow: "0 2px 8px rgba(0, 0, 0, 0.25)",
      }}
    >
      {children}
    </motion.button>
  );
}

function CenterNote({
  icon,
  title,
  subtitle,
  action,
}: {
  icon?: React.ReactNode;
  title: string;
  subtitle?: string;
  action?: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 9,
        minHeight: "50vh",
        textAlign: "center",
        color: "var(--color-text-dim)",
      }}
    >
      {icon && <div style={{ color: "var(--color-text-dim)" }}>{icon}</div>}
      <p
        style={{
          margin: 0,
          fontSize: 15,
          fontWeight: 700,
          color: "var(--color-text-hi)",
        }}
      >
        {title}
      </p>
      {subtitle && <p style={{ margin: 0, fontSize: 12.5 }}>{subtitle}</p>}
      {action}
    </div>
  );
}

function RetryBtn({ busy, onClick }: { busy: boolean; onClick: () => void }) {
  return (
    <button
      className="pressable"
      onClick={onClick}
      disabled={busy}
      style={{
        marginTop: 4,
        display: "flex",
        alignItems: "center",
        gap: 6,
        height: 32,
        padding: "0 14px",
        borderRadius: 99,
        border: "1px solid rgba(255,255,255,0.16)",
        background: "transparent",
        color: "var(--color-text-hi)",
        fontSize: 12.5,
        fontWeight: 600,
        cursor: busy ? "default" : "pointer",
      }}
    >
      <RefreshCw
        size={13}
        strokeWidth={2.2}
        style={{ animation: busy ? "spin 0.8s linear infinite" : undefined }}
      />
      {busy ? "Searching…" : "Try again"}
    </button>
  );
}
