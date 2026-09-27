import { useState, useMemo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Music } from "@/lib/icons";
import { SPRING } from "@/lib/motion";
import "../../styles/ui.css";
import { useQuery } from "@tanstack/react-query";
import { search } from "../../api/spotify";
import { useTopTracks, useSavedAlbums } from "../../hooks/useLibrary";
import { CirclePlayButton } from "./CirclePlayButton";

export interface CrateAlbum {
  id: string;
  title: string;
  artist?: string;
  coverUrl: string;
}

/* No fallback albums. This used to fill a thin crate with stock photos
   captioned as real records ("1989 - Taylor Swift" over an Unsplash picture),
   which is a small lie on every render. A crate with fewer real covers shows
   plain placeholder sleeves instead - honest, and still the right shape. */

const MotionLink = motion.create(Link);

export function GenreCrate({
  genre,
  albums: customAlbums,
  to,
  onPlay,
  isPlaying = false,
  style,
}: {
  genre: string;
  albums?: CrateAlbum[];
  to?: string;
  onPlay?: () => void;
  // whether this crate's music is what's playing now. owned by the caller,
  // which knows what onPlay started - the crate never guesses.
  isPlaying?: boolean;
  style?: React.CSSProperties;
}) {
  const [hover, setHover] = useState(false);

  // 1. Fetch real Spotify albums for this genre if customAlbums not provided
  const { data: searchResults } = useQuery({
    queryKey: ["genre-crate-albums", genre],
    queryFn: () => search(genre, "album"),
    enabled: !customAlbums,
    staleTime: 1000 * 60 * 30, // 30 minutes
  });

  // 2. Fetch user's top tracks & saved albums to prioritize albums they like
  const { data: topTracks = [] } = useTopTracks("medium_term");
  const { data: savedAlbums = [] } = useSavedAlbums();

  // 3. Assemble personalized & genre-specific albums
  const albums = useMemo(() => {
    if (customAlbums && customAlbums.length > 0) return customAlbums;

    const collected: CrateAlbum[] = [];
    const seenIds = new Set<string>();

    const genreLower = genre.toLowerCase();

    // Check user's saved albums matching artist or title
    for (const sa of savedAlbums) {
      if (sa.image_url && !seenIds.has(sa.id)) {
        const matchesArtist = sa.artists.some((a) =>
          a.name.toLowerCase().includes(genreLower)
        );
        if (matchesArtist) {
          seenIds.add(sa.id);
          collected.push({
            id: sa.id,
            title: sa.name,
            artist: sa.artists.map((a) => a.name).join(", "),
            coverUrl: sa.image_url,
          });
          if (collected.length >= 2) break;
        }
      }
    }

    // Check user's top tracks matching genre
    for (const tt of topTracks) {
      if (tt.album?.image_url && !seenIds.has(tt.album.id)) {
        const matchesArtist = tt.artists.some((a) =>
          a.name.toLowerCase().includes(genreLower)
        );
        if (matchesArtist) {
          seenIds.add(tt.album.id);
          collected.push({
            id: tt.album.id,
            title: tt.album.name,
            artist: tt.artists.map((a) => a.name).join(", "),
            coverUrl: tt.album.image_url,
          });
          if (collected.length >= 3) break;
        }
      }
    }

    // Add albums returned by Spotify search for this genre
    if (searchResults?.albums) {
      for (const alb of searchResults.albums) {
        if (alb.image_url && !seenIds.has(alb.id)) {
          seenIds.add(alb.id);
          collected.push({
            id: alb.id,
            title: alb.name,
            artist: alb.artists.map((a) => a.name).join(", "),
            coverUrl: alb.image_url,
          });
          if (collected.length >= 6) break;
        }
      }
    }

    return collected;
  }, [customAlbums, genre, searchResults, savedAlbums, topTracks]);

  // Pick 3 sleeves for the crate slot; missing ones are blank placeholders
  const visibleCards: (CrateAlbum | null)[] = [albums[0] ?? null, albums[1] ?? null, albums[2] ?? null];

  const destination = to || `/search?q=${encodeURIComponent(genre)}`;

  return (
    <MotionLink
      to={destination}
      className="card-link"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setHover(true)}
      onBlur={() => setHover(false)}
      whileHover={{ y: -4 }}
      whileTap={{ scale: 0.98 }}
      transition={SPRING}
      style={{
        position: "relative",
        display: "block",
        width: "100%",
        aspectRatio: "1 / 1",
        borderRadius: 14,
        overflow: "hidden",
        /* solid tint, no backdrop-filter: this card lifts on hover, and a blur
           under a moving layer is recomputed on every frame of the spring */
        background: hover
          ? "rgba(255, 255, 255, 0.08)"
          : "var(--color-surface, rgba(255, 255, 255, 0.035))",
        border: "1px solid var(--color-glass-border, rgba(255, 255, 255, 0.08))",
        textDecoration: "none",
        userSelect: "none",
        boxShadow: hover
          ? "0 14px 32px rgba(0, 0, 0, 0.45)"
          : "0 4px 14px rgba(0, 0, 0, 0.2)",
        transition: "background-color 0.22s var(--ease-out), box-shadow 0.22s var(--ease-out)",
        cursor: "pointer",
        ...style,
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
          // Angles and offsets matching the user sketch
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
              key={album ? `${album.id}-${idx}` : `empty-${idx}`}
              animate={{
                x: currentTransform.x,
                y: currentTransform.y,
                rotate: currentTransform.rotate,
                scale: currentTransform.scale,
              }}
              // no overshoot: the fan is a hover response, not a throw
              transition={SPRING}
              style={{
                position: "absolute",
                width: "56%",
                aspectRatio: "1 / 1",
                borderRadius: 6,
                overflow: "hidden",
                zIndex: currentTransform.zIndex,
                background: "var(--color-surface, rgba(255, 255, 255, 0.035))",
                border: "1px solid rgba(255, 255, 255, 0.12)",
                boxShadow: "0 6px 18px rgba(0, 0, 0, 0.4)",
              }}
            >
              {album ? (
                <img
                  src={album.coverUrl}
                  alt={album.title}
                  loading="lazy"
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "cover",
                    aspectRatio: "1 / 1",
                    display: "block",
                  }}
                />
              ) : (
                <div
                  aria-hidden
                  style={{
                    width: "100%",
                    height: "100%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    background: "linear-gradient(135deg, color-mix(in srgb, var(--color-accent) 20%, transparent) 0%, rgba(255, 255, 255, 0.04) 100%)",
                    color: "rgba(255, 255, 255, 0.35)",
                  }}
                >
                  <Music size={22} />
                </div>
              )}
            </motion.div>
          );
        })}
      </div>

      {/* Acrylic Pocket with ONLY the Genre Name */}
      <div
        className="glass-solid-fallback"
        style={{
          position: "absolute",
          bottom: 0,
          left: 0,
          right: 0,
          height: "34%",
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
            fontSize: "1rem",
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

        {onPlay && (
          <CirclePlayButton
            isPlaying={isPlaying}
            visible={hover || isPlaying}
            size={28}
            iconSize={12}
            ariaLabel={isPlaying ? `Pause ${genre}` : `Play ${genre}`}
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              onPlay();
            }}
          />
        )}
      </div>
    </MotionLink>
  );
}