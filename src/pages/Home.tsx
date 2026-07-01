import { useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import {
  Music, Play,
  Heart, Disc3, Users, ListMusic, Radio, Search,
  type LucideIcon,
} from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { getRecommendations } from "../api/spotify";
import {
  useRecentlyPlayed, useTopTracks, useTopArtists, useNewReleases,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";
import { Loader } from "../components/ui/Loader";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { TrackItem, ArtistItem } from "../types/spotify";
import type { TimeRange } from "../types/library";

const TILE_MIN = 158;

// grid reflow spring - tiles glide to new columns (lyrics panel en / resize) instead of snapping. positiononly so artwork never squishes midmove.
const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };

function MadeForYou() {
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext     = useQueueStore((s) => s.playContext);
  const { data: recs = [], isLoading } = useQuery({
    queryKey:  ["recommendations", "home"],
    queryFn:   () => getRecommendations(undefined, 12),
    /* recs are expensive and barely change - cache hard so revisiting Home is
    instant, and keep them around a day so a webview reload reuses them
    instead of refetching */
    staleTime: 30 * 60_000,
    gcTime:    24 * 60 * 60_000,
    refetchOnWindowFocus: false,
  });

  function play(i: number) {
    const start = playContext(recs, i, "made-for-you");
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  const grid: React.CSSProperties = {
    display: "grid",
    gap: "clamp(10px, 1.4vw, 14px)",
    gridTemplateColumns: `repeat(auto-fit, minmax(clamp(118px, 15vw, ${TILE_MIN}px), 1fr))`,
  };

  return (
    <section>
      <h2 style={{ margin: "0 0 16px", fontSize: "clamp(17px, 2.2vw, 20px)", fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Made for you</h2>
      {isLoading ? (
        <div style={grid}>
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={i} style={{ aspectRatio: "0.82", borderRadius: 12, background: "var(--color-surface)", border: "1px solid var(--color-border)" }} />
          ))}
        </div>
      ) : recs.length === 0 ? (
        <p style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
          Play and follow some artists, then recommendations will appear here.
        </p>
      ) : (
        <div style={grid}>
          {recs.map((t: TrackItem, i: number) => (
            <motion.div
              key={t.id}
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.34, delay: Math.min(i, 11) * 0.035, ease: [0.23, 1, 0.32, 1] }}
            >
              <RecTile track={t} onPlay={() => play(i)} />
            </motion.div>
          ))}
        </div>
      )}
    </section>
  );
}

function RecTile({ track, onPlay }: { track: TrackItem; onPlay: () => void }) {
  useReflowPulse(); // re-render on resize / panel toggle so layout glides the move
  return (
    <motion.button
      layout="position"
      transition={{ layout: REFLOW }}
      onClick={onPlay}
      className="group"
      style={{
        width: "100%", display: "flex", flexDirection: "column", gap: 10, padding: 12,
        borderRadius: 12, border: "none", background: "var(--color-surface)",
        cursor: "pointer", textAlign: "left", transition: "background 0.12s",
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface)"; }}
    >
      <div style={{ position: "relative", width: "100%", aspectRatio: "1 / 1" }}>
        {track.album?.image_url ? (
          <img src={track.album.image_url} alt="" style={{ width: "100%", height: "100%", borderRadius: 8, objectFit: "cover" }} />
        ) : (
          <div style={{ width: "100%", height: "100%", borderRadius: 8, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center" }}>
            <Music size={36} style={{ color: "var(--color-text-dim)" }} />
          </div>
        )}
        <span
          className="rec-play"
          style={{
            position: "absolute", right: 8, bottom: 8, width: 40, height: 40, borderRadius: "50%",
            background: "var(--color-accent)", color: "#fff", display: "flex", alignItems: "center",
            justifyContent: "center", boxShadow: "0 6px 16px rgba(0,0,0,0.4)",
            opacity: 0, transform: "translateY(6px)", transition: "opacity 0.15s, transform 0.15s",
          }}
        >
          <Play size={18} fill="currentColor" strokeWidth={0} style={{ marginLeft: 2 }} />
        </span>
      </div>
      <span style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%" }}>{track.name}</span>
      <span style={{ fontSize: 12, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "100%" }}>
        {track.artists.map((a) => a.name).join(", ")}
      </span>
    </motion.button>
  );
}

// shared section scaffolding
const tileGrid: React.CSSProperties = {
  display: "grid",
  gap: "clamp(10px, 1.4vw, 14px)",
  gridTemplateColumns: `repeat(auto-fit, minmax(clamp(118px, 15vw, ${TILE_MIN}px), 1fr))`,
};

function SectionTitle({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", margin: "0 0 16px" }}>
      <h2 style={{ margin: 0, fontSize: "clamp(17px, 2.2vw, 20px)", fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>{children}</h2>
      {right}
    </div>
  );
}

function TileSkeleton() {
  return (
    <div style={tileGrid}>
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} style={{ aspectRatio: "0.82", borderRadius: 12, background: "var(--color-surface)", border: "1px solid var(--color-border)" }} />
      ))}
    </div>
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
    <div style={tileGrid}>
      {tracks.map((t, i) => (
        <motion.div
          key={t.id}
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.34, delay: Math.min(i, 11) * 0.035, ease: [0.23, 1, 0.32, 1] }}
        >
          <RecTile track={t} onPlay={() => play(i)} />
        </motion.div>
      ))}
    </div>
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

// animated artist tiles, same stagger as the track tiles
function ArtistTiles({ artists }: { artists: ArtistItem[] }) {
  return (
    <ArtistGrid>
      {artists.map((a, i) => (
        <motion.div
          key={a.id}
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.34, delay: Math.min(i, 11) * 0.035, ease: [0.23, 1, 0.32, 1] }}
        >
          <ArtistCard artist={a} />
        </motion.div>
      ))}
    </ArtistGrid>
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
      {isLoading ? <TileSkeleton /> : (
        <AlbumGrid>
          {data.slice(0, 12).map((al) => <AlbumCard key={al.id} album={al} />)}
        </AlbumGrid>
      )}
    </section>
  );
}


const QUICK_ITEMS: { name: string; to: string; icon: LucideIcon; hue: string }[] = [
  { name: "Liked Songs", to: "/library?tab=songs",   icon: Heart,     hue: "#fa2d48" },
  { name: "Albums",      to: "/library?tab=albums",  icon: Disc3,     hue: "#8b5cf6" },
  { name: "Artists",     to: "/library?tab=artists", icon: Users,     hue: "#0ea5e9" },
  { name: "Playlists",   to: "/playlists",           icon: ListMusic, hue: "#10b981" },
  { name: "Radio",       to: "/radio",               icon: Radio,     hue: "#f59e0b" },
  { name: "Search",      to: "/search",              icon: Search,    hue: "#ec4899" },
];

function QuickItem({ name, to, icon: Icon, hue }: (typeof QUICK_ITEMS)[number]) {
  return (
    <Link
      to={to}
      style={{
        display:      "flex",
        alignItems:   "center",
        gap:          12,
        height:       56,
        borderRadius: 10,
        overflow:     "hidden",
        background:   "var(--color-surface)",
        border:       "1px solid var(--color-border)",
        textDecoration: "none",
        color:        "var(--color-text-hi)",
        fontSize:     13.5,
        fontWeight:   600,
        letterSpacing: "-0.01em",
        transition:   "background 0.12s",
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "var(--color-surface-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "var(--color-surface)"; }}
    >
      <div
        style={{
          width: 56, height: 56, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center",
          // soft tinted tile in the items hue
          background: `linear-gradient(135deg, ${hue}33, ${hue}14)`,
          borderRight: `1px solid ${hue}22`,
        }}
      >
        <Icon size={20} strokeWidth={2.2} style={{ color: hue }} fill={Icon === Heart ? "currentColor" : "none"} />
      </div>
      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", paddingRight: 12 }}>{name}</span>
    </Link>
  );
}

function greeting() {
  const h = new Date().getHours();
  return h < 12 ? "Good morning" : h < 18 ? "Good afternoon" : "Good evening";
}

export default function Home() {
  const { loggedIn, displayName, isLoading, login, loggingIn } = useAuth();
  const hello = greeting();

  if (isLoading) {
    return (
      <div>
        <h1 style={{ margin: "0 0 16px", fontSize: 30, fontWeight: 700, letterSpacing: "-0.015em", color: "var(--color-text-hi)" }}>{hello}</h1>
        <Loader fill={false} />
      </div>
    );
  }

  if (!loggedIn) {
    return (
      <div style={{ maxWidth: 520 }}>
        <h1 style={{ margin: "0 0 8px", fontSize: 30, fontWeight: 700, letterSpacing: "-0.015em", color: "var(--color-text-hi)" }}>{hello}</h1>
        <p style={{ margin: "0 0 24px", fontSize: 14, color: "var(--color-text-dim)" }}>Connect your Spotify account to get started.</p>
        <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
          <button
            onClick={() => login()}
            disabled={loggingIn}
            style={{
              height: 44, padding: "0 26px", borderRadius: 99, border: "none",
              background: "var(--color-accent)", color: "var(--color-accent-text)",
              fontSize: 14, fontWeight: 600, cursor: loggingIn ? "default" : "pointer", opacity: loggingIn ? 0.5 : 1,
            }}
          >
            {loggingIn ? "Waiting for browser…" : "Login with Spotify"}
          </button>
          <Link
            to="/settings"
            style={{
              height: 44, padding: "0 26px", borderRadius: 99,
              display: "inline-flex", alignItems: "center",
              background: "var(--color-surface)", border: "1px solid var(--color-border)",
              color: "var(--color-text-hi)", fontSize: 14, fontWeight: 600, textDecoration: "none",
            }}
          >
            Settings
          </Link>
        </div>
        {loggingIn && (
          <p style={{ margin: "16px 0 0", fontSize: 12, color: "var(--color-text-dim)" }}>
            Complete login in your browser — the app listens on port 8888 for the callback.
          </p>
        )}
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(22px, 3vw, 32px)" }}>
      <h1 style={{ margin: 0, fontSize: "clamp(22px, 3.4vw, 30px)", fontWeight: 700, letterSpacing: "-0.015em", color: "var(--color-text-hi)" }}>
        {displayName ? `${hello}, ${displayName.split(" ")[0]}` : hello}
      </h1>

      {/* quick access grid - auto-fit so tiles stretch to fill the row evenly
          at any width instead of leaving a ragged gap at the end */}
      <div style={{ display: "grid", gap: "clamp(8px, 1vw, 10px)", gridTemplateColumns: "repeat(auto-fit, minmax(clamp(150px, 18vw, 200px), 1fr))" }}>
        {QUICK_ITEMS.map((item, i) => (
          <motion.div
            key={item.name}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: i * 0.04, ease: [0.23, 1, 0.32, 1] }}
          >
            <QuickItem {...item} />
          </motion.div>
        ))}
      </div>

      {/* made for you - recommendation radio */}
      <MadeForYou />

      {/* recently played - instant from local cache */}
      <RecentlyPlayed />

      {/* top tracks / artists, with a time-range toggle */}
      <TopTracks />
      <TopArtists />

      {/* new releases - personalized off your artists */}
      <NewReleases />
    </div>
  );
}
