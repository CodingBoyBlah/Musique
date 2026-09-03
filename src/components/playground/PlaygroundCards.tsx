import { useState } from "react";
import { motion } from "framer-motion";
import { CirclePlayButton } from "../ui/CirclePlayButton";

// Random Access Memories Album Card (Kept as user explicitly requested!)
export function MusiqueAlbumCard({
  title = "Random Access Memories",
  artist = "Daft Punk",
  year = "2013",
  coverUrl = "https://images.unsplash.com/photo-1614613535308-eb5fbd3d2c17?w=400&auto=format&fit=crop&q=80",
}: {
  title?: string;
  artist?: string;
  year?: string;
  coverUrl?: string;
}) {
  const [hover, setHover] = useState(false);
  const [playing, setPlaying] = useState(false);

  return (
    <motion.div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ type: "spring", stiffness: 520, damping: 44 }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 8,
        padding: 14,
        borderRadius: 14,
        width: 190,
        boxSizing: "border-box",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
        cursor: "pointer",
        position: "relative",
      }}
    >
      <div style={{ position: "relative", width: "100%", aspectRatio: "1 / 1", borderRadius: 8, overflow: "hidden" }}>
        <img
          src={coverUrl}
          alt={title}
          style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
        />

        {/* Hover play button with blur+scale icon morph */}
        <CirclePlayButton
          isPlaying={playing}
          visible={hover || playing}
          onClick={(e) => {
            e.stopPropagation();
            setPlaying(!playing);
          }}
          style={{ position: "absolute", right: 10, bottom: 10 }}
        />
      </div>

      <span
        style={{
          fontSize: 14,
          fontWeight: 500,
          color: "var(--color-text-hi)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {title}
      </span>
      <span style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
        {year} • {artist}
      </span>
    </motion.div>
  );
}
