import { memo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";
import { episodeLength, episodeProgress } from "../../utils/episode";
import { PRESS, PRESS_TRANSITION } from "../../lib/motion";
import type { EpisodeItem } from "../../types/podcast";

function releaseLabel(date: string | null): string | null {
  if (!date) return null;
  const d = new Date(date);
  if (Number.isNaN(d.getTime())) return date;
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: sameYear ? undefined : "numeric" });
}

/* one episode in a show / search list: cover, date + length, title, a two
line blurb, and how far in you are. the whole row plays; the round button is
the visible affordance for it. */
export const EpisodeRow = memo(function EpisodeRow({
  episode,
  active,
  playing,
  onPlay,
  showShow,
}: {
  episode: EpisodeItem;
  active: boolean;
  playing: boolean;
  onPlay: () => void;
  // search results mix shows, so name the show there
  showShow?: boolean;
}) {
  const progress = episodeProgress(episode);
  const meta = [releaseLabel(episode.release_date), progress?.label ?? episodeLength(episode.duration_ms)]
    .filter(Boolean)
    .join(" · ");

  return (
    <div
      className="q-row"
      data-clickable
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
      style={{
        display: "flex",
        gap: 14,
        alignItems: "flex-start",
        padding: "12px 10px",
        borderRadius: 10,
        opacity: episode.is_playable ? 1 : 0.45,
        borderBottom: "1px solid var(--color-divider)",
      }}
    >
      <div style={{ width: 72, height: 72, borderRadius: 8, overflow: "hidden", flexShrink: 0 }}>
        <CoverArt url={episode.image_url} alt="" size={72} style={{ width: "100%", height: "100%" }} />
      </div>
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 3 }}>
        {showShow && episode.show_name && (
          episode.show_id ? (
            <Link
              to={`/show/${episode.show_id}`}
              onClick={(e) => e.stopPropagation()}
              className="t-caption"
              style={{ fontSize: 12, color: "var(--color-text-dim)", textDecoration: "none", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}
            >
              {episode.show_name}
            </Link>
          ) : (
            <span className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>{episode.show_name}</span>
          )
        )}
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: active ? "var(--color-accent)" : "var(--color-text-hi)",
            display: "-webkit-box",
            WebkitLineClamp: 2,
            WebkitBoxOrient: "vertical",
            overflow: "hidden",
          }}
        >
          {episode.explicit && (
            <span aria-label="Explicit" style={{ fontSize: 9, fontWeight: 800, padding: "1px 4px", borderRadius: 3, background: "rgba(255,255,255,0.18)", marginRight: 6, verticalAlign: 2 }}>E</span>
          )}
          {episode.name}
        </span>
        {episode.description && (
          <span
            className="t-caption"
            style={{
              fontSize: 12.5,
              lineHeight: 1.45,
              color: "var(--color-text-dim)",
              display: "-webkit-box",
              WebkitLineClamp: 2,
              WebkitBoxOrient: "vertical",
              overflow: "hidden",
            }}
          >
            {episode.description}
          </span>
        )}
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginTop: 4 }}>
          <span className="t-caption tnum" style={{ fontSize: 12, color: "var(--color-text-dim)", whiteSpace: "nowrap" }}>{meta}</span>
          {progress && progress.fraction < 1 && (
            <div aria-hidden style={{ width: 64, height: 3, borderRadius: 2, background: "rgba(255,255,255,0.15)", overflow: "hidden" }}>
              <div style={{ width: `${progress.fraction * 100}%`, height: "100%", background: "var(--color-accent)" }} />
            </div>
          )}
        </div>
      </div>
      <motion.button
        type="button"
        aria-label={playing ? "Pause" : "Play"}
        className="focus-ring"
        onClick={(e) => {
          e.stopPropagation();
          onPlay();
        }}
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        style={{
          width: 34,
          height: 34,
          borderRadius: "50%",
          border: "none",
          flexShrink: 0,
          alignSelf: "center",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          background: active ? "var(--color-accent)" : "rgba(255,255,255,0.1)",
          color: "#fff",
        }}
      >
        <AnimatedPlayPause isPlaying={playing} size={14} />
      </motion.button>
    </div>
  );
});
