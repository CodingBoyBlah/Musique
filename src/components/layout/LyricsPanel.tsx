import { useEffect, useMemo, useRef, useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import {
  Languages,
  Music2,
  RefreshCw,
} from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useLyrics } from "../../hooks/useLyrics";
import { seekPlayback } from "../../api/playback";
import { Loader } from "../ui/Loader";
import { Tooltip } from "../ui/Tooltip";
import { isMac } from "../../lib/platform";
import { zTransform } from "../../lib/motion";
import {
  detectLyricScript,
  canRomanize,
  scriptLabel,
  romanizeLines,
} from "../../utils/romanize";
import {
  ActiveLine,
  buildRows,
  mapWords,
  useActiveRow,
  useLyricClock,
  type Row,
} from "../../lib/lyrics";

const WIDTH = 366;

// panel

export function LyricsPanel() {
  const track = usePlayerStore((s) => s.currentTrack);
  const setPosition = usePlayerStore((s) => s.setPosition);
  const reduceMotion = useReducedMotion();

  const { data, isLoading, isError, isFetching, refetch } = useLyrics(track);

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
  const canPron = canRomanize(script);

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

  // auto-scroll the active row to about 40%
  const scrollRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLDivElement | null)[]>([]);
  useEffect(() => {
    if (active < 0) return;
    const el = rowRefs.current[active];
    const cont = scrollRef.current;
    if (!el || !cont) return;
    cont.scrollTo({
      top: el.offsetTop - cont.clientHeight * 0.4 + el.clientHeight / 2,
      behavior: reduceMotion ? "auto" : "smooth",
    });
  }, [active, reduceMotion]);

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
      initial={{ opacity: 0, x: 60, scale: 0.96 }}
      animate={{ opacity: 1, x: 0, scale: 1 }}
      exit={{ opacity: 0, x: 60, scale: 0.96 }}
      transformTemplate={zTransform}
      transition={{ type: "spring", stiffness: 320, damping: 32 }}
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
        {/* header - only pronunciation toggle if applicable */}
        {canPron && (
          <div
            style={{
              position: "relative",
              zIndex: 2,
              flexShrink: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "flex-end",
              padding: isMac ? "4px 12px 0" : "4px 140px 0 14px",
              height: 36,
            }}
          >
            <Tooltip
              label={
                pron ? "Hide pronunciation" : `Show ${scriptLabel(script)}`
              }
              side="bottom"
            >
              <motion.button
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                transition={{ type: "spring", stiffness: 450, damping: 25 }}
                onClick={() => setPron((v) => !v)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 5,
                  height: 28,
                  padding: "0 10px",
                  borderRadius: 99,
                  cursor: "pointer",
                  border: "none",
                  background: pron
                    ? "var(--color-accent)"
                    : "rgba(255, 255, 255, 0.14)",
                  color: pron ? "var(--color-accent-text, #ffffff)" : "#ffffff",
                  fontSize: 11.5,
                  fontWeight: 650,
                  outline: "none",
                  boxShadow: "0 2px 8px rgba(0, 0, 0, 0.25)",
                }}
              >
                <Languages size={13} strokeWidth={2.4} />
                <span>{scriptLabel(script)}</span>
              </motion.button>
            </Tooltip>
          </div>
        )}

        {/* lyrics body */}
        <div
          ref={scrollRef}
          data-selectable
          className="scroll-y"
          style={{
            position: "relative",
            zIndex: 1,
            flex: 1,
            overflowY: "auto",
            overflowX: "hidden",
            padding: "26px 18px 40vh",
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
                const multi = row.voices.length > 1;
                return (
                  <div
                    key={ri}
                    ref={(el) => {
                      rowRefs.current[ri] = el;
                    }}
                    onClick={() => seekTo(ri)}
                    style={{
                      padding: isActive ? "8px 10px" : "4px 10px",
                      margin: isActive ? "3px 0" : "0",
                      borderRadius: 10,
                      cursor: synced ? "pointer" : "default",
                      transition:
                        "opacity 0.35s ease, transform 0.48s cubic-bezier(0.16, 1, 0.3, 1), filter 0.35s ease",
                      transform: isActive ? "scale(1.09)" : "scale(0.96)",
                      transformOrigin: "left center",
                      opacity: !synced
                        ? 0.92
                        : isActive
                          ? 1
                          : ri < active
                            ? 0.28
                            : 0.42,
                      filter: !synced || isActive ? "none" : "blur(1.6px)",
                      contentVisibility: "auto",
                      containIntrinsicSize: "0 40px",
                      display: "flex",
                      flexDirection: "column",
                      gap: multi ? 3 : 0,
                      width: "100%",
                      minWidth: 0,
                      boxSizing: "border-box",
                    }}
                    onMouseEnter={(e) => {
                      if (synced && !isActive)
                        (e.currentTarget as HTMLDivElement).style.background =
                          "rgba(255,255,255,0.05)";
                    }}
                    onMouseLeave={(e) => {
                      (e.currentTarget as HTMLDivElement).style.background =
                        "transparent";
                    }}
                  >
                    {row.voices.map((voice, vi) => {
                      // secondary voices (backing vocals) read smaller + indented
                      const size = vi === 0 ? 25 : 19;
                      const weight = vi === 0 ? 800 : 700;
                      const indent = vi === 0 ? 0 : 16;
                      const romIdx = rowOffsets[ri] + vi;
                      return (
                        <div
                          key={vi}
                          style={{
                            marginLeft: indent,
                            borderLeft:
                              vi === 0
                                ? "none"
                                : "2px solid rgba(255,255,255,0.18)",
                            paddingLeft: vi === 0 ? 0 : 8,
                            width: "100%",
                            minWidth: 0,
                            boxSizing: "border-box",
                          }}
                        >
                          {isActive && voice.words.length ? (
                            // word-by-word (musixmatch ANDOR netease real timings)
                            <ActiveLine
                              words={mapWords(voice)}
                              getClock={getClock}
                              size={size}
                              weight={weight}
                              halo={0.35}
                            />
                          ) : isActive ? (
                            // line level source (LRCLIB) = whole line lit, no word sweep, no estimation

                            <p
                              style={{
                                margin: 0,
                                fontSize: size,
                                lineHeight: 1.3,
                                letterSpacing: "-0.01em",
                                fontWeight: weight,
                                color: "var(--color-text-hi)",
                                textShadow: "0 0 32px rgba(255,255,255,0.45), 0 2px 10px rgba(0,0,0,0.3)",
                                whiteSpace: "pre-wrap",
                                wordBreak: "break-word",
                                overflowWrap: "break-word",
                              }}
                            >
                              {voice.text || "♪"}
                            </p>
                          ) : (
                            <p
                              style={{
                                margin: 0,
                                fontSize: size,
                                lineHeight: 1.3,
                                letterSpacing: "-0.01em",
                                fontWeight: weight,
                                color: "rgba(255,255,255,0.72)",
                                whiteSpace: "pre-wrap",
                                wordBreak: "break-word",
                                overflowWrap: "break-word",
                              }}
                            >
                              {voice.text || "♪"}
                            </p>
                          )}
                          {pron && (
                            <p
                              style={{
                                margin: "2px 0 0",
                                fontSize: 12.5,
                                fontWeight: 600,
                                color: isActive
                                  ? "rgba(255,255,255,0.7)"
                                  : "rgba(255,255,255,0.4)",
                                whiteSpace: "pre-wrap",
                                wordBreak: "break-word",
                                overflowWrap: "break-word",
                              }}
                            >
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
                  color: "rgba(255,255,255,0.28)",
                }}
              >
                {synced
                  ? data?.word_level
                    ? "Word-by-word"
                    : "Synced"
                  : "Lyrics"}{" "}
                ·{" "}
                {data?.source === "musixmatch"
                  ? "Musixmatch"
                  : data?.source === "netease"
                    ? "NetEase"
                    : "LRCLIB"}
              </p>
            </div>
          )}
        </div>
      </div>
    </motion.div>
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
