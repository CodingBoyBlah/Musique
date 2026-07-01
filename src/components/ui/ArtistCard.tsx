import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQueryClient } from "@tanstack/react-query";
import { CoverArt } from "./CoverArt";
import { prefetchArtist } from "../../lib/prefetch";
import { useReflowPulse } from "../../hooks/useReflowPulse";

interface Props {
  artist: { id: string; name: string; image_url?: string | null };
  size?:  number;
}

// see AlbumCard --animate position on grid reflow, never size   BROKEN ON BUILD, TODO   DONE
const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };
const MotionLink = motion.create(Link);

// responsive grid of circular artist cards, even gutters, fills the row
export function ArtistGrid({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(clamp(108px, 14vw, 150px), 1fr))",
        gap: "clamp(8px, 1.2vw, 14px)",
        width: "100%",
      }}
    >
      {children}
    </div>
  );
}

function ArtistCardImpl({ artist }: Props) {
  const [hover, setHover] = useState(false);
  const qc = useQueryClient();
  useReflowPulse(); // rerender on resize / panel toggle so layout glides the move UDPATE macos FIXED

  return (
    <MotionLink
      to={`/artist/${artist.id}`}
      layout="position"
      onMouseEnter={() => { setHover(true); prefetchArtist(qc, artist.id); }}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ layout: REFLOW }}
      style={{
        display: "flex", flexDirection: "column", alignItems: "center", gap: 10,
        padding: 14, borderRadius: 14, width: "100%", boxSizing: "border-box",
        textDecoration: "none", color: "inherit",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
      }}
    >
      {/* circular avatar fills the padded width, soft ring + lift shadow */}
      <div
        style={{
          width: "100%", aspectRatio: "1 / 1", borderRadius: "50%", overflow: "hidden",
          boxShadow: hover ? "0 10px 28px rgba(0,0,0,0.45)" : "0 4px 14px rgba(0,0,0,0.30)",
          transition: "box-shadow 0.18s ease",
        }}
      >
        <CoverArt url={artist.image_url ?? null} alt={artist.name} size={140} rounded style={{ width: "100%", height: "100%" }} />
      </div>
      <span className="text-sm font-semibold text-center line-clamp-2" style={{ maxWidth: "100%" }}>
        {artist.name}
      </span>
      <span className="text-xs" style={{ color: "var(--color-text-dim)" }}>Artist</span>
    </MotionLink>
  );
}

export const ArtistCard = memo(ArtistCardImpl);
