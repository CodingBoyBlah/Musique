import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQueryClient } from "@tanstack/react-query";
import type { AlbumItem } from "../../types/spotify";
import { CoverArt } from "./CoverArt";
import { ReleaseCountdown } from "./ReleaseCountdown";
import { releaseYear, isUpcoming } from "../../utils/fmt";
import { prefetchAlbum } from "../../lib/prefetch";
import { useReflowPulse } from "../../hooks/useReflowPulse";

/* reflow spring. when a grid recolumns (lyrics panel opens, window resizes)
 items glide to their new cell instead of snapping. layout="position" o ly
 animates position not size, so covers/text never squish mid-move. */

/* TODO - fix above task, does not work, teleports snaps */
const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };

const MotionLink = motion.create(Link);

interface Props {
  album: AlbumItem;
  size?: number;
}


export function AlbumGrid({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(clamp(118px, 15vw, 160px), 1fr))",
        gap: "clamp(10px, 1.5vw, 16px)",
        width: "100%",
      }}
    >
      {children}
    </div>
  );
}

function AlbumCardImpl({ album, size = 160 }: Props) {
  const imgSize = size - 24;
  const [hover, setHover] = useState(false);
  const qc = useQueryClient();
  useReflowPulse(); // rerender on resize / panel toggle so layout glides the move {Broken}

  return (
    <MotionLink
      to={`/album/${album.id}`}
      layout="position"
      // warm the album page the moment the pointer lands so opening it is instant
      onMouseEnter={() => { setHover(true); prefetchAlbum(qc, album.id); }}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ layout: REFLOW }}
      style={{
        /* 14px inset all round keeps the hover wash off the cover and text so
         they never touch the lit edge. cover fills the padded width so the
         inset is equal everywhere (no unsemetrical gap). */
        display: "flex", flexDirection: "column", gap: 8,
        padding: 14, borderRadius: 14, width: "100%", boxSizing: "border-box",
        textDecoration: "none", color: "inherit",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
      }}
    >
      <div style={{ width: "100%", aspectRatio: "1 / 1" }}>
        <CoverArt url={album.image_url} alt={album.name} size={imgSize} style={{ width: "100%", height: "100%" }} />
      </div>
      <span className="text-sm font-medium line-clamp-2" style={{ maxWidth: "100%" }}>
        {album.name}
      </span>
      {isUpcoming(album.release_date) ? (
        // not out yet -> live countdown instead of a static year {DOES NOT WORK}
        <ReleaseCountdown date={album.release_date!} />
      ) : (
        <span className="text-xs" style={{ color: "var(--color-text-dim)" }}>
          {releaseYear(album.release_date)}
          {album.release_date && album.album_type ? " · " : ""}
          <span className="capitalize">{album.album_type}</span>
        </span>
      )}
    </MotionLink>
  );
}

// memoised so grids don't re-render every card when some unrelated parent state (hover, -- a sibling) changes
export const AlbumCard = memo(AlbumCardImpl);
