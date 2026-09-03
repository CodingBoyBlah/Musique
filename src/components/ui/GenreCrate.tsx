import { useState, useMemo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
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

// Fallback curated albums for all genres (ensures zero empty states and immediate rendering)
const FALLBACK_GENRE_ALBUMS: Record<string, CrateAlbum[]> = {
  Pop: [
    {
      id: "pop-1",
      title: "1989",
      artist: "Taylor Swift",
      coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "pop-2",
      title: "Future Nostalgia",
      artist: "Dua Lipa",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "pop-3",
      title: "SOUR",
      artist: "Olivia Rodrigo",
      coverUrl: "https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=400&auto=format&fit=crop&q=80",
    },
  ],
  "Hip-Hop": [
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
  ],
  Rock: [
    {
      id: "rk-1",
      title: "AM",
      artist: "Arctic Monkeys",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "rk-2",
      title: "Rumours",
      artist: "Fleetwood Mac",
      coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "rk-3",
      title: "Nevermind",
      artist: "Nirvana",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
  ],
  "R&B": [
    {
      id: "rnb-1",
      title: "SOS",
      artist: "SZA",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "rnb-2",
      title: "Blonde",
      artist: "Frank Ocean",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "rnb-3",
      title: "After Hours",
      artist: "The Weeknd",
      coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Indie: [
    {
      id: "ind-1",
      title: "Currents",
      artist: "Tame Impala",
      coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ind-2",
      title: "Punisher",
      artist: "Phoebe Bridgers",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ind-3",
      title: "Immunity",
      artist: "Clairo",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Electronic: [
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
      title: "Settle",
      artist: "Disclosure",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Chill: [
    {
      id: "ch-1",
      title: "For Emma, Forever Ago",
      artist: "Bon Iver",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ch-2",
      title: "Cry",
      artist: "Cigarettes After Sex",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ch-3",
      title: "In Between Dreams",
      artist: "Jack Johnson",
      coverUrl: "https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Focus: [
    {
      id: "foc-1",
      title: "Music for Airports",
      artist: "Brian Eno",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "foc-2",
      title: "Immunity",
      artist: "Jon Hopkins",
      coverUrl: "https://images.unsplash.com/photo-1614613535308-eb5fbd3d2c17?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "foc-3",
      title: "All Melody",
      artist: "Nils Frahm",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Workout: [
    {
      id: "wo-1",
      title: "The Eminem Show",
      artist: "Eminem",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "wo-2",
      title: "Graduation",
      artist: "Kanye West",
      coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "wo-3",
      title: "Quest For Fire",
      artist: "Skrillex",
      coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Party: [
    {
      id: "pa-1",
      title: "BRAT",
      artist: "Charli xcx",
      coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "pa-2",
      title: "Funk Wav Bounces",
      artist: "Calvin Harris",
      coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "pa-3",
      title: "RENAISSANCE",
      artist: "Beyoncé",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Jazz: [
    {
      id: "jz-1",
      title: "Kind of Blue",
      artist: "Miles Davis",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "jz-2",
      title: "Bewitched",
      artist: "Laufey",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "jz-3",
      title: "A Love Supreme",
      artist: "John Coltrane",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Classical: [
    {
      id: "cl-1",
      title: "Divenire",
      artist: "Ludovico Einaudi",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "cl-2",
      title: "Sleep",
      artist: "Max Richter",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "cl-3",
      title: "re:member",
      artist: "Ólafur Arnalds",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
  ],
  "Lo-fi": [
    {
      id: "lf-1",
      title: "Life",
      artist: "Jinsang",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "lf-2",
      title: "Rainy Evening",
      artist: "Idealism",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "lf-3",
      title: "Harbor",
      artist: "Tomppabeats",
      coverUrl: "https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Metal: [
    {
      id: "met-1",
      title: "Master of Puppets",
      artist: "Metallica",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "met-2",
      title: "White Pony",
      artist: "Deftones",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "met-3",
      title: "Toxicity",
      artist: "System of a Down",
      coverUrl: "https://images.unsplash.com/photo-1470225620780-dba8ba36b745?w=400&auto=format&fit=crop&q=80",
    },
  ],
  "K-Pop": [
    {
      id: "kp-1",
      title: "Get Up",
      artist: "NewJeans",
      coverUrl: "https://images.unsplash.com/photo-1514525253161-7a46d19cd819?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "kp-2",
      title: "Map of the Soul: 7",
      artist: "BTS",
      coverUrl: "https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "kp-3",
      title: "THE ALBUM",
      artist: "BLACKPINK",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Country: [
    {
      id: "co-1",
      title: "Zach Bryan",
      artist: "Zach Bryan",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "co-2",
      title: "Golden Hour",
      artist: "Kacey Musgraves",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "co-3",
      title: "Traveller",
      artist: "Chris Stapleton",
      coverUrl: "https://images.unsplash.com/photo-1498038432885-c6f3f1b912ee?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Soul: [
    {
      id: "so-1",
      title: "Coming Home",
      artist: "Leon Bridges",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "so-2",
      title: "Back to Black",
      artist: "Amy Winehouse",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "so-3",
      title: "What's Going On",
      artist: "Marvin Gaye",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
  ],
  Acoustic: [
    {
      id: "ac-1",
      title: "+",
      artist: "Ed Sheeran",
      coverUrl: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ac-2",
      title: "O",
      artist: "Damien Rice",
      coverUrl: "https://images.unsplash.com/photo-1508700115892-45ecd05ae2ad?w=400&auto=format&fit=crop&q=80",
    },
    {
      id: "ac-3",
      title: "Our Endless Numbered Days",
      artist: "Iron & Wine",
      coverUrl: "https://images.unsplash.com/photo-1518609878373-06d740f60d8b?w=400&auto=format&fit=crop&q=80",
    },
  ],
};

const MotionLink = motion.create(Link);

export function GenreCrate({
  genre,
  albums: customAlbums,
  to,
  onPlay,
  style,
}: {
  genre: string;
  albums?: CrateAlbum[];
  to?: string;
  onPlay?: () => void;
  style?: React.CSSProperties;
}) {
  const [hover, setHover] = useState(false);
  const [playing, setPlaying] = useState(false);

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

    // Fallback if fewer than 3 albums found
    const fallbacks = FALLBACK_GENRE_ALBUMS[genre] || FALLBACK_GENRE_ALBUMS.Pop;
    for (const fb of fallbacks) {
      if (collected.length >= 3) break;
      if (!seenIds.has(fb.id)) {
        seenIds.add(fb.id);
        collected.push(fb);
      }
    }

    return collected.length >= 3 ? collected : fallbacks;
  }, [customAlbums, genre, searchResults, savedAlbums, topTracks]);

  // Pick 3 cards to show in the crate slot
  const visibleCards = [
    albums[0] || albums[0],
    albums[1] || albums[0],
    albums[2] || albums[0],
  ];

  const destination = to || `/search?q=${encodeURIComponent(genre)}`;

  return (
    <MotionLink
      to={destination}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -4 }}
      transition={{ type: "spring", stiffness: 420, damping: 30 }}
      style={{
        position: "relative",
        display: "block",
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
        textDecoration: "none",
        userSelect: "none",
        boxShadow: hover
          ? "0 14px 32px rgba(0, 0, 0, 0.45)"
          : "0 4px 14px rgba(0, 0, 0, 0.2)",
        transition: "all 0.22s cubic-bezier(0.23, 1, 0.32, 1)",
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
            fontSize: "clamp(13px, 1.4vw, 15px)",
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
          size={28}
          iconSize={12}
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setPlaying(!playing);
            onPlay?.();
          }}
        />
      </div>
    </MotionLink>
  );
}