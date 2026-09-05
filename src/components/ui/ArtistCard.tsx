import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowUpRight, User } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { CoverArt } from "./CoverArt";
import { prefetchArtist } from "../../lib/prefetch";
import { gpuLayer, zTransform } from "../../lib/motion";

interface Props {
  artist: { id: string; name: string; image_url?: string | null };
  size?:  number;
}

const MotionLink = motion.create(Link);

// responsive grid of editorial artist cards, even gutters, fills the row
export function ArtistGrid({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(clamp(116px, 14vw, 160px), 1fr))",
        gap: "clamp(10px, 1.4vw, 16px)",
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

  return (
    <MotionLink
      to={`/artist/${artist.id}`}
      transformTemplate={zTransform}
      onMouseEnter={() => { setHover(true); prefetchArtist(qc, artist.id); }}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ type: "spring", stiffness: 520, damping: 44 }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 10,
        padding: 12,
        borderRadius: 16,
        width: "100%",
        boxSizing: "border-box",
        textDecoration: "none",
        color: "inherit",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
        position: "relative",
        cursor: "pointer",
        minWidth: 0,
        ...gpuLayer,
      }}
    >
      {/* Editorial squircle frame with ambient depth and micro-badge */}
      <div
        style={{
          position: "relative",
          width: "100%",
          aspectRatio: "1 / 1",
          borderRadius: 14,
          overflow: "hidden",
          boxShadow: hover
            ? "0 14px 32px -4px rgba(0, 0, 0, 0.55), 0 0 20px var(--color-accent-dim)"
            : "0 6px 18px rgba(0, 0, 0, 0.35)",
          transition: "box-shadow 0.28s ease",
        }}
      >
        {artist.image_url ? (
          <motion.div
            animate={{ scale: hover ? 1.05 : 1 }}
            transition={{ duration: 0.35, ease: [0.23, 1, 0.32, 1] }}
            style={{ width: "100%", height: "100%" }}
          >
            <CoverArt
              url={artist.image_url}
              alt={artist.name}
              size={160}
              style={{ width: "100%", height: "100%", borderRadius: "inherit", objectFit: "cover", display: "block" }}
            />
          </motion.div>
        ) : (
          <div
            style={{
              width: "100%",
              height: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              background: "linear-gradient(135deg, rgba(88, 115, 216, 0.22) 0%, rgba(255, 255, 255, 0.05) 100%)",
              color: "rgba(255, 255, 255, 0.6)",
              fontSize: 32,
              fontWeight: 700,
              letterSpacing: "-0.02em",
            }}
          >
            {artist.name ? artist.name.charAt(0).toUpperCase() : <User size={28} />}
          </div>
        )}

        {/* Subtle inner highlight border */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            borderRadius: "inherit",
            border: "1px solid rgba(255, 255, 255, 0.08)",
            pointerEvents: "none",
          }}
        />

        {/* Soft bottom vignette for photographic contrast */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "linear-gradient(to top, rgba(10, 12, 16, 0.45) 0%, transparent 40%)",
            pointerEvents: "none",
          }}
        />


        {/* Hover action cue: floating explore arrow */}
        <AnimatePresence>
          {hover && (
            <motion.div
              initial={{ opacity: 0, scale: 0.75, y: 6 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.75, y: 6 }}
              transition={{ type: "spring", stiffness: 450, damping: 26 }}
              style={{
                position: "absolute",
                right: 8,
                bottom: 8,
                width: 32,
                height: 32,
                borderRadius: "50%",
                background: "var(--color-accent)",
                color: "#ffffff",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                boxShadow: "0 6px 16px rgba(0, 0, 0, 0.45)",
                pointerEvents: "none",
              }}
            >
              <ArrowUpRight size={16} strokeWidth={2.4} />
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Typography & Subtitle */}
      <div style={{ display: "flex", flexDirection: "column", gap: 3, width: "100%", minWidth: 0 }}>
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: hover ? "var(--color-accent)" : "var(--color-text-hi)",
            transition: "color 0.16s ease",
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
            maxWidth: "100%",
          }}
        >
          {artist.name}
        </span>

        <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 12, color: "var(--color-text-dim)" }}>
          <span
            style={{
              width: 5,
              height: 5,
              borderRadius: "50%",
              background: hover ? "var(--color-accent)" : "rgba(255, 255, 255, 0.25)",
              transition: "background 0.16s ease",
              flexShrink: 0,
            }}
          />
          <span style={{ transition: "color 0.16s ease", color: hover ? "var(--color-text)" : "var(--color-text-dim)" }}>
            {hover ? "View profile" : "Artist"}
          </span>
        </div>
      </div>
    </MotionLink>
  );
}

export const ArtistCard = memo(ArtistCardImpl);
