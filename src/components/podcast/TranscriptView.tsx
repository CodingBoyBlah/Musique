import { useEffect, useMemo, useRef, useState } from "react";
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
  const positionMs = usePlayerStore((s) => s.positionMs);
  const current = useMemo(() => (data && data.synced ? activeLine(data, positionMs) : -1), [data, positionMs]);
  const turns = useMemo(() => (data ? toTurns(data.lines) : []), [data]);

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

  useEffect(() => {
    const box = scroller.current;
    const el = box?.querySelector<HTMLElement>(`[data-line="${current}"]`);
    if (!box || !el) return;
    const top = el.offsetTop - box.clientHeight * 0.36;
    if (following) box.scrollTo({ top, behavior: "smooth" });
    else setNowAbove(el.offsetTop < box.scrollTop);
  }, [current, following]);

  const big = size === "stage";
  const fs = big ? "clamp(21px, 1.9vw, 28px)" : "16px";

  if (isLoading) {
    return <div style={{ padding: 16, color: `rgba(${ink}, 0.5)`, fontSize: 14 }}>Loading transcript...</div>;
  }
  if (!data) {
    return <>{empty ?? <div style={{ padding: 16, color: `rgba(${ink}, 0.55)`, fontSize: 14 }}>No transcript for this episode.</div>}</>;
  }

  const currentTurn = turns.findIndex((t) => t.lines.some((l) => l.index === current));

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
        {turns.map((turn, ti) => {
          if (turn.heading) {
            const { line, index } = turn.lines[0];
            return (
              <div key={ti} data-line={index} style={{ display: "flex", alignItems: "center", gap: 10, margin: big ? "34px 0 18px" : "22px 0 12px" }}>
                <span style={{ height: 1, width: 18, background: `rgba(${ink}, 0.3)` }} />
                <span style={{ fontSize: big ? 13 : 11.5, fontWeight: 700, letterSpacing: "0.06em", textTransform: "uppercase", color: `rgba(${ink}, 0.6)` }}>{line.text}</span>
                <span style={{ height: 1, flex: 1, background: `rgba(${ink}, 0.12)` }} />
              </div>
            );
          }
          const prev = turns[ti - 1];
          const showSpeaker = !!turn.speaker && (!prev || prev.heading || prev.speaker !== turn.speaker);
          const live = ti === currentTurn;
          const color = turn.speaker ? speakerColor(turn.speaker) : null;
          return (
            <div key={ti} style={{ marginBottom: big ? 26 : 16, opacity: !data.synced || live || currentTurn < 0 ? 1 : 0.9, transition: "opacity 0.4s ease" }}>
              {showSpeaker && color && (
                <div style={{ display: "flex", alignItems: "center", gap: 7, marginBottom: big ? 8 : 5 }}>
                  <span style={{ width: 7, height: 7, borderRadius: 99, background: color, boxShadow: `0 0 10px ${color}` }} />
                  <span style={{ fontSize: big ? 13 : 11.5, fontWeight: 650, color: `rgba(${ink}, 0.62)` }}>{turn.speaker}</span>
                </div>
              )}
              <p style={{ margin: 0, fontSize: fs, lineHeight: big ? 1.42 : 1.55, fontWeight: big ? 650 : 560, letterSpacing: "-0.012em" }}>
                {turn.lines.map(({ line, index }) => {
                  const state = !data.synced ? "static" : index === current ? "now" : index < current ? "past" : "next";
                  return (
                    <span
                      key={index}
                      data-line={index}
                      role={data.synced ? "button" : undefined}
                      tabIndex={data.synced ? 0 : undefined}
                      onClick={data.synced ? () => { setUserScrolledAt(0); transportSeek(line.start_ms); } : undefined}
                      onKeyDown={data.synced ? (e) => { if (e.key === "Enter") transportSeek(line.start_ms); } : undefined}
                      className="tr-sentence"
                      style={{
                        cursor: data.synced ? "pointer" : "text",
                        borderRadius: 6,
                        color:
                          state === "now" || state === "static"
                            ? `rgb(${ink})`
                            : state === "past"
                              ? `rgba(${ink}, 0.36)`
                              : `rgba(${ink}, 0.58)`,
                        textShadow: state === "now" ? `0 0 22px rgba(${ink}, 0.28)` : "none",
                        transition: "color 0.35s ease, text-shadow 0.35s ease",
                        userSelect: data.synced ? "none" : "text",
                      }}
                    >
                      {line.text}{" "}
                    </span>
                  );
                })}
              </p>
            </div>
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
