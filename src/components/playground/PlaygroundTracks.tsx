import { useState } from "react";
import { motion } from "framer-motion";
import { Plus } from "lucide-react";
import { AnimatedPlayPause, AnimatedHeart } from "./AnimatedIcons";

const SAMPLE_TRACKS = [
  { id: "1", index: 0, name: "Give Life Back to Music", artist: "Daft Punk", duration: "4:34" },
  { id: "2", index: 1, name: "Instant Crush", artist: "Daft Punk ft. Julian Casablancas", duration: "5:37" },
  { id: "3", index: 2, name: "Giorgio by Moroder", artist: "Daft Punk", duration: "9:04" },
  { id: "4", index: 3, name: "Within", artist: "Daft Punk", duration: "3:48" },
  { id: "5", index: 4, name: "Get Lucky", artist: "Daft Punk ft. Pharrell Williams", duration: "6:09" },
  { id: "6", index: 5, name: "Lose Yourself to Dance", artist: "Daft Punk ft. Pharrell Williams", duration: "5:53" },
];

function TrackRowItem({
  track,
  index,
  isPlaying,
  onTogglePlay,
}: {
  track: typeof SAMPLE_TRACKS[0];
  index: number;
  isPlaying: boolean;
  onTogglePlay: () => void;
}) {
  const [hover, setHover] = useState(false);
  const [liked, setLiked] = useState(index === 1);

  // Alternating colors directly from Musique TrackRow.tsx:
  const baseBg = index % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent";

  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "9px 12px",
        borderRadius: 8,
        transition: "background 0.1s",
        background: hover ? "var(--color-surface-hover, rgba(255,255,255,0.06))" : baseBg,
        cursor: "pointer",
        userSelect: "none",
      }}
      onClick={onTogglePlay}
    >
      {/* Left Slot: Track Number or Play/Pause Button with blur+scale morph */}
      <div style={{ position: "relative", width: 30, height: 30, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
        {hover || isPlaying ? (
          <motion.button
            whileHover={{ scale: 1.15 }}
            whileTap={{ scale: 0.88 }}
            onClick={(e) => {
              e.stopPropagation();
              onTogglePlay();
            }}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: 28,
              height: 28,
              borderRadius: "50%",
              border: "none",
              background: "transparent",
              color: isPlaying ? "var(--color-accent)" : "#ffffff",
              cursor: "pointer",
              padding: 0,
            }}
          >
            <AnimatedPlayPause isPlaying={isPlaying} size={15} strokeWidth={2.4} fill={isPlaying ? "var(--color-accent)" : "#ffffff"} />
          </motion.button>
        ) : (
          <span style={{ fontSize: 13, color: "var(--color-text-muted, rgba(255,255,255,0.4))", fontVariantNumeric: "tabular-nums" }}>
            {index + 1}
          </span>
        )}
      </div>

      {/* Title & Artist with Live 3-Bar Equalizer */}
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <p
            style={{
              margin: 0,
              fontSize: 14,
              fontWeight: 500,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              color: isPlaying ? "var(--color-accent)" : "rgba(255,255,255,0.90)",
            }}
          >
            {track.name}
          </p>

          {/* Playing indicator: animated 3-bar equalizer */}
          {isPlaying && (
            <div style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 12, width: 12, flexShrink: 0 }}>
              {[0.4, 1.0, 0.6].map((_, i) => (
                <motion.div
                  key={i}
                  animate={{ height: ["20%", "100%", "20%"] }}
                  transition={{ repeat: Infinity, duration: 0.55 + i * 0.15, ease: "easeInOut" }}
                  style={{ flex: 1, borderRadius: 1, background: "var(--color-accent)" }}
                />
              ))}
            </div>
          )}
        </div>

        <p style={{ margin: 0, fontSize: 12.5, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "rgba(255,255,255,0.45)" }}>
          {track.artist}
        </p>
      </div>

      {/* Right Slot: Like Button & Duration */}
      <div style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
        <button
          onClick={(e) => {
            e.stopPropagation();
            setLiked(!liked);
          }}
          title={liked ? "Remove from Liked Songs" : "Save to Liked Songs"}
          style={{
            background: "transparent",
            border: "none",
            color: liked ? "var(--color-accent)" : "rgba(255,255,255,0.45)",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            padding: 4,
            opacity: hover || liked ? 1 : 0,
            transition: "opacity 0.12s",
          }}
        >
          <AnimatedHeart liked={liked} size={15} />
        </button>

        <button
          onClick={(e) => e.stopPropagation()}
          title="Add to playlist"
          style={{
            background: "transparent",
            border: "none",
            color: "rgba(255,255,255,0.45)",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            padding: 4,
            opacity: hover ? 1 : 0,
            transition: "opacity 0.12s",
          }}
        >
          <Plus size={15} />
        </button>

        <span style={{ fontSize: 12.5, color: "rgba(255,255,255,0.45)", width: 36, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>
          {track.duration}
        </span>
      </div>
    </div>
  );
}

export function AlbumTrackRowsSection() {
  const [playingId, setPlayingId] = useState<string>("2"); // "Instant Crush" playing by default

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100%" }}>
      {SAMPLE_TRACKS.map((t, i) => (
        <TrackRowItem
          key={t.id}
          track={t}
          index={i}
          isPlaying={playingId === t.id}
          onTogglePlay={() => setPlayingId(playingId === t.id ? "" : t.id)}
        />
      ))}
    </div>
  );
}
