import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getEpisodeTranscript, type Transcript } from "../../api/podcasts";
import { usePlayerStore } from "../../store/player.store";
import { transportSeek } from "../../hooks/usePlayerControls";
import { EPISODE_PREFIX } from "../../utils/episode";

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

/* spotify's read-along transcript, as subtitles. the line being spoken is big
and bright, what's been said fades back, what's coming waits a shade dimmer.
it follows the playhead like the lyrics do, backs off while you scroll, and a
click on any line jumps there. `size` scales it: the side panel is compact,
the immersive view reads from across the room. */
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

  const scroller = useRef<HTMLDivElement>(null);
  const [userScrolledAt, setUserScrolledAt] = useState(0);
  const following = Date.now() - userScrolledAt > 4000;

  useEffect(() => {
    if (!following || current < 0) return;
    const el = scroller.current?.querySelector<HTMLElement>(`[data-line="${current}"]`);
    const box = scroller.current;
    if (!el || !box) return;
    box.scrollTo({ top: el.offsetTop - box.clientHeight * 0.32, behavior: "smooth" });
  }, [current, following]);

  const big = size === "stage";
  const fs = big ? "clamp(20px, 2vw, 30px)" : "17px";

  if (isLoading) {
    return <div style={{ padding: 16, color: `rgba(${ink}, 0.5)`, fontSize: 14 }}>Loading transcript...</div>;
  }
  if (!data) {
    return <>{empty ?? <div style={{ padding: 16, color: `rgba(${ink}, 0.55)`, fontSize: 14 }}>No transcript for this episode.</div>}</>;
  }

  return (
    <div
      ref={scroller}
      className="ovs-hide"
      onWheel={() => setUserScrolledAt(Date.now())}
      onTouchMove={() => setUserScrolledAt(Date.now())}
      style={{
        position: "relative",
        height: "100%",
        overflowY: "auto",
        padding: big ? "20vh 4px 45vh" : "16px 16px 50%",
        maskImage: "linear-gradient(180deg, transparent 0, #000 12%, #000 80%, transparent 100%)",
        WebkitMaskImage: "linear-gradient(180deg, transparent 0, #000 12%, #000 80%, transparent 100%)",
      }}
    >
      {data.lines.map((line, i) => {
        const state = !data.synced ? "static" : i === current ? "now" : i < current ? "past" : "next";
        const showSpeaker = line.speaker && (i === 0 || data.lines[i - 1].speaker !== line.speaker);
        if (line.heading) {
          return (
            <div key={i} data-line={i} style={{ margin: big ? "28px 0 12px" : "18px 0 8px", fontSize: 11.5, fontWeight: 800, letterSpacing: "0.1em", textTransform: "uppercase", color: "var(--color-accent)" }}>
              {line.text}
            </div>
          );
        }
        return (
          <div key={i} data-line={i}>
            {showSpeaker && (
              <div style={{ margin: big ? "22px 0 6px" : "14px 0 4px", fontSize: 11, fontWeight: 700, letterSpacing: "0.06em", textTransform: "uppercase", color: `rgba(${ink}, 0.5)` }}>
                {line.speaker}
              </div>
            )}
            <p
              role={data.synced ? "button" : undefined}
              tabIndex={data.synced ? 0 : undefined}
              onClick={data.synced ? () => transportSeek(line.start_ms) : undefined}
              onKeyDown={data.synced ? (e) => { if (e.key === "Enter") transportSeek(line.start_ms); } : undefined}
              style={{
                margin: big ? "0 0 14px" : "0 0 10px",
                fontSize: fs,
                lineHeight: 1.35,
                fontWeight: state === "now" ? 750 : 650,
                letterSpacing: "-0.015em",
                cursor: data.synced ? "pointer" : "text",
                color:
                  state === "now" || state === "static"
                    ? `rgb(${ink})`
                    : state === "past"
                      ? `rgba(${ink}, 0.34)`
                      : `rgba(${ink}, 0.52)`,
                transform: state === "now" ? "scale(1)" : "scale(0.985)",
                transformOrigin: "left center",
                transition: "color 0.35s ease, transform 0.35s ease",
                userSelect: data.synced ? "none" : "text",
              }}
            >
              {line.text}
            </p>
          </div>
        );
      })}
      {!following && data.synced && current >= 0 && (
        <button
          type="button"
          className="btn-pill"
          onClick={() => setUserScrolledAt(0)}
          style={{ position: "sticky", bottom: 12, left: "50%", display: "block", margin: "0 auto" }}
        >
          Back to now
        </button>
      )}
    </div>
  );
}
