import { useState, useRef, useLayoutEffect, useEffect, useMemo, memo } from "react";
import { coverUrl } from "../lib/coverUrl";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { Heart, Music2, ListMusic, Disc3, Users } from "@/lib/icons";
import { useAuth } from "../hooks/useAuth";
import { getRecommendations, getAlbum, getArtist, getPlaylist, getTrack } from "../api/spotify";
import { getLikedSongs } from "../api/library";
import {
  useRecentlyPlayed,
  useTopTracks,
  useTopArtists,
  useNewReleases,
  useMyPlaylists,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { LIKED_SONGS_URI, useQueueStore } from "../store/queue.store";
import { useSpeedDialStore, type SpeedDialEntry } from "../store/speedDial.store";
import { playTrack } from "../api/playback";
import { transportPlay, transportPause } from "../hooks/usePlayerControls";
import { Loader } from "../components/ui/Loader";
import { SignInPrompt } from "../components/ui/SignInPrompt";
import { SectionTitle } from "../components/ui/SectionTitle";
import { useCarousel, CarouselControls, CarouselTrack } from "../components/ui/Carousel";
import { ArtistCard } from "../components/ui/ArtistCard";
import { AlbumCard } from "../components/ui/AlbumCard";
import { EvenGrid, EvenGridSkeleton } from "../components/ui/EvenGrid";
import { CirclePlayButton } from "../components/ui/CirclePlayButton";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { meshGradient } from "../lib/mesh";
import { gpuLayer, zTransform, EASE_OUT, PRESS } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { TrackItem, ArtistItem } from "../types/spotify";
import type { TimeRange } from "../types/library";
import { HomeFeedShelves } from "../components/ui/HomeFeedShelves";
import { TILE_PAD, TILE_BLEED } from "../lib/layout";
import { flipChildren, useRailPin } from "../lib/railFlip";
import { flushSync } from "react-dom";

// grid reflow spring for smooth panel gliding (critically damped)
const REFLOW = { type: "spring" as const, stiffness: 340, damping: 37 };

/* the quick-action cascade plays once per session. Home is the screen people
return to most; replaying a stagger on every back-navigation reads as the page
being slow to arrive, when it was already there. */
let quickShelfHasEntered = false;

// every track tile renders at this exact size regardless of window width
const TILE_COVER = 164;
const TILE_H     = 212; // 20 padding + 144 cover + 2×8 gap + 17 title + 15 artist

// --- top 6 quick action cards ---
interface QuickItem {
  id: string;
  title: string;
  imageUrl?: string | null;
  to: string;
  type?: "liked-songs" | "playlist" | "album" | "artist" | "track";
  track?: TrackItem;
  albumId?: string;
  playlistId?: string;
  artistId?: string;
  isLikedSongs?: boolean;
}

const QuickActionCard = memo(function QuickActionCard({
  item,
  recentTracks,
  index = 0,
  animateIn,
}: {
  item: QuickItem;
  recentTracks: TrackItem[];
  index?: number;
  animateIn: boolean;
}) {
  const [hover, setHover] = useState(false);
  const [focused, setFocused] = useState(false);
  // the click was heard; playback's context is being fetched
  const [pending, setPending] = useState(false);
  const navigate = useNavigate();
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const isCurrentTrackMatch = usePlayerStore((s) => Boolean(item.track && s.currentTrack?.id === item.track.id));
  const contextId = useQueueStore((s) => s.contextId);

  const isContextMatch = Boolean(
    (item.isLikedSongs && (contextId === "liked-songs" || contextId === "liked")) ||
    (item.playlistId && contextId === item.playlistId) ||
    (item.albumId && contextId === item.albumId) ||
    (item.artistId && (contextId === item.artistId || contextId === `artist-top-${item.artistId}`))
  );

  const isThisPlaying = Boolean(isPlaying && (isContextMatch || isCurrentTrackMatch));

  async function handlePlay(e?: React.MouseEvent) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (pending) return;

    if (isThisPlaying) {
      transportPause();
      return;
    }

    if (isCurrentTrackMatch && !isPlaying) {
      transportPlay();
      return;
    }

    setPending(true);
    try {
      await startPlayback();
    } finally {
      setPending(false);
    }
  }

  async function startPlayback() {
    const playContext = useQueueStore.getState().playContext;
    const setCurrentTrack = usePlayerStore.getState().setCurrentTrack;
    const setPlaying = (val: boolean) => usePlayerStore.getState().setPlaying(val);

    // 1. Liked Songs
    if (item.isLikedSongs || item.id === "liked-songs") {
      try {
        const tracks = await getLikedSongs(100, 0);
        if (tracks && tracks.length > 0) {
          const start = playContext(tracks, 0, "liked-songs", LIKED_SONGS_URI);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
          }
          return;
        }
      } catch (err) {
        console.error("[speed-dial] Failed to play liked songs:", err);
      }
      navigate("/library?tab=songs");
      return;
    }

    // 2. Individual Track (Always play the exact track directly!)
    const trackId = item.track?.id || (item.type === "track" ? item.id.replace("track-", "") : null);
    if (item.track || (item.type === "track" && trackId)) {
      let target = item.track;
      if (!target && trackId) {
        try {
          const detail = await getTrack(trackId);
          if (detail) {
            target = {
              id: detail.id,
              name: detail.name,
              duration_ms: detail.duration_ms,
              explicit: detail.explicit,
              popularity: detail.popularity,
              artists: detail.artists,
              album: detail.album,
            };
          }
        } catch (err) {
          console.error("[speed-dial] Failed to fetch track:", err);
        }
      }
      if (target) {
        const pool = recentTracks.filter((t) => t.id !== target!.id);
        const contextTracks = [target, ...pool];
        const start = playContext(contextTracks, 0, "quick-action");
        const trackToPlay = start || target;
        setCurrentTrack(trackToPlay);
        playTrack(trackToPlay.id).then(() => setPlaying(true)).catch(() => {});
        return;
      }
    }

    // 3. Playlist
    if (item.playlistId) {
      try {
        const pl = await getPlaylist(item.playlistId);
        const tracks = pl?.tracks ?? [];
        if (tracks.length > 0) {
          const start = playContext(tracks, 0, item.playlistId, `spotify:playlist:${item.playlistId}`);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
          }
          return;
        }
      } catch (err) {
        console.error("[speed-dial] Failed to play playlist:", err);
      }
      navigate(`/playlist/${item.playlistId}`);
      return;
    }

    // 4. Album
    if (item.albumId) {
      try {
        const full = await getAlbum(item.albumId);
        const tracks = full?.tracks ?? [];
        if (tracks.length > 0) {
          const start = playContext(tracks, 0, item.albumId, `spotify:album:${item.albumId}`);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
          }
          return;
        }
      } catch (err) {
        console.error("[speed-dial] Failed to play album:", err);
      }
      navigate(`/album/${item.albumId}`);
      return;
    }

    // 5. Artist
    if (item.artistId) {
      try {
        const artist = await getArtist(item.artistId);
        const tracks = artist?.top_tracks ?? [];
        if (tracks.length > 0) {
          const start = playContext(tracks, 0, item.artistId, `spotify:artist:${item.artistId}`);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
          }
          return;
        }
      } catch (err) {
        console.error("[speed-dial] Failed to play artist:", err);
      }
      navigate(`/artist/${item.artistId}`);
      return;
    }

    navigate(item.to);
  }

  /* the whole card is a mouse target, but the play button inside it is the one
  real control - so there is no role=button wrapping a <button>. keyboard users
  tab straight to the play button, and focusing it lights the card up the same
  way hovering does. */
  const lit = hover || focused;
  return (
    <motion.div
      layout="position"
      transformTemplate={zTransform}
      initial={animateIn ? { opacity: 0, y: 10 } : false}
      animate={{ opacity: 1, y: 0 }}
      whileTap={PRESS}
      transition={{
        opacity: { duration: 0.28, delay: index * 0.035, ease: EASE_OUT },
        y: { duration: 0.28, delay: index * 0.035, ease: EASE_OUT },
        scale: { duration: 0.12, ease: EASE_OUT },
        layout: REFLOW,
      }}
      onClick={() => handlePlay()}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      aria-busy={pending || undefined}
      style={{
        display: "flex",
        alignItems: "center",
        height: 60,
        padding: "5px 12px 5px 6px",
        borderRadius: 10,
        overflow: "hidden",
        cursor: pending ? "progress" : "pointer",
        background: lit
          ? "rgba(255, 255, 255, 0.09)"
          : "rgba(255, 255, 255, 0.045)",
        border: "1px solid rgba(255, 255, 255, 0.07)",
        transition: "background 0.16s ease, box-shadow 0.16s ease",
        boxShadow: lit ? "0 10px 24px rgba(0,0,0,0.42)" : "0 2px 6px rgba(0,0,0,0.18)",
        userSelect: "none",
        minWidth: 0,
        ...gpuLayer,
      }}
    >
      {item.isLikedSongs ? (
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: 6,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "linear-gradient(135deg, #450af5 0%, #8e8ee5 100%)",
            color: "#ffffff",
            boxShadow: "0 2px 8px rgba(69, 10, 245, 0.35)",
          }}
        >
          <Heart size={20} fill="#ffffff" strokeWidth={0} />
        </div>
      ) : item.imageUrl ? (
        <img
          src={coverUrl(item.imageUrl, 48) ?? item.imageUrl}
          alt=""
          loading="lazy"
          style={{
            width: 48,
            height: 48,
            borderRadius: 6,
            flexShrink: 0,
            objectFit: "cover",
            boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
          }}
        />
      ) : (
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: 6,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(255,255,255,0.06)",
            color: "var(--color-text-dim)",
            boxShadow: "0 2px 8px rgba(0,0,0,0.2)",
          }}
        >
          {item.playlistId ? (
            <ListMusic size={20} strokeWidth={1.8} />
          ) : item.artistId ? (
            <Users size={20} strokeWidth={1.8} />
          ) : item.albumId ? (
            <Disc3 size={20} strokeWidth={1.8} />
          ) : (
            <Music2 size={20} strokeWidth={1.8} />
          )}
        </div>
      )}

      <div style={{ flex: 1, minWidth: 0, padding: "0 14px", display: "flex", alignItems: "center", gap: 8 }}>
        <span
          style={{
            fontSize: 14,
            fontWeight: 700,
            letterSpacing: "-0.01em",
            color: isThisPlaying ? "var(--color-accent)" : "var(--color-text-hi)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            flex: 1,
            minWidth: 0,
          }}
        >
          {item.title}
        </span>
        {isThisPlaying && (
          <div style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 12, width: 12, flexShrink: 0 }}>
            {[0.4, 1.0, 0.6].map((_, i) => (
              <div
                key={i}
                className="eq-bar"
                style={{
                  flex: 1,
                  height: "100%",
                  borderRadius: 1,
                  background: "var(--color-accent)",
                  ["--eq-dur" as string]: `${0.55 + i * 0.15}s`,
                }}
              />
            ))}
          </div>
        )}
      </div>

      <div style={{ marginRight: 12, flexShrink: 0 }}>
        <CirclePlayButton
          isPlaying={isThisPlaying}
          visible={lit || isThisPlaying || pending}
          pending={pending}
          onClick={(e) => handlePlay(e)}
          size={42}
          iconSize={17}
          ariaLabel={isThisPlaying ? `Pause ${item.title}` : `Play ${item.title}`}
        />
      </div>
    </motion.div>
  );
});

function QuickActionsShelf() {
  const containerRef = useRef<HTMLDivElement>(null);

  const [cols, setCols] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const w = Math.max(320, window.innerWidth - (window.innerWidth < 768 ? 88 : 280));
      return w >= 680 ? 3 : w >= 400 ? 2 : 1;
    }
    return 3;
  });

  /* Re-column as the shelf's width crosses 680 / 400. The cards slide to
     their new cells and grow or shrink to their new width together
     (flipChildren, "size" mode). They used to glide their position while
     their width snapped - a card jumping from a third of the row to half of
     it mid-move, which read as a stutter. */
  const colsRef = useRef(cols);
  colsRef.current = cols;
  const colsFor = (w: number) => (w >= 680 ? 3 : w >= 400 ? 2 : 1);
  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    let rafId = 0;
    const ro = new ResizeObserver((entries) => {
      const w = entries[entries.length - 1]?.contentRect.width ?? 0;
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        if (w <= 0) return;
        const next = colsFor(w);
        if (next === colsRef.current) return;
        flipChildren(el, () => flushSync(() => setCols(next)), "size");
      });
    });
    ro.observe(el);
    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      ro.disconnect();
    };
  }, []);

  // a right-rail slide pins this grid: re-column now, before the FLIP reads it
  useRailPin(containerRef, () => {
    const w = containerRef.current?.getBoundingClientRect().width ?? 0;
    if (w > 0) setCols(colsFor(w));
  });

  // captured once per mount: true only for the first Home visit this session
  const [animateIn] = useState(() => !quickShelfHasEntered);
  useEffect(() => {
    quickShelfHasEntered = true;
  }, []);

  const speedDialEntries = useSpeedDialStore((s) => s.entries);
  const { data: playlists = [] } = useMyPlaylists();
  const { data: recentTracks = [] } = useRecentlyPlayed();
  const { data: topTracks = [] } = useTopTracks("short_term");
  const { data: topArtists = [] } = useTopArtists("short_term");

  const items = useMemo<QuickItem[]>(() => {
    const inAppMap = new Map<string, SpeedDialEntry>();
    for (const entry of speedDialEntries) {
      inAppMap.set(entry.id, entry);
    }

    interface ScoredCandidate {
      item: QuickItem;
      score: number;
      lastPlayedAt: number;
    }

    const candidateMap = new Map<string, ScoredCandidate>();

    function addOrUpdateCandidate(
      item: QuickItem,
      baseScore: number,
      lastPlayedAt: number = 0
    ) {
      const inApp = inAppMap.get(item.id);
      const inAppPlays = inApp?.playCount || 0;
      const inAppLastPlayed = inApp?.lastPlayedAt || inApp?.timestamp || 0;
      // In-app plays add heavy rotation weight (+5 pts per play)
      const totalScore = baseScore + inAppPlays * 5;
      const effectiveLastPlayed = Math.max(lastPlayedAt, inAppLastPlayed);

      const existing = candidateMap.get(item.id);
      if (!existing || totalScore > existing.score) {
        candidateMap.set(item.id, {
          item: {
            ...item,
            albumId: item.albumId || item.track?.album?.id,
          },
          score: totalScore,
          lastPlayedAt: effectiveLastPlayed,
        });
      }
    }

    // 1. Liked Songs (Perennial user favorite with strong baseline)
    addOrUpdateCandidate(
      {
        id: "liked-songs",
        title: "Liked Songs",
        to: "/library?tab=songs",
        isLikedSongs: true,
        type: "liked-songs",
      },
      25,
      Date.now()
    );

    // 2. In-App Played Items from speedDialEntries (Albums, Playlists, Artists, Tracks)
    for (const entry of speedDialEntries) {
      if (entry.id === "liked-songs") continue;
      addOrUpdateCandidate(
        {
          id: entry.id,
          type: entry.type,
          title: entry.title,
          imageUrl: entry.imageUrl,
          to: entry.to,
          track: entry.track,
          albumId: entry.albumId || entry.track?.album?.id,
          playlistId: entry.playlistId,
          artistId: entry.artistId,
        },
        0,
        entry.lastPlayedAt || entry.timestamp || 0
      );
    }

    // 3. User's Top Tracks from Spotify (The user's official most-played songs)
    const albumScoresFromTracks = new Map<string, { album: any; scoreSum: number; trackCount: number }>();
    topTracks.slice(0, 20).forEach((track, index) => {
      // Rank 0 is #1 most played track -> 22 points, down to min 4 points
      const spotifyScore = Math.max(4, 22 - index);
      addOrUpdateCandidate(
        {
          id: `track-${track.id}`,
          type: "track",
          title: track.name,
          imageUrl: track.album?.image_url,
          to: track.album?.id ? `/album/${track.album.id}` : "/library?tab=songs",
          track,
          albumId: track.album?.id,
        },
        spotifyScore,
        0
      );

      if (track.album?.id) {
        const existingAlbum = albumScoresFromTracks.get(track.album.id) || {
          album: track.album,
          scoreSum: 0,
          trackCount: 0,
        };
        existingAlbum.scoreSum += spotifyScore;
        existingAlbum.trackCount += 1;
        albumScoresFromTracks.set(track.album.id, existingAlbum);
      }
    });

    // 4. Albums with multiple top tracks or high listening frequency
    for (const [albId, info] of albumScoresFromTracks.entries()) {
      if (info.trackCount >= 2) {
        const albumBaseScore = Math.round(info.scoreSum * 0.75);
        addOrUpdateCandidate(
          {
            id: `album-${albId}`,
            type: "album",
            title: info.album.name,
            imageUrl: info.album.image_url,
            to: `/album/${albId}`,
            albumId: albId,
          },
          albumBaseScore,
          0
        );
      }
    }

    // 5. User's Saved Playlists
    playlists.slice(0, 8).forEach((pl, index) => {
      addOrUpdateCandidate(
        {
          id: `playlist-${pl.id}`,
          type: "playlist",
          title: pl.name,
          imageUrl: pl.image_url,
          to: `/playlist/${pl.id}`,
          playlistId: pl.id,
        },
        Math.max(2, 10 - index),
        0
      );
    });

    // 6. Top Artists from Spotify
    topArtists.slice(0, 6).forEach((artist, index) => {
      addOrUpdateCandidate(
        {
          id: `artist-${artist.id}`,
          type: "artist",
          title: artist.name,
          imageUrl: artist.image_url,
          to: `/artist/${artist.id}`,
          artistId: artist.id,
        },
        Math.max(2, 12 - index * 2),
        0
      );
    });

    // Sort all candidates primarily by MOST PLAYED score descending, then recency
    const sortedCandidates = Array.from(candidateMap.values()).sort((a, b) => {
      if (b.score !== a.score) {
        return b.score - a.score;
      }
      return b.lastPlayedAt - a.lastPlayedAt;
    });

    // Pick top 6 with STRICT Track & Album mutual exclusion deduplication:
    // "if a track is in an album, dont put the TRACK AND THE ALBUM BOTH IN THE TOP 6"
    const result: QuickItem[] = [];
    const seenIds = new Set<string>();
    const includedAlbumCards = new Set<string>(); // album IDs of album cards in top 6
    const includedTrackAlbums = new Set<string>(); // album IDs of track cards in top 6

    for (const { item } of sortedCandidates) {
      if (result.length >= 6) break;
      if (seenIds.has(item.id)) continue;

      if (item.type === "album" && item.albumId) {
        // If a track from this album is already in the top 6, do NOT put this album in the top 6!
        if (includedTrackAlbums.has(item.albumId)) {
          continue;
        }
        if (includedAlbumCards.has(item.albumId)) {
          continue;
        }
      } else if (item.type === "track") {
        const trackAlbumId = item.track?.album?.id || item.albumId;
        // If the album containing this track is already in the top 6, do NOT put this track in the top 6!
        if (trackAlbumId && includedAlbumCards.has(trackAlbumId)) {
          continue;
        }
      }

      // Accepted!
      seenIds.add(item.id);
      if (item.type === "album" && item.albumId) {
        includedAlbumCards.add(item.albumId);
      } else if (item.type === "track") {
        const trackAlbumId = item.track?.album?.id || item.albumId;
        if (trackAlbumId) {
          includedTrackAlbums.add(trackAlbumId);
        }
      }
      result.push(item);
    }

    return result;
  }, [speedDialEntries, playlists, topTracks, topArtists]);

  return (
    <motion.section
      ref={containerRef}
      layout="position"
      transformTemplate={zTransform}
      transition={{ layout: REFLOW }}
      data-rail-lock="flip-size"
      aria-label="Quick actions"
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gap: "clamp(8px, 1.2vw, 12px)",
        width: "100%",
        contain: "layout style",
        ...gpuLayer,
      }}
    >
      {items.map((item, i) => (
        <QuickActionCard
          key={item.id}
          item={item}
          recentTracks={recentTracks}
          index={i}
          animateIn={animateIn}
        />
      ))}
    </motion.section>
  );
}

// --- section scaffolding ---

function TileSkeleton() {
  return <EvenGridSkeleton minColWidth={TILE_COVER} gap={14} maxRows={1} />;
}

// --- recommendation / track tile ---

function RecTile({ track, onPlay }: { track: TrackItem; onPlay: () => void }) {
  const [hover, setHover] = useState(false);
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const isThisTrackPlaying = Boolean(currentTrack?.id === track.id && isPlaying);
  const art = track.album?.image_url;

  return (
    <motion.button
      layout="position"
      transformTemplate={zTransform}
      transition={{ layout: REFLOW }}
      onClick={onPlay}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileTap={{ scale: 0.97 }}
      style={{
        width: "100%",
        overflow: "hidden",
        minWidth: 0,
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
        gap: 8,
        padding: TILE_PAD,
        borderRadius: 12,
        border: "none",
        background: hover ? "var(--color-surface-hover, rgba(255,255,255,0.06))" : "transparent",
        cursor: "pointer",
        textAlign: "left",
        transition: "background 0.18s ease",
        boxSizing: "border-box",
        ...gpuLayer,
      }}
    >
      <div
        style={{
          position: "relative",
          width: "100%",
          aspectRatio: "1 / 1",
          borderRadius: 8,
          overflow: "hidden",
          boxShadow: hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)",
          transition: "box-shadow 0.25s ease",
          flexShrink: 0,
        }}
      >
        {art ? (
          <img
            src={coverUrl(art, 164) ?? art}
            alt=""
            loading="lazy"
            decoding="async"
            style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
          />
        ) : (
          <div style={{ width: "100%", height: "100%", overflow: "hidden", ...meshGradient(track.id) }} />
        )}
        <CirclePlayButton
          isPlaying={isThisTrackPlaying}
          visible={hover || isThisTrackPlaying}
          onClick={(e) => {
            e.stopPropagation();
            if (isThisTrackPlaying) {
              transportPause();
            } else {
              onPlay();
            }
          }}
          size={40}
          iconSize={16}
          style={{ position: "absolute", right: 8, bottom: 8 }}
          ariaLabel={isThisTrackPlaying ? `Pause ${track.name}` : `Play ${track.name}`}
        />
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 6, width: "100%", minWidth: 0, height: 17, flexShrink: 0 }}>
        <span
          style={{
            fontSize: 13.5,
            fontWeight: 600,
            letterSpacing: "-0.012em",
            lineHeight: "17px",
            height: 17,
            color: isThisTrackPlaying ? "var(--color-accent)" : "var(--color-text-hi)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            flex: 1,
            minWidth: 0,
          }}
        >
          {track.name}
        </span>
        {isThisTrackPlaying && (
          <div style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 11, width: 11, flexShrink: 0 }}>
            {[0.4, 1.0, 0.6].map((_, i) => (
              <div
                key={i}
                className="eq-bar"
                style={{
                  flex: 1,
                  height: "100%",
                  borderRadius: 1,
                  background: "var(--color-accent)",
                  ["--eq-dur" as string]: `${0.55 + i * 0.15}s`,
                }}
              />
            ))}
          </div>
        )}
      </div>
      <span className="t-caption" style={{ fontSize: 12, lineHeight: "15px", height: 15, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%", width: "100%", display: "block", flexShrink: 0, minWidth: 0 }}>
        {track.artists.map((a) => a.name).join(", ")}
      </span>
    </motion.button>
  );
}

// row of play-on-click track tiles that all share one playback context
function TrackTiles({ tracks, context }: { tracks: TrackItem[]; context: string }) {
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext     = useQueueStore((s) => s.playContext);

  function play(i: number) {
    const start = playContext(tracks, i, context);
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  return (
    <EvenGrid
      items={tracks}
      minColWidth={TILE_COVER}
      gap={14}
      style={TILE_BLEED}
      maxRows={2}
      getKey={(t) => t.id}
      renderItem={(t, i) => (
        <RecTile track={t} onPlay={() => play(i)} />
      )}
    />
  );
}

const tileKey = (t: { id: string }) => t.id;

function MadeForYou() {
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);
  const { data: recs = [], isLoading } = useQuery({
    queryKey:  ["recommendations", "home"],
    queryFn:   () => getRecommendations(undefined, 16),
    staleTime: 10 * 60_000,
    gcTime:    24 * 60 * 60_000,
    placeholderData: (prev) => prev,
    refetchOnWindowFocus: false,
  });
  const carousel = useCarousel([recs.length]);

  function play(i: number) {
    const start = playContext(recs, i, "made-for-you");
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-labelledby="home-top-picks">
      <SectionTitle id="home-top-picks" right={<CarouselControls carousel={carousel} label="top picks" />}>
        Top picks for you
      </SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : recs.length === 0 ? (
        <EmptyHint>Play and follow some artists, recommendations will grow here.</EmptyHint>
      ) : (
        <CarouselTrack
          carousel={carousel}
          label="Top picks for you"
          items={recs}
          getKey={tileKey}
          itemWidth={TILE_COVER}
          itemHeight={TILE_H}
          renderItem={(t, i) => <RecTile track={t} onPlay={() => play(i)} />}
        />
      )}
    </motion.section>
  );
}

function RecentlyPlayed() {
  const { data = [], isLoading } = useRecentlyPlayed();
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);
  const shown = data.slice(0, 18);
  const carousel = useCarousel([shown.length]);

  function play(i: number) {
    const start = playContext(data, i, "recently-played");
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  if (!isLoading && data.length === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-labelledby="home-jump-back">
      <SectionTitle id="home-jump-back" right={<CarouselControls carousel={carousel} label="recently played" />}>
        Jump back in
      </SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : (
        <CarouselTrack
          carousel={carousel}
          label="Jump back in"
          items={shown}
          getKey={tileKey}
          itemWidth={TILE_COVER}
          itemHeight={TILE_H}
          renderItem={(t, i) => <RecTile track={t} onPlay={() => play(i)} />}
        />
      )}
    </motion.section>
  );
}

const KEY_TO_LABEL: Record<TimeRange, string> = {
  short_term: "4 weeks",
  medium_term: "6 months",
  long_term: "All time",
};

const LABEL_TO_KEY: Record<string, TimeRange> = {
  "4 weeks": "short_term",
  "6 months": "medium_term",
  "All time": "long_term",
};

function RangeSlider({
  value,
  onChange,
  layoutId,
}: {
  value: TimeRange;
  onChange: (r: TimeRange) => void;
  layoutId: string;
}) {
  return (
    <SegmentedControl
      options={["4 weeks", "6 months", "All time"]}
      value={KEY_TO_LABEL[value]}
      onChange={(label) => onChange(LABEL_TO_KEY[label])}
      layoutId={layoutId}
    />
  );
}

function EmptyHint({ children }: { children: React.ReactNode }) {
  return <p className="t-caption" style={{ margin: 0, fontSize: 12.5, color: "var(--color-text-dim)" }}>{children}</p>;
}

/* while a new range loads, keep the last one on screen and dim it. swapping in
a skeleton blanked the whole grid on every segment tap; this keeps the page
continuous and the change reads as the content updating, not reloading. */
function Refreshing({ busy, children }: { busy: boolean; children: React.ReactNode }) {
  return (
    <div
      aria-busy={busy || undefined}
      style={{ opacity: busy ? 0.5 : 1, transition: "opacity 0.2s ease" }}
    >
      {children}
    </div>
  );
}

const rangeWord = (r: TimeRange) =>
  r === "short_term" ? "the last 4 weeks" : r === "long_term" ? "all time" : "the last 6 months";

function ArtistTiles({ artists }: { artists: ArtistItem[] }) {
  return (
    <EvenGrid
      items={artists}
      minColWidth={TILE_COVER}
      gap={14}
      style={TILE_BLEED}
      maxRows={2}
      getKey={(a) => a.id}
      renderItem={(a, i) => <ArtistCard artist={a} index={i} />}
    />
  );
}

function TopTracks() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading, isPlaceholderData, isError, refetch } = useTopTracks(range);
  const probe = useTopTracks("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-labelledby="home-top-tracks">
      <SectionTitle id="home-top-tracks" right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-tracks-range" />}>Your top tracks</SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : isError && data.length === 0 ? (
        <EmptyHint>
          Couldn't load this range.{" "}
          <button type="button" className="btn-text" onClick={() => refetch()} style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>
            Try again
          </button>
        </EmptyHint>
      ) : data.length === 0 ? (
        <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
      ) : (
        <Refreshing busy={isPlaceholderData}>
          <TrackTiles tracks={data.slice(0, 16)} context={`top-tracks-${range}`} />
        </Refreshing>
      )}
    </motion.section>
  );
}

function TopArtists() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading, isPlaceholderData, isError, refetch } = useTopArtists(range);
  const probe = useTopArtists("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-labelledby="home-top-artists">
      <SectionTitle id="home-top-artists" right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-artists-range" />}>Your top artists</SectionTitle>
      {isLoading ? (
        <EvenGridSkeleton minColWidth={TILE_COVER} gap={14} maxRows={1} borderRadius={999} />
      ) : isError && data.length === 0 ? (
        <EmptyHint>
          Couldn't load this range.{" "}
          <button type="button" className="btn-text" onClick={() => refetch()} style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>
            Try again
          </button>
        </EmptyHint>
      ) : data.length === 0 ? (
        <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
      ) : (
        <Refreshing busy={isPlaceholderData}>
          <ArtistTiles artists={data.slice(0, 16)} />
        </Refreshing>
      )}
    </motion.section>
  );
}

function NewReleases() {
  const { data = [], isLoading } = useNewReleases();
  const shown = data.slice(0, 18);
  const carousel = useCarousel([shown.length]);

  if (!isLoading && data.length === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-labelledby="home-new-releases">
      <SectionTitle id="home-new-releases" right={<CarouselControls carousel={carousel} label="new releases" />}>
        New releases
      </SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : (
        <CarouselTrack
          carousel={carousel}
          label="New releases"
          items={shown}
          getKey={tileKey}
          itemWidth={TILE_COVER}
          itemHeight={TILE_H}
          renderItem={(al, i) => <AlbumCard album={al} index={i} style={{ height: TILE_H, maxWidth: TILE_COVER }} />}
        />
      )}
    </motion.section>
  );
}

function greeting() {
  const h = new Date().getHours();
  return h < 12 ? "Good morning" : h < 18 ? "Good afternoon" : "Good evening";
}

// the greeting line. display role: large, light, tightly tracked; rem so it
// follows the root text size
const GREETING_STYLE: React.CSSProperties = {
  margin: 0,
  fontSize: "clamp(2rem, 3.8vw, 2.714rem)",
  fontWeight: 400,
  letterSpacing: "var(--type-display-track)",
  lineHeight: 1.14,
  color: "rgba(255, 255, 255, 0.62)",
  textWrap: "balance",
} as React.CSSProperties;

export default function Home() {
  useReflowPulse();
  const { loggedIn, displayName, isLoading } = useAuth();
  const hello = greeting();
  const firstName = displayName ? displayName.trim().split(" ")[0] : null;

  if (isLoading) {
    return (
      <div>
        <h1 style={{ ...GREETING_STYLE, margin: "0 0 16px" }}>{hello}</h1>
        <Loader fill={false} />
      </div>
    );
  }

  if (!loggedIn) {
    return (
      <SignInPrompt
        heading="Welcome to Musique"
        title="Sign in with Spotify"
        description="Connect your account to listen to your music, playlists, and recommendations."
      />
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(26px, 3.4vw, 38px)" }}>
      <motion.div
        layout="position"
        transformTemplate={zTransform}
        transition={{ layout: REFLOW }}
      >
        <h1 style={GREETING_STYLE}>
          {hello}
          {firstName ? (
            <>
              ,{" "}
              <span
                style={{
                  fontWeight: 800,
                  color: "var(--color-text-hi)",
                }}
              >
                {firstName}
              </span>
            </>
          ) : null}
        </h1>
      </motion.div>

      {/* top 6 quick action shelf */}
      <QuickActionsShelf />

      <HomeFeedShelves />
      <MadeForYou />
      <RecentlyPlayed />
      <TopTracks />
      <TopArtists />
      <NewReleases />
    </div>
  );
}
