import { useState, useRef, useLayoutEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { Heart, Music2 } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { getRecommendations, getAlbum, getArtist } from "../api/spotify";
import { getLikedSongs } from "../api/library";
import {
  useRecentlyPlayed,
  useTopTracks,
  useTopArtists,
  useNewReleases,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack, pausePlayback, resumePlayback } from "../api/playback";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import { ArtistCard } from "../components/ui/ArtistCard";
import { AlbumCard } from "../components/ui/AlbumCard";
import { EvenGrid, EvenGridSkeleton } from "../components/ui/EvenGrid";
import { CirclePlayButton } from "../components/ui/CirclePlayButton";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { meshGradient } from "../lib/mesh";
import type { TrackItem, ArtistItem } from "../types/spotify";
import type { TimeRange } from "../types/library";

// grid reflow spring for smooth panel gliding
const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };

// ─── top 6 quick action cards ────────────────────────────────────────────────

interface QuickItem {
  id: string;
  title: string;
  imageUrl?: string | null;
  to: string;
  track?: TrackItem;
  albumId?: string;
  artistId?: string;
  isLikedSongs?: boolean;
}

function QuickActionCard({
  item,
  recentTracks,
}: {
  item: QuickItem;
  recentTracks: TrackItem[];
}) {
  const [hover, setHover] = useState(false);
  const navigate = useNavigate();
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const setPlaying = usePlayerStore((s) => s.setPlaying);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);

  const isThisPlaying = Boolean(
    isPlaying && (
      (item.track && currentTrack?.id === item.track.id) ||
      (item.albumId && currentTrack?.album?.id === item.albumId)
    )
  );

  async function handlePlay(e?: React.MouseEvent) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }

    if (isThisPlaying) {
      pausePlayback().then(() => setPlaying(false)).catch(() => {});
      return;
    }

    if (item.track && currentTrack?.id === item.track.id && !isPlaying) {
      resumePlayback().then(() => setPlaying(true)).catch(() => {});
      return;
    }

    if (item.isLikedSongs) {
      try {
        const tracks = await getLikedSongs(50, 0);
        if (tracks && tracks.length > 0) {
          const start = playContext(tracks, 0, "liked-songs");
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
          }
          return;
        }
      } catch {}
      navigate("/library?tab=songs");
      return;
    }

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
      } catch {}
    }

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
      } catch {}
    }

    if (item.track) {
      const contextTracks = recentTracks.length > 0 ? recentTracks : [item.track];
      const idx = contextTracks.findIndex((t) => t.id === item.track!.id);
      const start = playContext(contextTracks, idx >= 0 ? idx : 0, "quick-action");
      const trackToPlay = start || item.track;
      setCurrentTrack(trackToPlay);
      playTrack(trackToPlay.id).then(() => setPlaying(true)).catch(() => {});
      return;
    }

    navigate(item.to);
  }

  return (
    <div
      role="button"
      tabIndex={0}
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
          <Music2 size={22} strokeWidth={1.8} />
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
          }}
        >
          {item.title}
        </span>
        {isThisPlaying && (
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

      <CirclePlayButton
        isPlaying={isThisPlaying}
        visible={hover || isThisPlaying}
        onClick={(e) => handlePlay(e)}
        size={42}
        iconSize={17}
        style={{ marginRight: 12 }}
        ariaLabel={isThisPlaying ? `Pause ${item.title}` : `Play ${item.title}`}
      />
    </div>
  );
}

function QuickActionsShelf() {
  const containerRef = useRef<HTMLDivElement>(null);
  useReflowPulse();

  const [containerWidth, setContainerWidth] = useState<number>(() => {
    if (typeof window !== "undefined") {
      return Math.max(320, window.innerWidth - 320);
    }
    return 800;
  });

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const measure = () => {
      const w = el.getBoundingClientRect().width;
      if (w > 0) setContainerWidth(w);
    };
    measure();
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0) setContainerWidth(entry.contentRect.width);
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const cols = containerWidth >= 680 ? 3 : containerWidth >= 400 ? 2 : 1;

  const { data: recentTracks = [] } = useRecentlyPlayed();
  const { data: topTracks = [] } = useTopTracks("short_term");
  const { data: topArtists = [] } = useTopArtists("short_term");
  const { data: newReleases = [] } = useNewReleases();

  const items = useMemo<QuickItem[]>(() => {
    const result: QuickItem[] = [
      {
        id: "liked-songs",
        title: "Liked Songs",
        to: "/library?tab=songs",
        isLikedSongs: true,
      },
    ];

    const seen = new Set<string>(["liked-songs"]);

    // 1. Collect from recently played tracks (favoring distinct albums/tracks)
    for (const track of recentTracks) {
      if (result.length >= 6) break;
      const key = track.album?.id ? `album-${track.album.id}` : `track-${track.id}`;
      if (!seen.has(key)) {
        seen.add(key);
        result.push({
          id: key,
          title: track.album?.name || track.name,
          imageUrl: track.album?.image_url,
          to: track.album?.id ? `/album/${track.album.id}` : "/library?tab=songs",
          track: track,
          albumId: track.album?.id,
        });
      }
    }

    // 2. Weave in top artists
    for (const artist of topArtists) {
      if (result.length >= 6) break;
      const key = `artist-${artist.id}`;
      if (!seen.has(key)) {
        seen.add(key);
        result.push({
          id: key,
          title: artist.name,
          imageUrl: artist.image_url,
          to: `/artist/${artist.id}`,
          artistId: artist.id,
        });
      }
    }

    // 3. Fallbacks from topTracks
    for (const track of topTracks) {
      if (result.length >= 6) break;
      const key = `track-${track.id}`;
      if (!seen.has(key)) {
        seen.add(key);
        result.push({
          id: key,
          title: track.name,
          imageUrl: track.album?.image_url,
          to: track.album?.id ? `/album/${track.album.id}` : "/library?tab=songs",
          track: track,
          albumId: track.album?.id,
        });
      }
    }

    // 4. Fallbacks from new releases
    for (const album of newReleases) {
      if (result.length >= 6) break;
      const key = `album-${album.id}`;
      if (!seen.has(key)) {
        seen.add(key);
        result.push({
          id: key,
          title: album.name,
          imageUrl: album.image_url,
          to: `/album/${album.id}`,
          albumId: album.id,
        });
      }
    }

    // 5. Default entries if history is completely empty
    const defaults: QuickItem[] = [
      { id: "def-daily", title: "Daily Mix 1", to: "/playlists" },
      { id: "def-top", title: "Top Hits", to: "/library?tab=songs" },
      { id: "def-disc", title: "Discover Weekly", to: "/playlists" },
      { id: "def-chill", title: "Chill Mix", to: "/playlists" },
      { id: "def-release", title: "Release Radar", to: "/playlists" },
    ];

    for (const def of defaults) {
      if (result.length >= 6) break;
      result.push(def);
    }

    return result.slice(0, 6);
  }, [recentTracks, topArtists, topTracks, newReleases]);

  return (
    <motion.div
      ref={containerRef}
      layout="position"
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gap: "clamp(8px, 1.2vw, 12px)",
        width: "100%",
      }}
    >
      {items.map((item, i) => (
        <motion.div
          key={item.id}
          layout="position"
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ layout: REFLOW, duration: 0.28, delay: i * 0.035 }}
        >
          <QuickActionCard item={item} recentTracks={recentTracks} />
        </motion.div>
      ))}
    </motion.div>
  );
}

// ─── section scaffolding ─────────────────────────────────────────────────────

function SectionTitle({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12, margin: "0 0 14px" }}>
      <h2 style={{ margin: 0, fontSize: "clamp(17px, 2.2vw, 21px)", fontWeight: 700, letterSpacing: "-0.02em", color: "var(--color-text-hi)", textWrap: "balance" } as React.CSSProperties}>
        {children}
      </h2>
      {right}
    </div>
  );
}

function TileSkeleton() {
  return <EvenGridSkeleton minColWidth={140} gap={14} maxRows={1} />;
}

// ─── recommendation / track tile ─────────────────────────────────────────────

function RecTile({ track, onPlay }: { track: TrackItem; onPlay: () => void }) {
  useReflowPulse();
  const [hover, setHover] = useState(false);
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const setPlaying = usePlayerStore((s) => s.setPlaying);
  const isThisTrackPlaying = Boolean(currentTrack?.id === track.id && isPlaying);
  const art = track.album?.image_url;

  return (
    <motion.button
      layout="position"
      transition={{ layout: REFLOW }}
      onClick={onPlay}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileTap={{ scale: 0.97 }}
      style={{
        width: "100%", display: "flex", flexDirection: "column", gap: 10,
        padding: 8, borderRadius: 14, border: "none", background: hover ? "var(--color-surface)" : "transparent",
        cursor: "pointer", textAlign: "left", transition: "background 0.18s ease",
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
              pausePlayback().then(() => setPlaying(false)).catch(() => {});
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
      <div style={{ display: "flex", alignItems: "center", gap: 6, maxWidth: "100%" }}>
        <span style={{ fontSize: 13, fontWeight: 600, color: isThisTrackPlaying ? "var(--color-accent)" : "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
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
      <span style={{ fontSize: 12, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%", marginTop: -4 }}>
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
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.34, delay: Math.min(i, 11) * 0.035, ease: [0.23, 1, 0.32, 1] }}
        >
          <RecTile track={t} onPlay={() => play(i)} />
        </motion.div>
      )}
    />
  );
}

function MadeForYou() {
  const { data: recs = [], isLoading } = useQuery({
    queryKey:  ["recommendations", "home"],
    queryFn:   () => getRecommendations(undefined, 12),
    staleTime: 30 * 60_000,
    gcTime:    24 * 60 * 60_000,
    refetchOnWindowFocus: false,
  });

  return (
    <section>
      <SectionTitle>Made for you</SectionTitle>
      {isLoading ? <TileSkeleton />
        : recs.length === 0 ? (
          <p style={{ margin: 0, fontSize: 13, color: "var(--color-text-dim)" }}>
            Play and follow some artists, recommendations will grow here.
          </p>
        ) : <TrackTiles tracks={recs} context="made-for-you" />}
    </section>
  );
}

function RecentlyPlayed() {
  const { data = [], isLoading } = useRecentlyPlayed();
  if (!isLoading && data.length === 0) return null;
  return (
    <section>
      <SectionTitle>Jump back in</SectionTitle>
      {isLoading ? <TileSkeleton /> : <TrackTiles tracks={data.slice(0, 12)} context="recently-played" />}
    </section>
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
      renderItem={(a, i) => (
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.34, delay: Math.min(i, 11) * 0.035, ease: [0.23, 1, 0.32, 1] }}
        >
          <ArtistCard artist={a} />
        </motion.div>
      )}
    />
  );
}

function TopTracks() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading } = useTopTracks(range);
  const probe = useTopTracks("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <section>
      <SectionTitle right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-tracks-range" />}>Your top tracks</SectionTitle>
      {isLoading ? <TileSkeleton />
        : data.length === 0 ? <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
        : <TrackTiles tracks={data.slice(0, 12)} context={`top-tracks-${range}`} />}
    </section>
  );
}

function TopArtists() {
  const [range, setRange] = useState<TimeRange>("medium_term");
  const { data = [], isLoading } = useTopArtists(range);
  const probe = useTopArtists("medium_term");
  if (!probe.isLoading && (probe.data?.length ?? 0) === 0) return null;
  return (
    <section>
      <SectionTitle right={<RangeSlider value={range} onChange={setRange} layoutId="home-top-artists-range" />}>Your top artists</SectionTitle>
      {isLoading ? <TileSkeleton />
        : data.length === 0 ? <EmptyHint>Not enough listening from {rangeWord(range)} yet.</EmptyHint>
        : <ArtistTiles artists={data.slice(0, 12)} />}
    </section>
  );
}

function NewReleases() {
  const { data = [], isLoading } = useNewReleases();
  if (!isLoading && data.length === 0) return null;
  return (
    <section>
      <SectionTitle>New releases</SectionTitle>
      {isLoading ? (
        <TileSkeleton />
      ) : (
        <EvenGrid
          items={data.slice(0, 12)}
          minColWidth={140}
          gap={14}
          maxRows={2}
          getKey={(al) => al.id}
          renderItem={(al) => <AlbumCard album={al} />}
        />
      )}
    </section>
  );
}

function greeting() {
  const h = new Date().getHours();
  return h < 12 ? "Good morning" : h < 18 ? "Good afternoon" : "Good evening";
}

export default function Home() {
  const { loggedIn, displayName, isLoading, login, loggingIn } = useAuth();
  const hello = greeting();
  useReflowPulse();

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
              {loggingIn ? "Waiting for browser…" : "Log in with Spotify"}
            </button>
          }
          hint={loggingIn ? "Finish signing in in your browser. The app is listening on port 8989." : undefined}
        />
      </div>
    );
  }

  return (
    <motion.div layout="position" style={{ display: "flex", flexDirection: "column", gap: "clamp(26px, 3.4vw, 38px)" }}>
      <motion.div layout="position">
        <p style={{ margin: "0 0 4px", fontSize: 13, fontWeight: 600, letterSpacing: "0.02em", color: "var(--color-text-dim)" }}>{hello}</p>
        <h1 style={{ margin: 0, fontSize: "clamp(26px, 3.8vw, 34px)", fontWeight: 800, letterSpacing: "-0.03em", color: "var(--color-text-hi)", textWrap: "balance" } as React.CSSProperties}>
          {displayName ? displayName.split(" ")[0] : "Welcome back"}
        </h1>
      </motion.div>

      {/* top 6 quick action shelf */}
      <QuickActionsShelf />

      <motion.div layout="position"><MadeForYou /></motion.div>
      <motion.div layout="position"><RecentlyPlayed /></motion.div>
      <motion.div layout="position"><TopTracks /></motion.div>
      <motion.div layout="position"><TopArtists /></motion.div>
      <motion.div layout="position"><NewReleases /></motion.div>
    </motion.div>
  );
}
