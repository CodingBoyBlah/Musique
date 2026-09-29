import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { getGridItemTransition } from "../../lib/motion";
import { useReflowPulse } from "../../hooks/useReflowPulse";
import { TILE_PAD } from "../../lib/layout";

const MotionLink = motion.create(Link);

/* a linked artwork tile with a title and a caption, dressed exactly like
AlbumCard so a shelf of playlists / shows / profiles / mixes sits flush with a
shelf of albums: same padding, radius, artwork shadow and type. `round` for
people (profiles, followers). */
export const MediaTile = memo(function MediaTile({
  to,
  imageUrl,
  title,
  subtitle,
  index = 0,
  round,
  onContextMenu,
}: {
  to: string;
  imageUrl: string | null | undefined;
  title: string;
  subtitle?: string | null;
  index?: number;
  round?: boolean;
  onContextMenu?: (e: React.MouseEvent) => void;
}) {
  useReflowPulse();
  const [hover, setHover] = useState(false);
  return (
    <MotionLink
      to={to}
      layout="position"
      className="card-link"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setHover(true)}
      onBlur={() => setHover(false)}
      onContextMenu={onContextMenu}
      whileHover={{ y: -3 }}
      whileTap={{ scale: 0.98 }}
      transition={{ type: "spring", stiffness: 480, damping: 36, ...getGridItemTransition(index) }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 8,
        padding: TILE_PAD,
        borderRadius: 12,
        width: "100%",
        boxSizing: "border-box",
        textDecoration: "none",
        color: "inherit",
        background: hover ? "var(--color-surface-hover)" : "transparent",
        transition: "background 0.18s ease",
        minWidth: 0,
        overflow: "hidden",
        textAlign: round ? "center" : undefined,
      }}
    >
      <div
        style={{
          width: "100%",
          aspectRatio: "1 / 1",
          borderRadius: round ? "50%" : 8,
          overflow: "hidden",
          flexShrink: 0,
          boxShadow: hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)",
          transition: "box-shadow 0.25s ease",
        }}
      >
        <CoverArt url={imageUrl} alt={title} size={160} rounded={round} style={{ width: "100%", height: "100%" }} />
      </div>
      <span
        style={{
          display: "block",
          fontSize: 13.5,
          fontWeight: 600,
          letterSpacing: "-0.012em",
          lineHeight: "17px",
          height: 17,
          color: "var(--color-text-hi)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          minWidth: 0,
        }}
      >
        {title}
      </span>
      {subtitle && (
        <span
          className="t-caption"
          style={{
            display: "block",
            fontSize: 12,
            lineHeight: "15px",
            height: 15,
            color: "var(--color-text-dim)",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
            minWidth: 0,
          }}
        >
          {subtitle}
        </span>
      )}
    </MotionLink>
  );
});
