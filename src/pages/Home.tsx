import { useState, useRef, useLayoutEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { Heart, Music2, ListMusic, Disc3, Users } from "lucide-react";
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
import { useQueueStore } from "../store/queue.store";
import { useSpeedDialStore, type SpeedDialEntry } from "../store/speedDial.store";
import { playTrack } from "../api/playback";
import { transportPlay, transportPause } from "../hooks/usePlayerControls";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import { ArtistCard } from "../components/ui/ArtistCard";
import { AlbumCard } from "../components/ui/AlbumCard";
import { EvenGrid, EvenGridSkeleton } from "../components/ui/EvenGrid";
import { CirclePlayButton } from "../components/ui/CirclePlayButton";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { meshGradient } from "../lib/mesh";
import { gpuLayer, zTransform } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { TrackItem, ArtistItem } from "../types/spotify";
import type { TimeRange } from "../types/library";

// grid reflow spring for smooth panel gliding
const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };

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

function QuickActionCard({
  item,
  recentTracks,
  index = 0,
}: {
  item: QuickItem;
  recentTracks: TrackItem[];
  index?: number;
}) {
  const [hover, setHover] = useState(false);
  const navigate = useNavigate();
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const setPlaying = usePlayerStore((s) => s.setPlaying);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);
  const contextId = useQueueStore((s) => s.contextId);

  const isThisPlaying = Boolean(
    isPlaying && (
      (item.isLikedSongs && (contextId === "liked-songs" || contextId === "liked")) ||
      (item.playlistId && contextId === item.playlistId) ||
      (item.albumId && contextId === item.albumId) ||
      (item.artistId && (contextId === item.artistId || contextId === `artist-top-${item.artistId}`)) ||
      (item.track && currentTrack?.id === item.track.id)
    )
  );

  async function handlePlay(e?: React.MouseEvent) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }

    if (isThisPlaying) {
      transportPause();
      return;
    }

    if (item.track && currentTrack?.id === item.track.id && !isPlaying) {
      transportPlay();
      return;
    }

    // 1. Liked Songs
    if (item.isLikedSongs || item.id === "liked-songs") {
      try {
        const tracks = await getLikedSongs(100, 0);
        if (tracks && tracks.length > 0) {
          const start = playContext(tracks, 0, "liked-songs");
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
          const start = playContext(tracks, 0, item.playlistId);
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
          const start = playContext(tracks, 0, item.albumId);
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
          const start = playContext(tracks, 0, item.artistId);
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

  return (
    <motion.div
      role="button"
      tabIndex={0}
      layout="position"
      transformTemplate={zTransform}
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{
        opacity: { duration: 0.28, delay: index * 0.035 },
        y: { duration: 0.28, delay: index * 0.035 },
        layout: REFLOW,
      }}
      onClick={() => handlePlay()}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") handlePlay(); }}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: "flex",
        alignItems: "center",
        height: 64,
        borderRadius: 8,
        overflow: "hidden",
        cursor: "pointer",
        background: hover
          ? "var(--color-surface-hover, rgba(255,255,255,0.12))"
          : "var(--color-surface, rgba(255,255,255,0.06))",
        border: "1px solid rgba(255, 255, 255, 0.05)",
        transition: "background 0.16s ease, box-shadow 0.16s ease",
        boxShadow: hover ? "0 8px 24px rgba(0,0,0,0.32)" : "0 2px 8px rgba(0,0,0,0.18)",
        userSelect: "none",
        minWidth: 0,
        ...gpuLayer,
      }}
    >
      {item.isLikedSongs ? (
        <div
          style={{
            width: 64,
            height: 64,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "linear-gradient(135deg, #450af5 0%, #8e8ee5 100%)",
            color: "#ffffff",
          }}
        >
          <Heart size={22} fill="#ffffff" strokeWidth={0} />
        </div>
      ) : item.imageUrl ? (
        <img
          src={item.imageUrl}
          alt=""
          loading="lazy"
          style={{
            width: 64,
            height: 64,
            flexShrink: 0,
            objectFit: "cover",
          }}
        />
      ) : (
        <div
          style={{
            width: 64,
            height: 64,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(255,255,255,0.06)",
            color: "var(--color-text-dim)",
          }}
        >
          {item.playlistId ? (
            <ListMusic size={22} strokeWidth={1.8} />
          ) : item.artistId ? (
            <Users size={22} strokeWidth={1.8} />
          ) : item.albumId ? (
            <Disc3 size={22} strokeWidth={1.8} />
          ) : (
            <Music2 size={22} strokeWidth={1.8} />
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
              <motion.div
                key={i}
                animate={{ scaleY: [0.2, 1.0, 0.2] }}
                transition={{ repeat: Infinity, duration: 0.55 + i * 0.15, ease: "easeInOut" }}
                style={{ flex: 1, height: "100%", borderRadius: 1, background: "var(--color-accent)", transformOrigin: "bottom" }}
              />
            ))}
          </div>
        )}
      </div>

      <div style={{ marginRight: 12, flexShrink: 0 }}>
        <CirclePlayButton
          isPlaying={isThisPlaying}
          visible={hover || isThisPlaying}
          onClick={(e) => handlePlay(e)}
          size={42}
          iconSize={17}
          ariaLabel={isThisPlaying ? `Pause ${item.title}` : `Play ${item.title}`}
        />
      </div>
    </motion.div>
  );
}

function QuickActionsShelf() {
  const containerRef = useRef<HTMLDivElement>(null);

  const [cols, setCols] = useState<number>(() => {
    if (typeof window !== "undefined") {
      const w = Math.max(320, window.innerWidth - (window.innerWidth < 768 ? 88 : 280));
      return w >= 680 ? 3 : w >= 400 ? 2 : 1;
    }
    return 3;
  });

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    let rafId = 0;
    const measure = (w: number) => {
      if (w <= 0) return;
      const nextCols = w >= 680 ? 3 : w >= 400 ? 2 : 1;
      setCols((prev) => (prev === nextCols ? prev : nextCols));
    };
    measure(el.getBoundingClientRect().width);
    const ro = new ResizeObserver((entries) => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        for (const entry of entries) {
          measure(entry.contentRect.width);
        }
      });
    });
    ro.observe(el);
    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      ro.disconnect();
    };
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
        />
      ))}
    </motion.section>
  );
}

// --- section scaffolding ---

function SectionTitle({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 12,
        flexWrap: "wrap",
        rowGap: 8,
        margin: "0 0 14px",
      }}
    >
      <h2
        style={{
          margin: 0,
          fontSize: "clamp(17px, 2.2vw, 21px)",
          fontWeight: 700,
          letterSpacing: "-0.02em",
          color: "var(--color-text-hi)",
          textWrap: "balance",
        } as React.CSSProperties}
      >
        {children}
      </h2>
      {right}
    </div>
  );
}

function TileSkeleton() {
  return <EvenGridSkeleton minColWidth={140} gap={14} maxRows={1} />;
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
      transformTemplate={zTransform}
      onClick={onPlay}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileTap={{ scale: 0.97 }}
      style={{
        width: "100%",
        display: "flex",
        flexDirection: "column",
        gap: 10,
        padding: 8,
        borderRadius: 14,
        border: "none",
        background: hover ? "var(--color-surface)" : "transparent",
        cursor: "pointer",
        textAlign: "left",
        transition: "background 0.18s ease",
        boxSizing: "border-box",
        ...gpuLayer,
      }}
    >
      <div style={{ position: "relative", width: "100%", aspectRatio: "1 / 1" }}>
        {art ? (
          <img
            src={art}
            alt=""
            loading="lazy"
            decoding="async"
            style={{ width: "100%", height: "100%", borderRadius: 10, objectFit: "cover", outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1 }}
          />
        ) : (
          <div style={{ width: "100%", height: "100%", borderRadius: 10, outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1, overflow: "hidden", ...meshGradient(track.id) }} />
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
          size={42}
          iconSize={17}
          style={{ position: "absolute", right: 10, bottom: 10 }}
          ariaLabel={isThisTrackPlaying ? `Pause ${track.name}` : `Play ${track.name}`}
        />
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 6, width: "100%", minWidth: 0 }}>
        <span
          style={{
            fontSize: 13,
            fontWeight: 600,
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
      <span style={{ fontSize: 12, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%", width: "100%", display: "block", marginTop: -4 }}>
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
      minColWidth={140}
      gap={14}
      maxRows={2}
      getKey={(t) => t.id}
      renderItem={(t, i) => (
        <RecTile track={t} onPlay={() => play(i)} />
      )}
    />
  );
}

function MadeForYou() {
  const { data: recs = [], isLoading } = useQuery({
    queryKey:  ["recommendations", "home"],
    queryFn:   () => getRecommendations(undefined, 16),
    staleTime: 30 * 60_000,
    gcTime:    24 * 60 * 60_000,
    placeholderData: (prev) => prev,
    refetchOnWindowFocus: false,
  });

  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-label="Made for you">
      <SectionTitle>Made for you</SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : recs.length === 0 ? (
        <p style={{ margin: 0, fontSize: 13, color: "var(--color-text-dim)" }}>
          Play and follow some artists, recommendations will grow here.
        </p>
      ) : (
        <TrackTiles tracks={recs} context="made-for-you" />
      )}
    </motion.section>
  );
}

function RecentlyPlayed() {
  const { data = [], isLoading } = useRecentlyPlayed();
  if (!isLoading && data.length === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-label="Jump back in">
      <SectionTitle>Jump back in</SectionTitle>
      {isLoading ? <TileSkeleton /> : <TrackTiles tracks={data.slice(0, 16)} context="recently-played" />}
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
  return <p style={{ margin: 0, fontSize: 12.5, color: "var(--color-text-dim)" }}>{children}</p>;
}

const rangeWord = (r: TimeRange) =>
  r === "short_term" ? "the last 4 weeks" : r === "long_term" ? "all time" : "the last 6 months";

function ArtistTiles({ artists }: { artists: ArtistItem[] }) {
  return (
    <EvenGrid
      items={artists}
      minColWidth={126}
      gap={14}
      maxRows={2}
      getKey={(a) => a.id}
      renderItem={(a) => <ArtistCard artist={a} />}
    />
  );
}

function TopTracks() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading } = useTopTracks(range);
  const probe = useTopTracks("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-label="Your top tracks">
      <SectionTitle right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-tracks-range" />}>Your top tracks</SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : data.length === 0 ? (
        <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
      ) : (
        <TrackTiles tracks={data.slice(0, 16)} context={`top-tracks-${range}`} />
      )}
    </motion.section>
  );
}

function TopArtists() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading } = useTopArtists(range);
  const probe = useTopArtists("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-label="Your top artists">
      <SectionTitle right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-artists-range" />}>Your top artists</SectionTitle>
      {isLoading ? (
        <EvenGridSkeleton minColWidth={126} gap={14} maxRows={1} borderRadius={999} />
      ) : data.length === 0 ? (
        <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
      ) : (
        <ArtistTiles artists={data.slice(0, 16)} />
      )}
    </motion.section>
  );
}

function NewReleases() {
  const { data = [], isLoading } = useNewReleases();
  if (!isLoading && data.length === 0) return null;
  return (
    <motion.section layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }} aria-label="New releases">
      <SectionTitle>New releases</SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : (
        <EvenGrid
          items={data.slice(0, 16)}
          minColWidth={140}
          gap={14}
          maxRows={2}
          getKey={(al) => al.id}
          renderItem={(al) => <AlbumCard album={al} />}
        />
      )}
    </motion.section>
  );
}

function greeting() {
  const h = new Date().getHours();
  return h < 12 ? "Good morning" : h < 18 ? "Good afternoon" : "Good evening";
}

export default function Home() {
  useReflowPulse();
  const { loggedIn, displayName, isLoading, login, loggingIn } = useAuth();
  const hello = greeting();

  if (isLoading) {
    return (
      <div>
        <h1 style={{ margin: "0 0 16px", fontSize: 30, fontWeight: 700, letterSpacing: "-0.02em", color: "var(--color-text-hi)" }}>{hello}</h1>
        <Loader fill={false} />
      </div>
    );
  }

  if (!loggedIn) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "clamp(24px, 3vw, 36px)" }}>
        <div>
          <p style={{ margin: "0 0 4px", fontSize: 13, fontWeight: 600, letterSpacing: "0.02em", color: "var(--color-text-dim)" }}>{hello}</p>
          <h1 style={{ margin: 0, fontSize: "clamp(26px, 3.8vw, 34px)", fontWeight: 800, letterSpacing: "-0.03em", color: "var(--color-text-hi)" }}>
            Welcome
          </h1>
        </div>

        <EmptyState
          icon={
            <MusiqueLogo
              size={56}
              style={{
                filter: "drop-shadow(0 8px 24px rgba(88, 115, 216, 0.32))",
              }}
            />
          }
          iconContainerStyle={{ opacity: 1, marginBottom: 8 }}
          title="Sign in with Spotify"
          description="Connect your account to listen to your music, playlists, and recommendations."
          action={
            <button
              onClick={() => login()}
              disabled={loggingIn}
              style={{
                height: 38,
                padding: "0 24px",
                borderRadius: 99,
                border: "none",
                background: "var(--color-accent)",
                color: "#ffffff",
                fontSize: 13,
                fontWeight: 600,
                cursor: loggingIn ? "default" : "pointer",
                opacity: loggingIn ? 0.6 : 1,
                transition: "opacity 0.15s ease",
              }}
            >
              {loggingIn ? "Waiting for browser..." : "Log in with Spotify"}
            </button>
          }
          hint={loggingIn ? "Finish signing in in your browser. The app is listening on port 8989." : undefined}
        />
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(26px, 3.4vw, 38px)" }}>
      <motion.div layout="position" transformTemplate={zTransform} transition={{ layout: REFLOW }}>
        <p style={{ margin: "0 0 4px", fontSize: 13, fontWeight: 600, letterSpacing: "0.02em", color: "var(--color-text-dim)" }}>{hello}</p>
        <h1 style={{ margin: 0, fontSize: "clamp(26px, 3.8vw, 34px)", fontWeight: 800, letterSpacing: "-0.03em", color: "var(--color-text-hi)", textWrap: "balance" } as React.CSSProperties}>
          {displayName ? displayName.split(" ")[0] : "Welcome back"}
        </h1>
      </motion.div>

      {/* top 6 quick action shelf */}
      <QuickActionsShelf />

      <MadeForYou />
      <RecentlyPlayed />
      <TopTracks />
      <TopArtists />
      <NewReleases />
    </div>
  );
}
