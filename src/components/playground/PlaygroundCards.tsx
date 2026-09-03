import { useState } from "react";
import { motion } from "framer-motion";
import { AnimatedPlayPause } from "./AnimatedIcons";

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
        <motion.button
          initial={{ opacity: 0, scale: 0.8 }}
          animate={hover || playing ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.8 }}
          whileHover={{ scale: 1.08 }}
          whileTap={{ scale: 0.92 }}
          onClick={(e) => {
            e.stopPropagation();
            setPlaying(!playing);
          }}
          style={{
            position: "absolute",
            right: 10,
            bottom: 10,
            width: 42,
            height: 42,
            borderRadius: 99,
            border: "none",
            background: "var(--color-accent)",
            color: "var(--color-accent-text)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer",
            boxShadow: "0 4px 14px rgba(0,0,0,0.5)",
          }}
        >
          <AnimatedPlayPause isPlaying={playing} size={17} strokeWidth={0} />
        </motion.button>
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
