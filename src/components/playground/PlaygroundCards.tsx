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

export interface GenreBucketAlbum {
  id: string;
  title: string;
  artist: string;
  coverUrl: string;
}

export const HIPHOP_ALBUMS: GenreBucketAlbum[] = [
  {
    id: "hh-1",
    title: "DAMN.",
    artist: "Kendrick Lamar",
    coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
  },
  {
    id: "hh-2",
    title: "Heroes & Villains",
    artist: "Metro Boomin",
    coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
  },
  {
    id: "hh-3",
    title: "Illmatic",
    artist: "Nas",
    coverUrl: "https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=400&auto=format&fit=crop&q=80",
  },
  {
    id: "hh-4",
    title: "Rodeo",
    artist: "Travis Scott",
    coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
  },
];

export const ELECTRONIC_ALBUMS: GenreBucketAlbum[] = [
  {
    id: "el-1",
    title: "Discovery",
    artist: "Daft Punk",
    coverUrl: "https://images.unsplash.com/photo-1614613535308-eb5fbd3d2c17?w=400&auto=format&fit=crop&q=80",
  },
  {
    id: "el-2",
    title: "Cross",
    artist: "Justice",
    coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
  },
  {
    id: "el-3",
    title: "Immunity",
    artist: "Jon Hopkins",
    coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
  },
];

// 2D Genre Bucket / Basket holding square album cover cards (matches size of 1 normal album card)
export function GenreBucketCard({
  genre = "Hip-Hop",
  albums = HIPHOP_ALBUMS,
  onPlay,
}: {
  genre?: string;
  albums?: GenreBucketAlbum[];
  onPlay?: () => void;
}) {
  const [hover, setHover] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [deckOffset, setDeckOffset] = useState(0);

  // Rotate which card is at the front of the crate on click
  function cycleCrate(e: React.MouseEvent) {
    e.stopPropagation();
    setDeckOffset((prev) => (prev + 1) % albums.length);
  }

  // Pick 3 cards to render in the bucket slot
  const visibleCards = [
    albums[(deckOffset + 0) % albums.length],
    albums[(deckOffset + 1) % albums.length],
    albums[(deckOffset + 2) % albums.length],
  ];

  return (
    <motion.div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onClick={cycleCrate}
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
        userSelect: "none",
      }}
    >
      {/* 1:1 Album Square Container matching sketch with Musique 14px rounding */}
      <div
        style={{
          position: "relative",
          width: "100%",
          aspectRatio: "1 / 1",
          borderRadius: 14,
          overflow: "hidden",
          background: hover
            ? "rgba(255, 255, 255, 0.08)"
            : "var(--color-surface, rgba(255, 255, 255, 0.035))",
          backdropFilter: "blur(20px)",
          WebkitBackdropFilter: "blur(20px)",
          border: "1px solid var(--color-glass-border, rgba(255, 255, 255, 0.08))",
        }}
      >
        {/* 2D Square Album Cover Cards slotted inside the pocket */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            paddingBottom: 26,
          }}
        >
          {visibleCards.map((album, idx) => {
            // Natural tilted fan layout matching user sketch
            const restingTransforms = [
              { x: -24, y: 10, rotate: -24, scale: 0.92, zIndex: 1 },
              { x: -2, y: -6, rotate: -2, scale: 0.98, zIndex: 2 },
              { x: 22, y: 6, rotate: 13, scale: 1.0, zIndex: 3 },
            ][idx];

            const hoverTransforms = [
              { x: -32, y: -16, rotate: -28, scale: 0.95, zIndex: 1 },
              { x: 0, y: -26, rotate: 0, scale: 1.05, zIndex: 2 },
              { x: 30, y: -14, rotate: 18, scale: 1.02, zIndex: 3 },
            ][idx];

            const currentTransform = hover ? hoverTransforms : restingTransforms;

            return (
              <motion.div
                key={`${album.id}-${idx}`}
                animate={{
                  x: currentTransform.x,
                  y: currentTransform.y,
                  rotate: currentTransform.rotate,
                  scale: currentTransform.scale,
                }}
                transition={{ type: "spring", stiffness: 380, damping: 26 }}
                style={{
                  position: "absolute",
                  width: 102,
                  height: 102,
                  aspectRatio: "1 / 1",
                  borderRadius: 6,
                  overflow: "hidden",
                  zIndex: currentTransform.zIndex,
                  background: "var(--color-surface, rgba(255, 255, 255, 0.035))",
                  border: "1px solid rgba(255, 255, 255, 0.12)",
                  boxShadow: "0 6px 18px rgba(0, 0, 0, 0.4)",
                }}
              >
                <img
                  src={album.coverUrl}
                  alt={album.title}
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "cover",
                    aspectRatio: "1 / 1",
                    display: "block",
                  }}
                />
              </motion.div>
            );
          })}
        </div>

        {/* Acrylic Pocket with ONLY the Genre Name */}
        <div
          style={{
            position: "absolute",
            bottom: 0,
            left: 0,
            right: 0,
            height: 52,
            background: "rgba(18, 22, 32, 0.72)",
            backdropFilter: "blur(24px) saturate(180%)",
            WebkitBackdropFilter: "blur(24px) saturate(180%)",
            borderTop: "1px solid rgba(255, 255, 255, 0.16)",
            borderTopLeftRadius: 10,
            borderTopRightRadius: 10,
            borderBottomLeftRadius: 14,
            borderBottomRightRadius: 14,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "0 14px",
            zIndex: 10,
            boxShadow: "0 -4px 16px rgba(0, 0, 0, 0.2)",
          }}
        >
          <span
            style={{
              fontSize: 14,
              fontWeight: 700,
              letterSpacing: "-0.01em",
              color: "var(--color-text-hi, rgba(255, 255, 255, 0.97))",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {genre}
          </span>

          <CirclePlayButton
            isPlaying={playing}
            visible={hover || playing}
            size={26}
            iconSize={11}
            onClick={(e) => {
              e.stopPropagation();
              setPlaying(!playing);
              onPlay?.();
            }}
          />
        </div>
      </div>
    </motion.div>
  );
}
