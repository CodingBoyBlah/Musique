import { memo, startTransition, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "@/lib/icons";
import { getEpisodeTranscript, type Transcript } from "../../api/podcasts";
import { usePlayerStore } from "../../store/player.store";
import { transportSeek } from "../../hooks/usePlayerControls";
import { EPISODE_PREFIX } from "../../utils/episode";
import { EASE_OUT } from "../../lib/motion";
import type { TranscriptLine } from "../../types/podcast";

// the line being spoken: the last one whose start has passed
export function activeLine(t: Transcript, positionMs: number): number {
  let lo = 0;
  let hi = t.lines.length - 1;
  let best = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (t.lines[mid].start_ms <= positionMs) {
      best = mid;
      lo = mid + 1;
    } else hi = mid - 1;
  }
  return best;
}

export interface Turn {
  speaker: string | null;
  heading: boolean;
  /* each sentence with its index in the flat transcript */
  lines: { line: TranscriptLine; index: number }[];
}

/* group sentences into what reads like a transcript: a new paragraph where
the speaker changes (or a chapter heading sits), everything a speaker says in
between runs on as one block */
export function toTurns(lines: TranscriptLine[]): Turn[] {
  const turns: Turn[] = [];
  let speaker: string | null = null;
  lines.forEach((line, index) => {
    if (line.heading) {
      turns.push({ speaker: null, heading: true, lines: [{ line, index }] });
      return;
    }
    const cur = turns[turns.length - 1];
    const newSpeaker = line.speaker && line.speaker !== speaker;
    if (line.speaker) speaker = line.speaker;
    if (!cur || cur.heading || newSpeaker) {
      turns.push({ speaker: line.speaker ?? speaker, heading: false, lines: [{ line, index }] });
    } else {
      cur.lines.push({ line, index });
    }
  });
  return turns;
}

// a steady colour per speaker, soft enough to sit on any artwork
const SPEAKER_HUES = ["#8ab4ff", "#f5a3c7", "#9be3b4", "#ffd08a", "#c6a8ff", "#7fdcf0"];
function speakerColor(name: string) {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) | 0;
  return SPEAKER_HUES[Math.abs(h) % SPEAKER_HUES.length];
}

/* One speaker's turn, or a chapter heading.

Memoised, and handed only what it needs to draw: `mark` is the sentence being
spoken if it is in here, -1 if the playhead has not reached this paragraph and
Infinity once it has passed it. So a new sentence re-renders the paragraph it
is in (and the one it just left), never the rest of the transcript.

content-visibility lets the browser skip laying out and painting paragraphs
that are off screen; `auto` remembers each one's real height once it has been
shown, so following the playhead does not jump. */
const Paragraph = memo(function Paragraph({
  turn, showSpeaker, synced, mark, dim, ink, big, onSeek,
}: {
  turn: Turn;
  showSpeaker: boolean;
  synced: boolean;
  mark: number;
  dim: boolean;
  ink: string;
  big: boolean;
  onSeek: (ms: number) => void;
}) {
  if (turn.heading) {
    const { line, index } = turn.lines[0];
    return (
      <div data-line={index} style={{ display: "flex", alignItems: "center", gap: 10, margin: big ? "34px 0 18px" : "22px 0 12px" }}>
        <span style={{ height: 1, width: 18, background: `rgba(${ink}, 0.3)` }} />
        <span style={{ fontSize: big ? 13 : 11.5, fontWeight: 700, letterSpacing: "0.06em", textTransform: "uppercase", color: `rgba(${ink}, 0.6)` }}>{line.text}</span>
        <span style={{ height: 1, flex: 1, background: `rgba(${ink}, 0.12)` }} />
      </div>
    );
  }
  const color = turn.speaker ? speakerColor(turn.speaker) : null;
  return (
    <div
      style={{
        contentVisibility: "auto",
        containIntrinsicSize: big ? "auto 160px" : "auto 110px",
        marginBottom: big ? 26 : 16,
        opacity: dim ? 0.9 : 1,
        transition: "opacity 0.4s ease",
      }}
    >
      {showSpeaker && color && (
        <div style={{ display: "flex", alignItems: "center", gap: 7, marginBottom: big ? 8 : 5 }}>
          <span style={{ width: 7, height: 7, borderRadius: 99, background: color, boxShadow: `0 0 10px ${color}` }} />
          <span style={{ fontSize: big ? 13 : 11.5, fontWeight: 650, color: `rgba(${ink}, 0.62)` }}>{turn.speaker}</span>
        </div>
      )}
      <p style={{ margin: 0, fontSize: big ? "clamp(21px, 1.9vw, 28px)" : "16px", lineHeight: big ? 1.42 : 1.55, fontWeight: big ? 650 : 560, letterSpacing: "-0.012em" }}>
        {turn.lines.map(({ line, index }) => {
          const state = !synced ? "static" : index === mark ? "now" : index < mark ? "past" : "next";
          return (
            <span
              key={index}
              data-line={index}
              role={synced ? "button" : undefined}
              tabIndex={synced ? 0 : undefined}
              onClick={synced ? () => onSeek(line.start_ms) : undefined}
              onKeyDown={synced ? (e) => { if (e.key === "Enter") onSeek(line.start_ms); } : undefined}
              className="tr-sentence"
              style={{
                cursor: synced ? "pointer" : "text",
                borderRadius: 6,
                color:
                  state === "now" || state === "static"
                    ? `rgb(${ink})`
                    : state === "past"
                      ? `rgba(${ink}, 0.36)`
                      : `rgba(${ink}, 0.58)`,
                textShadow: state === "now" ? `0 0 22px rgba(${ink}, 0.28)` : "none",
                transition: "color 0.35s ease, text-shadow 0.35s ease",
                userSelect: synced ? "none" : "text",
              }}
            >
              {line.text}{" "}
            </span>
          );
        })}
      </p>
    </div>
  );
});

/* spotify's read-along transcript, as a readable transcript: paragraphs per
speaker, the sentence being said lit inside its paragraph, what's been said
fading back. follows the playhead unless you scroll; click any sentence to jump
there. `size` scales it - the side panel is compact, the immersive view reads
from across the room. */
export function TranscriptView({
  episodeId,
  size = "panel",
  ink = "255, 255, 255",
  empty,
}: {
  episodeId: string;
  size?: "panel" | "stage";
  ink?: string;
  empty?: React.ReactNode;
}) {
  const id = episodeId.startsWith(EPISODE_PREFIX) ? episodeId.slice(EPISODE_PREFIX.length) : episodeId;
  const { data, isLoading } = useQuery({
    queryKey: ["transcript", id],
    queryFn: () => getEpisodeTranscript(id),
    staleTime: Infinity,
    retry: false,
  });
  /* The sentence being spoken, not the playhead. The playhead ticks every
     second, and subscribing to it re-rendered the whole transcript - often
     thousands of sentences - every second, for a highlight that moves every
     few. This re-renders only when the sentence changes, and then only the
     paragraphs whose highlight actually moved (Paragraph is memoised). */
  const current = usePlayerStore((s) => (data && data.synced ? activeLine(data, s.positionMs) : -1));
  const turns = useMemo(() => (data ? toTurns(data.lines) : []), [data]);

  /* Build the paragraphs as an interruptible, low-priority render. The panel
     mounts this on the first frame of its slide; creating every sentence in
     that frame is what made the slide hitch. In a transition React yields to
     the browser every few ms, so the slide keeps its frames and the text
     arrives a beat later. */
  const [shownFor, setShownFor] = useState<Transcript | null>(null);
  useEffect(() => {
    if (data) startTransition(() => setShownFor(data));
  }, [data]);
  const ready = !!data && shownFor === data;

  const scroller = useRef<HTMLDivElement>(null);
  const [userScrolledAt, setUserScrolledAt] = useState(0);
  const [, tick] = useState(0);
  const following = Date.now() - userScrolledAt > 5000;
  // where "now" is relative to the viewport, so the pill points the right way
  const [nowAbove, setNowAbove] = useState(false);

  // re-check "following" once the pause runs out, so the view catches up by itself
  useEffect(() => {
    if (!userScrolledAt) return;
    const t = window.setTimeout(() => tick((n) => n + 1), 5100);
    return () => window.clearTimeout(t);
  }, [userScrolledAt]);

  // the first placement jumps: smooth-scrolling down a long transcript to
  // where the episode already is reads as the panel lagging
  const placed = useRef(false);
  useEffect(() => {
    const box = scroller.current;
    const el = box?.querySelector<HTMLElement>(`[data-line="${current}"]`);
    if (!box || !el) return;
    const first = !placed.current;
    placed.current = true;
    const top = el.offsetTop - box.clientHeight * 0.36;
    if (following) box.scrollTo({ top, behavior: first ? "auto" : "smooth" });
    else setNowAbove(el.offsetTop < box.scrollTop);
  }, [current, following, ready]);

  const seek = useCallback((ms: number) => {
    setUserScrolledAt(0);
    transportSeek(ms);
  }, []);

  const big = size === "stage";

  if (isLoading) {
    return <div style={{ padding: 16, color: `rgba(${ink}, 0.5)`, fontSize: 14 }}>Loading transcript...</div>;
  }
  if (!data) {
    return <>{empty ?? <div style={{ padding: 16, color: `rgba(${ink}, 0.55)`, fontSize: 14 }}>No transcript for this episode.</div>}</>;
  }

  return (
    <div style={{ position: "relative", height: "100%" }}>
      <div
        ref={scroller}
        className="ovs-hide"
        onWheel={() => setUserScrolledAt(Date.now())}
        onTouchMove={() => setUserScrolledAt(Date.now())}
        style={{
          height: "100%",
          overflowY: "auto",
          padding: big ? "18vh 6px 42vh" : "18px 14px 55%",
          maskImage: "linear-gradient(180deg, transparent 0, #000 10%, #000 82%, transparent 100%)",
          WebkitMaskImage: "linear-gradient(180deg, transparent 0, #000 10%, #000 82%, transparent 100%)",
        }}
      >
        {ready && turns.map((turn, ti) => {
          const prev = turns[ti - 1];
          const first = turn.lines[0].index;
          const last = turn.lines[turn.lines.length - 1].index;
          // where the playhead sits relative to this paragraph, collapsed so
          // that a paragraph it is not inside gets the same prop from one
          // sentence to the next, and its memo holds
          const mark = !data.synced ? -1 : current > last ? Infinity : current < first ? -1 : current;
          return (
            <Paragraph
              key={ti}
              turn={turn}
              showSpeaker={!turn.heading && !!turn.speaker && (!prev || prev.heading || prev.speaker !== turn.speaker)}
              synced={data.synced}
              mark={mark}
              dim={data.synced && current >= 0 && !(current >= first && current <= last)}
              ink={ink}
              big={big}
              onSeek={seek}
            />
          );
        })}
      </div>

      <AnimatePresence>
        {!following && data.synced && current >= 0 && (
          <motion.button
            key="back-to-now"
            type="button"
            onClick={() => setUserScrolledAt(0)}
            initial={{ opacity: 0, y: 10, x: "-50%" }}
            animate={{ opacity: 1, y: 0, x: "-50%" }}
            exit={{ opacity: 0, y: 10, x: "-50%", transition: { duration: 0.14 } }}
            transition={{ duration: 0.2, ease: EASE_OUT }}
            className="focus-ring"
            style={{
              position: "absolute",
              left: "50%",
              bottom: big ? 18 : 14,
              display: "inline-flex",
              alignItems: "center",
              gap: 7,
              height: 34,
              padding: "0 14px 0 12px",
              borderRadius: 99,
              border: "1px solid rgba(255,255,255,0.16)",
              background: "rgba(20, 20, 26, 0.72)",
              backdropFilter: "blur(14px)",
              WebkitBackdropFilter: "blur(14px)",
              color: "#fff",
              fontSize: 12.5,
              fontWeight: 650,
              cursor: "pointer",
              boxShadow: "0 10px 28px rgba(0,0,0,0.35)",
            }}
          >
            <ArrowRight size={13} style={{ transform: nowAbove ? "rotate(-90deg)" : "rotate(90deg)" }} />
            Back to now
          </motion.button>
        )}
      </AnimatePresence>
    </div>
  );
}
