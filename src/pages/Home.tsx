import { useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import {
  Play, Heart, Disc3, Users, ListMusic,
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
import { meshGradient } from "../lib/mesh";
import type { TrackItem, ArtistItem } from "../types/spotify";
import type { TimeRange } from "../types/library";

const TILE_MIN = 158;

// grid reflow spring--- tiles glide to new columns (lyrics panel/resize) instead
// of snapping. position only so artwork never squishes mid move
const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };

const tileGrid: React.CSSProperties = {
  display: "grid",
  gap: "clamp(10px, 1.4vw, 16px)",
  gridTemplateColumns: `repeat(auto-fill, minmax(clamp(118px, 15vw, ${TILE_MIN}px), 1fr))`,
};




const SHORTCUTS: { name: string; sub: string; to: string; icon: LucideIcon; seed: string }[] = [
  { name: "Liked Songs", sub: "Everything you've saved", to: "/library?tab=songs",   icon: Heart,     seed: "liked-songs" },
  { name: "Albums",      sub: "Your saved records",       to: "/library?tab=albums",  icon: Disc3,     seed: "albums-shelf" },
  { name: "Artists",     sub: "People you follow",        to: "/library?tab=artists", icon: Users,     seed: "artists-shelf" },
  { name: "Playlists",   sub: "Made and collected",       to: "/playlists",           icon: ListMusic, seed: "playlists-shelf" },
];

function ShortcutCard({ name, sub, to, icon: Icon, seed }: (typeof SHORTCUTS)[number]) {
  const [hover, setHover] = useState(false);
  return (
    <Link
      to={to}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        justifyContent: "flex-end",
        alignItems: "center",
        aspectRatio: "1 / 1",
        padding: 16,
        borderRadius: 18,
        overflow: "hidden",
        textDecoration: "none",
        color: "#fff",
        ...meshGradient(seed),
        outline: "1px solid rgba(255,255,255,0.08)",
        outlineOffset: -1,
        transform: hover ? "translateY(-3px)" : "translateY(0)",
        boxShadow: hover
          ? "0 18px 40px rgba(0,0,0,0.42)"
          : "0 6px 18px rgba(0,0,0,0.28)",
        transition: "transform 0.24s cubic-bezier(0.23,1,0.32,1), box-shadow 0.24s ease",
      }}
    >
      {/* legibility wash so the title reads on any mesh */}
      <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(0,0,0,0.05) 30%, rgba(0,0,0,0.42) 100%)" }} />
      {/* frosted icon badge, top-right (the "Music" chip in the reference) */}
      <div
        style={{
          position: "absolute", top: 12, right: 12,
          width: 30, height: 30, borderRadius: "50%",
          display: "flex", alignItems: "center", justifyContent: "center",
          background: "rgba(255,255,255,0.18)",
          backdropFilter: "blur(10px)", WebkitBackdropFilter: "blur(10px)",
          border: "1px solid rgba(255,255,255,0.22)",
        }}
      >
        <Icon size={15} strokeWidth={2.2} fill={Icon === Heart ? "currentColor" : "none"} style={{ color: "#fff" }} />
      </div>
      <div style={{ position: "relative", display: "flex", flexDirection: "column", gap: 2, alignItems: "center" }}>
        <span style={{ fontSize: "clamp(16px, 1.8vw, 19px)", fontWeight: 700, letterSpacing: "-0.02em", textShadow: "0 1px 12px rgba(0,0,0,0.35)", textAlign: "center" }}>
          {name}
        </span>
      </div>
    </Link>
  );
}




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
  return (
    <div style={tileGrid}>
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} style={{ aspectRatio: "1 / 1", borderRadius: 12, background: "var(--color-surface)" }} />
      ))}
    </div>
  );
}




function RecTile({ track, onPlay }: { track: TrackItem; onPlay: () => void }) {
  useReflowPulse(); // rerender on resize/panel toggle so layout glides the move
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

// row of play on click track tiles that all share one playback context
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
              Finish signing in in your browser, the app is listening on port 8888.
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

      
      <motion.div layout style={{ display: "grid", gap: "clamp(10px, 1.4vw, 16px)", gridTemplateColumns: "repeat(auto-fill, minmax(clamp(200px, 24vw, 280px), 1fr))" }}>
        {SHORTCUTS.map((item, i) => (
          <motion.div
            key={item.name}
            layout
            initial={reduceMotion ? false : { opacity: 0, y: 12, scale: 0.96 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.34, delay: i * 0.05, ease: [0.23, 1, 0.32, 1] }}
          >
            <ShortcutCard {...item} />
          </motion.div>
        ))}
      </motion.div>

      <motion.div layout="position"><MadeForYou /></motion.div>
      <motion.div layout="position"><RecentlyPlayed /></motion.div>
      <motion.div layout="position"><TopTracks /></motion.div>
      <motion.div layout="position"><TopArtists /></motion.div>
      <motion.div layout="position"><NewReleases /></motion.div>
    </motion.div>
  );
}
