import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Check, Mic, Plus, Link2, Play } from "@/lib/icons";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";
import { useContextMenu, type MenuEntry } from "./ContextMenu";
import { Tooltip } from "./Tooltip";
import { coverUrl } from "../../lib/coverUrl";
import { fmtMs } from "../../utils/fmt";
import { episodeProgress, episodeToTrack } from "../../utils/episode";
import { useQueueStore } from "../../store/queue.store";
import { shareSpotifyLink } from "../../lib/share";
import { toast } from "../../store/toast.store";
import { PRESS, PRESS_TRANSITION } from "../../lib/motion";
import type { EpisodeItem } from "../../types/podcast";

export function releaseLabel(date: string | null): string | null {
  if (!date) return null;
  const d = new Date(`${date}T12:00:00`);
  if (Number.isNaN(d.getTime())) return date;
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: sameYear ? undefined : "numeric" });
}

/* one episode, drawn exactly like a TrackRow: number that turns into play on
hover, cover, title over a caption line, then the right-hand meta. the caption
carries the date (and the show, in mixed lists) and the start of the blurb;
the right side shows how far in you are before the length. */
export const EpisodeRow = memo(function EpisodeRow({
  episode,
  index,
  active,
  playing,
  onPlay,
  showShow,
}: {
  episode: EpisodeItem;
  index?: number;
  active: boolean;
  playing: boolean;
  onPlay: () => void;
  // search results mix shows, so name the show there
  showShow?: boolean;
}) {
  const [hover, setHover] = useState(false);
  const enqueue = useQueueStore((s) => s.enqueue);
  const { open: openMenu, element: menuEl } = useContextMenu();
  const progress = episodeProgress(episode);
  const date = releaseLabel(episode.release_date);
  const baseBg = index != null && index % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent";

  const menu: MenuEntry[] = [
    { label: playing ? "Pause" : "Play", icon: <Play size={14} />, onSelect: onPlay },
    {
      label: "Add to queue",
      icon: <Plus size={14} />,
      onSelect: () => {
        enqueue(episodeToTrack(episode));
        toast(`Added "${episode.name}" to queue`);
      },
    },
    { label: "Copy Spotify link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("episode", episode.id) },
  ];

  return (
    <>
      <div
        role="button"
        tabIndex={0}
        aria-label={`Play ${episode.name}`}
        onClick={onPlay}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onPlay();
          }
        }}
        onMouseEnter={() => setHover(true)}
        onMouseLeave={() => setHover(false)}
        onContextMenu={openMenu(menu)}
        className="group track-row"
        style={{
          position: "relative",
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "8px 12px",
          margin: "1.5px 0",
          borderRadius: 8,
          transition: "background 0.12s ease",
          background: hover ? "var(--color-surface-hover, rgba(255,255,255,0.06))" : baseBg,
          cursor: "pointer",
          userSelect: "none",
          opacity: episode.is_playable ? 1 : 0.5,
        }}
      >
        <div style={{ width: 28, height: 28, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          {hover || playing ? (
            <motion.button
              whileTap={PRESS}
              transition={PRESS_TRANSITION}
              className="focus-ring"
              onClick={(e) => {
                e.stopPropagation();
                onPlay();
              }}
              aria-label={playing ? `Pause ${episode.name}` : `Play ${episode.name}`}
              style={{ display: "flex", alignItems: "center", justifyContent: "center", width: 28, height: 28, borderRadius: "50%", border: "none", background: "transparent", padding: 0, cursor: "pointer" }}
            >
              <AnimatedPlayPause isPlaying={playing} size={15} strokeWidth={2.4} fill={playing ? "var(--color-accent)" : "#ffffff"} />
            </motion.button>
          ) : index != null ? (
            <span className="tnum" style={{ fontSize: 13, color: active ? "var(--color-accent)" : "var(--color-text-muted)" }}>{index + 1}</span>
          ) : (
            <Mic size={14} style={{ color: "var(--color-text-dim)" }} />
          )}
        </div>

        {episode.image_url ? (
          <img src={coverUrl(episode.image_url, 38) ?? episode.image_url} alt="" loading="lazy" decoding="async" style={{ width: 38, height: 38, borderRadius: 6, objectFit: "cover", flexShrink: 0, boxShadow: "0 2px 6px rgba(0,0,0,0.25)" }} />
        ) : (
          <div style={{ width: 38, height: 38, borderRadius: 6, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
            <Mic size={16} style={{ color: "var(--color-text-dim)" }} />
          </div>
        )}

        <div style={{ flex: 1, minWidth: 0 }}>
          <p style={{ margin: 0, fontSize: 13.5, fontWeight: 600, letterSpacing: "-0.012em", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: active ? "var(--color-accent)" : "rgba(255,255,255,0.92)" }}>
            {episode.name}
          </p>
          <p className="t-caption" style={{ margin: "2px 0 0", fontSize: 12, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "var(--color-text-dim)" }}>
            {episode.explicit && (
              <span style={{ display: "inline-block", marginRight: 5, padding: "1px 4px", borderRadius: 3, fontSize: 9.5, fontWeight: 700, background: "rgba(255,255,255,0.12)", color: "rgba(255,255,255,0.70)" }}>E</span>
            )}
            {showShow && episode.show_name && (
              <>
                {episode.show_id ? (
                  <Link to={`/show/${episode.show_id}`} onClick={(e) => e.stopPropagation()} className="link-inline">{episode.show_name}</Link>
                ) : episode.show_name}
                {" · "}
              </>
            )}
            {date}
            {episode.description && <span style={{ color: "var(--color-text-muted)" }}>{date ? " · " : ""}{episode.description}</span>}
          </p>
        </div>

        {progress && (
          progress.fraction >= 1 ? (
            <Tooltip label="Played" side="top">
              <span style={{ display: "flex", alignItems: "center", color: "var(--color-accent)", flexShrink: 0 }} aria-label="Played">
                <Check size={15} strokeWidth={2.6} />
              </span>
            </Tooltip>
          ) : (
            <Tooltip label={progress.label} side="top">
              <span aria-label={progress.label} style={{ width: 44, height: 4, borderRadius: 4, background: "rgba(255,255,255,0.12)", overflow: "hidden", flexShrink: 0 }}>
                <span style={{ display: "block", width: `${progress.fraction * 100}%`, height: "100%", background: "var(--color-accent)", borderRadius: 4 }} />
              </span>
            </Tooltip>
          )
        )}

        <span className="tnum t-caption" style={{ fontSize: 12.5, flexShrink: 0, minWidth: 40, textAlign: "right", color: "rgba(255,255,255,0.35)" }}>
          {fmtMs(episode.duration_ms)}
        </span>
      </div>
      {menuEl}
    </>
  );
});
