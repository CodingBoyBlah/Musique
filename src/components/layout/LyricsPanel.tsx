import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import {
  X,
  Languages,
  Music2,
  RefreshCw,
  Minus,
  Plus,
  Clock,
} from "lucide-react";
import { usePlayerStore } from "../../store/player.store";
import { useLyrics } from "../../hooks/useLyrics";
import { seekPlayback } from "../../api/playback";
import { Loader } from "../ui/Loader";
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

// living background: the actual cover art, blurred + drifting TODO - reuse in immersive - DONEN

function CoverBg({ url }: { url: string | null | undefined }) {
  const reduceMotion = useReducedMotion();
  if (!url) {
    return (
      <div
        aria-hidden
        style={{
          position: "absolute",
          inset: 0,
          background: "rgba(10,10,18,0.30)",
          zIndex: 0,
        }}
      />
    );
  }
  /* perf: two translucent blurred copies of the cover, drifting so this song's
  colours churn like a slow living gradient (APLE MUSIC MOBILE). only translate/rotate animated
   (NEVER scale or filter) so the heavy blur rasterizes once and just
   composites after - animating scale re-blurs every frame and was a real jank
   source stays translucent so the os Mica/acrylic reads through.
   no transform here, framer owns it scale goes through framer (constant) so
   the blur rasterizes once and only translate/rotate composite per frame */

/* UPDATE REUSED IN IMMERSIVE LYRICS PANEL TOO, TODO DONE */
  const layer = (opacity: number): React.CSSProperties => ({
    position: "absolute",
    inset: "-35%",
    backgroundImage: `url(${url})`,
    backgroundSize: "cover",
    backgroundPosition: "center",
    filter: "blur(52px) saturate(1.9)",
    opacity,
    willChange: "transform",
    backfaceVisibility: "hidden",
  });
  return (
    <div
      aria-hidden
      style={{ position: "absolute", inset: 0, overflow: "hidden", zIndex: 0 }}
    >
      <motion.div
        initial={false}
        animate={reduceMotion ? { scale: 1.45 } : { scale: 1.45, x: [0, 54, -38, 0], y: [0, -42, 32, 0], rotate: [0, 6, -5, 0] }}
        transition={{ duration: 26, repeat: Infinity, ease: "easeInOut" }}
        style={layer(0.28)}
      />
      <motion.div
        initial={false}
        animate={reduceMotion ? { scale: 1.7 } : { scale: 1.7, x: [0, -48, 40, 0], y: [0, 36, -30, 0], rotate: [0, -7, 5, 0] }}
        transition={{ duration: 38, repeat: Infinity, ease: "easeInOut" }}
        style={layer(0.26)}
      />
      {/* legibility scrim = darken header + bottom for text contrast; middle
          stays light so the moving colour appears through clearly */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background:
            "linear-gradient(180deg, rgba(6,6,10,0.42) 0%, rgba(6,6,10,0.10) 18%, rgba(6,6,10,0.08) 78%, rgba(6,6,10,0.48) 100%)",
        }}
      />
    </div>
  );
}

// panel

export function LyricsPanel() {
  const setLyricsOpen = usePlayerStore((s) => s.setLyricsOpen);
  const track = usePlayerStore((s) => s.currentTrack);
  const setPosition = usePlayerStore((s) => s.setPosition);
  const offset = usePlayerStore((s) => s.lyricsOffsetMs);
  const adjustOffset = usePlayerStore((s) => s.adjustLyricsOffset);
  const setOffset = usePlayerStore((s) => s.setLyricsOffset);
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

  // TODO sync controls live in a popover off the clock icon (not always on screen) (DONE)
  const [syncOpen, setSyncOpen] = useState(false);
  useEffect(() => {
    setSyncOpen(false);
  }, [track?.id]);

  return (
    <motion.div
      initial={{ width: 0, opacity: 0 }}
      animate={{ width: WIDTH, opacity: 1 }}
      exit={{ width: 0, opacity: 0 }}
      transition={{
        type: "spring",
        stiffness: 360,
        damping: 38,
        opacity: { duration: 0.18 },
      }}
      style={{
        flexShrink: 0,
        overflow: "hidden",
        borderLeft: "1px solid var(--color-border)",
        
        background: "rgba(18,18,26,0.30)",
        backdropFilter: "blur(40px) saturate(1.7)",
        WebkitBackdropFilter: "blur(40px) saturate(1.7)",
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
        <CoverBg url={track?.album?.image_url} />

        {/* header */}
        <div
          style={{
            position: "relative",
            zIndex: 2,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            gap: 10,
            padding: "0 12px",
            height: 48,
            borderBottom: "1px solid rgba(255,255,255,0.08)",
          }}
        >
          <span
            style={{
              fontSize: 13,
              fontWeight: 700,
              color: "var(--color-text-hi)",
            }}
          >
            Lyrics
          </span>
          <div style={{ flex: 1 }} />
          {canPron && (
            <button
              onClick={() => setPron((v) => !v)}
              title={
                pron ? "Hide pronunciation" : `Show ${scriptLabel(script)}`
              }
              style={{
                display: "flex",
                alignItems: "center",
                gap: 5,
                height: 28,
                padding: "0 10px",
                borderRadius: 99,
                cursor: "pointer",
                border: pron
                  ? "1px solid transparent"
                  : "1px solid rgba(255,255,255,0.16)",
                background: pron ? "var(--color-accent)" : "transparent",
                color: pron ? "#fff" : "var(--color-text)",
                fontSize: 11.5,
                fontWeight: 600,
              }}
            >
              <Languages size={13} strokeWidth={2.2} /> {scriptLabel(script)}
            </button>
          )}
          {synced && (
            <button
              onClick={() => setSyncOpen((v) => !v)}
              title="Adjust sync"
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: 26,
                height: 26,
                borderRadius: 6,
                border: "none",
                background: syncOpen ? "rgba(255,255,255,0.12)" : "transparent",
                color: syncOpen ? "var(--color-text-hi)" : "var(--color-text)",
                cursor: "pointer",
                flexShrink: 0,
              }}
              onMouseEnter={(e) => {
                if (!syncOpen)
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "rgba(255,255,255,0.08)";
              }}
              onMouseLeave={(e) => {
                if (!syncOpen)
                  (e.currentTarget as HTMLButtonElement).style.background =
                    "transparent";
              }}
            >
              <Clock size={14} strokeWidth={2.2} />
            </button>
          )}
          <button
            onClick={() => setLyricsOpen(false)}
            title="Close"
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: 26,
              height: 26,
              borderRadius: 6,
              border: "none",
              background: "transparent",
              color: "var(--color-text)",
              cursor: "pointer",
              flexShrink: 0,
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background =
                "rgba(255,255,255,0.08)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.background =
                "transparent";
            }}
          >
            <X size={14} strokeWidth={2.2} />
          </button>
        </div>

        {/* sync calibration popover (off the clock icon) */}
        {synced && syncOpen && (
          <>
            <div
              onClick={() => setSyncOpen(false)}
              style={{ position: "absolute", inset: 0, zIndex: 5 }}
            />
            <motion.div
              initial={{ opacity: 0, y: -6, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ duration: 0.16, ease: [0.23, 1, 0.32, 1] }}
              style={{
                position: "absolute",
                top: 46,
                right: 8,
                zIndex: 6,
                width: 304,
                padding: "11px 13px 12px",
                borderRadius: 12,
                background: "rgba(18,18,24,0.92)",
                backdropFilter: "blur(24px) saturate(1.4)",
                WebkitBackdropFilter: "blur(24px) saturate(1.4)",
                border: "1px solid rgba(255,255,255,0.12)",
                boxShadow: "0 14px 44px rgba(0,0,0,0.5)",
              }}
            >
              <SyncBar
                offset={offset}
                adjust={adjustOffset}
                setOffset={setOffset}
              />
            </motion.div>
          </>
        )}

        {/* lyrics body */}
        <div
          ref={scrollRef}
          data-selectable
          style={{
            position: "relative",
            zIndex: 1,
            flex: 1,
            overflowY: "auto",
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
            <div style={{ display: "flex", flexDirection: "column", gap: 9 }}>
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
                      padding: "5px 8px",
                      borderRadius: 9,
                      cursor: synced ? "pointer" : "default",
                      transition:
                        "opacity 0.32s ease, transform 0.4s cubic-bezier(0.23,1,0.32,1)",
                      transform: isActive ? "scale(1.015)" : "scale(1)",
                      transformOrigin: "left center",
                      opacity: !synced
                        ? 0.9
                        : isActive
                          ? 1
                          : ri < active
                            ? 0.32
                            : 0.5,
                      // stack concurrent voices TODO
                      display: "flex",
                      flexDirection: "column",
                      gap: multi ? 3 : 0,
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
                      const size = vi === 0 ? 21 : 17;
                      const weight = vi === 0 ? (synced ? 800 : 600) : 700;
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
                          }}
                        >
                          {isActive && voice.words.length ? (
                            // word-by-word (musixmatch ANDOR netease real timings)
                            <ActiveLine
                              words={mapWords(voice)}
                              getClock={getClock}
                              size={size}
                              weight={weight}
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
                                textShadow: "0 0 18px rgba(255,255,255,0.14)",
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
                                color: "rgba(255,255,255,0.82)",
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

// sync calibration

const SYNC_RANGE = 3000; // +- 3s on the slider (store clamps at +-5s)

function SyncBar({
  offset,
  adjust,
  setOffset,
}: {
  offset: number;
  adjust: (d: number) => void;
  setOffset: (ms: number) => void;
}) {
  const label = `${offset >= 0 ? "+" : "−"}${(Math.abs(offset) / 1000).toFixed(2)}s`;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <Clock
          size={13}
          strokeWidth={2}
          style={{ color: "var(--color-text-dim)" }}
        />
        <span
          style={{
            fontSize: 11,
            fontWeight: 600,
            color: "var(--color-text-dim)",
            textTransform: "uppercase",
            letterSpacing: "0.04em",
          }}
        >
          Sync
        </span>
        <span
          style={{
            fontSize: 12,
            fontWeight: 700,
            fontVariantNumeric: "tabular-nums",
            color:
              offset === 0 ? "var(--color-text-dim)" : "var(--color-text-hi)",
          }}
        >
          {label}
        </span>
        <div style={{ flex: 1 }} />
        <button
          onClick={() => setOffset(0)}
          disabled={offset === 0}
          title="Reset to 0"
          style={{
            fontSize: 11,
            fontWeight: 600,
            color:
              offset === 0 ? "rgba(255,255,255,0.25)" : "var(--color-text)",
            background: "none",
            border: "none",
            cursor: offset === 0 ? "default" : "pointer",
            padding: "0 2px",
          }}
        >
          Reset
        </button>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <RepeatBtn onStep={() => adjust(-50)} title="Lyrics later">
          <Minus size={13} strokeWidth={2.4} />
        </RepeatBtn>
        <input
          type="range"
          min={-SYNC_RANGE}
          max={SYNC_RANGE}
          step={10}
          value={Math.max(-SYNC_RANGE, Math.min(SYNC_RANGE, offset))}
          onChange={(e) => setOffset(Number(e.target.value))}
          aria-label="Lyrics sync offset"
          style={{
            flex: 1,
            accentColor: "#fff",
            height: 18,
            cursor: "pointer",
          }}
        />
        <RepeatBtn onStep={() => adjust(50)} title="Lyrics earlier">
          <Plus size={13} strokeWidth={2.4} />
        </RepeatBtn>
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 9.5,
          fontWeight: 600,
          letterSpacing: "0.04em",
          textTransform: "uppercase",
          color: "rgba(255,255,255,0.26)",
          padding: "0 30px",
        }}
      >
        <span>Later</span>
        <span>Earlier</span>
      </div>
    </div>
  );
}

// press and holdhold accelerating stepped so tap = one fine step, hold to ramp up.
function RepeatBtn({
  onStep,
  children,
  title,
}: {
  onStep: () => void;
  children: React.ReactNode;
  title: string;
}) {
  const timer = useRef<number | undefined>(undefined);
  const stop = useCallback(() => {
    if (timer.current !== undefined) {
      window.clearTimeout(timer.current);
      timer.current = undefined;
    }
  }, []);
  const start = useCallback(() => {
    onStep();
    let delay = 340;
    const run = () => {
      onStep();
      delay = Math.max(55, delay * 0.8);
      timer.current = window.setTimeout(run, delay);
    };
    timer.current = window.setTimeout(run, 340);
  }, [onStep]);
  useEffect(() => stop, [stop]);
  return (
    <button
      onPointerDown={start}
      onPointerUp={stop}
      onPointerLeave={stop}
      onPointerCancel={stop}
      title={title}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        width: 26,
        height: 24,
        borderRadius: 6,
        border: "1px solid rgba(255,255,255,0.14)",
        background: "transparent",
        color: "var(--color-text-hi)",
        cursor: "pointer",
        flexShrink: 0,
        touchAction: "none",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background =
          "rgba(255,255,255,0.08)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background = "transparent";
      }}
    >
      {children}
    </button>
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
