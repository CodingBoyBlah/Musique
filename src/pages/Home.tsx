import { useState, useRef, useLayoutEffect, useMemo } from "react";
import { Link, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { Play, Heart, Music2 } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { getRecommendations } from "../api/spotify";
import {
  useRecentlyPlayed,
  useTopTracks,
  useTopArtists,
  useNewReleases,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";
import { Loader } from "../components/ui/Loader";
import { ArtistCard } from "../components/ui/ArtistCard";
import { AlbumCard } from "../components/ui/AlbumCard";
import { EvenGrid, EvenGridSkeleton } from "../components/ui/EvenGrid";
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
  trackId?: string;
  isLikedSongs?: boolean;
}

function QuickActionCard({ item }: { item: QuickItem }) {
  const [hover, setHover] = useState(false);
  const navigate = useNavigate();

  function handlePlay(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (item.trackId) {
      playTrack(item.trackId).catch(() => {});
    } else if (item.isLikedSongs) {
      navigate("/library?tab=songs");
    } else {
      navigate(item.to);
    }
  }

  return (
    <Link
      to={item.to}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        display: "flex",
        alignItems: "center",
        height: 56,
        borderRadius: 8,
        overflow: "hidden",
        textDecoration: "none",
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
            width: 56,
            height: 56,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "linear-gradient(135deg, #450af5 0%, #8e8ee5 100%)",
            color: "#ffffff",
          }}
        >
          <Heart size={20} fill="#ffffff" strokeWidth={0} />
        </div>
      ) : item.imageUrl ? (
        <img
          src={item.imageUrl}
          alt=""
          loading="lazy"
          style={{
            width: 56,
            height: 56,
            flexShrink: 0,
            objectFit: "cover",
          }}
        />
      ) : (
        <div
          style={{
            width: 56,
            height: 56,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(255,255,255,0.06)",
            color: "var(--color-text-dim)",
          }}
        >
          <Music2 size={20} strokeWidth={1.8} />
        </div>
      )}

      <div style={{ flex: 1, minWidth: 0, padding: "0 12px" }}>
        <span
          style={{
            display: "block",
            fontSize: 13.5,
            fontWeight: 700,
            letterSpacing: "-0.01em",
            color: "var(--color-text-hi)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {item.title}
        </span>
      </div>

      <motion.button
        aria-label={`Play ${item.title}`}
        onClick={handlePlay}
        initial={false}
        animate={{
          opacity: hover ? 1 : 0,
          scale: hover ? 1 : 0.85,
        }}
        transition={{ duration: 0.16, ease: "easeOut" }}
        style={{
          width: 34,
          height: 34,
          borderRadius: "50%",
          border: "none",
          background: "var(--color-accent)",
          color: "var(--color-accent-text, #fff)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          marginRight: 12,
          flexShrink: 0,
          cursor: "pointer",
          boxShadow: "0 6px 16px rgba(0,0,0,0.38)",
          pointerEvents: hover ? "auto" : "none",
        }}
      >
        <Play size={15} fill="currentColor" strokeWidth={0} style={{ marginLeft: 2 }} />
      </motion.button>
    </Link>
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
          trackId: track.id,
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
          trackId: track.id,
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
          <QuickActionCard item={item} />
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
        <span
          aria-hidden
          style={{
            position: "absolute", right: 9, bottom: 9, width: 42, height: 42, borderRadius: "50%",
            background: "var(--color-accent)", color: "#fff", display: "flex", alignItems: "center",
            justifyContent: "center", boxShadow: "0 8px 20px rgba(0,0,0,0.45)",
            opacity: hover ? 1 : 0, transform: hover ? "translateY(0) scale(1)" : "translateY(8px) scale(0.9)",
            transition: "opacity 0.18s ease, transform 0.22s cubic-bezier(0.23,1,0.32,1)",
          }}
        >
          <Play size={18} fill="currentColor" strokeWidth={0} style={{ marginLeft: 2 }} />
        </span>
      </div>
      <span style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%" }}>{track.name}</span>
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

const RANGES: { key: TimeRange; label: string }[] = [
  { key: "short_term",  label: "4 weeks" },
  { key: "medium_term", label: "6 months" },
  { key: "long_term",   label: "All time" },
];

function RangeToggle({ value, onChange }: { value: TimeRange; onChange: (r: TimeRange) => void }) {
  return (
    <div style={{ display: "flex", gap: 2, padding: 2, borderRadius: 99, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
      {RANGES.map((r) => {
        const active = r.key === value;
        return (
          <button
            key={r.key}
            onClick={() => onChange(r.key)}
            style={{
              height: 28, padding: "0 12px", borderRadius: 99, border: "none", cursor: "pointer",
              fontSize: 12, fontWeight: 600, fontFamily: "inherit",
              background: active ? "var(--color-accent)" : "transparent",
              color: active ? "var(--color-accent-text)" : "var(--color-text-dim)",
              transition: "background 0.12s, color 0.12s",
            }}
          >
            {r.label}
          </button>
        );
      })}
    </div>
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
      <SectionTitle right={<RangeToggle value={range} onChange={setRange} />}>Your top tracks</SectionTitle>
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
      <SectionTitle right={<RangeToggle value={range} onChange={setRange} />}>Your top artists</SectionTitle>
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
      <div
        style={{
          position: "relative", maxWidth: 560, marginTop: "6vh",
          padding: "clamp(28px, 4vw, 44px)", borderRadius: 24, overflow: "hidden",
          color: "#fff", ...meshGradient("welcome-musique"),
          outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1,
        }}
      >
        <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(0,0,0,0.12), rgba(0,0,0,0.5))" }} />
        <div style={{ position: "relative" }}>
          <h1 style={{ margin: "0 0 10px", fontSize: "clamp(28px, 5vw, 40px)", fontWeight: 800, letterSpacing: "-0.03em", textWrap: "balance", textShadow: "0 2px 20px rgba(0,0,0,0.4)" } as React.CSSProperties}>{hello}</h1>
          <p style={{ margin: "0 0 26px", fontSize: 15, color: "rgba(255,255,255,0.86)", maxWidth: 380 }}>Connect your Spotify account to bring your library, playlists, and listening here.</p>
          <button
            onClick={() => login()}
            disabled={loggingIn}
            style={{
              height: 46, padding: "0 28px", borderRadius: 99, border: "none",
              background: "#fff", color: "#0a0a0c",
              fontSize: 14.5, fontWeight: 700, cursor: loggingIn ? "default" : "pointer", opacity: loggingIn ? 0.6 : 1,
              transition: "opacity 0.15s, transform 0.12s",
            }}
          >
            {loggingIn ? "Waiting for browser…" : "Log in with Spotify"}
          </button>
          {loggingIn && (
            <p style={{ margin: "16px 0 0", fontSize: 12, color: "rgba(255,255,255,0.72)" }}>
              Finish signing in in your browser. The app is listening on port 8989.
            </p>
          )}
        </div>
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
