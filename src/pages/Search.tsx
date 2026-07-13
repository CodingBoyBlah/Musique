import { useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { motion } from "framer-motion";
import { useSearch } from "../hooks/useSearch";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { useAuthStore } from "../store/auth.store";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { CoverArt } from "../components/ui/CoverArt";
import { TrackRow } from "../components/ui/TrackRow";
import type { PlaylistCard as PlaylistCardType } from "../types/spotify";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { meshGradient } from "../lib/mesh";
import { errMsg } from "../lib/err";

const CATEGORIES = ["all", "songs", "artists", "albums", "playlists"] as const;
type Category = (typeof CATEGORIES)[number];


const REFLOW = { type: "spring" as const, stiffness: 520, damping: 44 };
const MotionLink = motion.create(Link);

function PlaylistResultCard({ playlist }: { playlist: PlaylistCardType }) {
  const [hover, setHover] = useState(false);
  useReflowPulse(); 
  return (
    <MotionLink
      to={`/playlist/${playlist.id}`}
      layout="position"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ layout: REFLOW }}
      style={{
        display: "flex", flexDirection: "column", gap: 8,
        padding: 14, borderRadius: 14, width: "100%", boxSizing: "border-box",
        textDecoration: "none", color: "inherit",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
      }}
    >
      <div style={{ width: "100%", aspectRatio: "1 / 1" }}>
        <CoverArt url={playlist.image_url} alt={playlist.name} size={136} style={{ width: "100%", height: "100%" }} />
      </div>
      <span className="text-sm font-medium line-clamp-2" style={{ maxWidth: "100%" }}>{playlist.name}</span>
      <span className="text-xs line-clamp-1" style={{ color: "var(--color-text-dim)" }}>
        {playlist.owner_name ? `By ${playlist.owner_name}` : "Playlist"}
      </span>
    </MotionLink>
  );
}



const BROWSE: string[] = [
  "Pop", "Hip-Hop", "Rock", "R&B", "Indie", "Electronic",
  "Chill", "Focus", "Workout", "Party", "Jazz", "Classical",
  "Lo-fi", "Metal", "K-Pop", "Country", "Soul", "Acoustic",
];

function BrowseCard({ label }: { label: string }) {
  const [hover, setHover] = useState(false);
  return (
    <MotionLink
      to={`/search?q=${encodeURIComponent(label)}`}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        position: "relative",
        display: "flex",
        alignItems: "flex-end",
        aspectRatio: "4 / 3",
        padding: 14,
        borderRadius: 14,
        overflow: "hidden",
        textDecoration: "none",
        color: "#fff",
        ...meshGradient(label),
        outline: "1px solid rgba(255,255,255,0.08)",
        outlineOffset: -1,
        transform: hover ? "translateY(-3px)" : "translateY(0)",
        boxShadow: hover ? "0 16px 34px rgba(0,0,0,0.4)" : "0 4px 14px rgba(0,0,0,0.26)",
        transition: "transform 0.22s cubic-bezier(0.23,1,0.32,1), box-shadow 0.22s ease",
      }}
    >
      <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(0,0,0,0.02), rgba(0,0,0,0.34))" }} />
      <span style={{ position: "relative", fontSize: "clamp(15px, 1.7vw, 18px)", fontWeight: 700, letterSpacing: "-0.02em", textShadow: "0 1px 10px rgba(0,0,0,0.4)" }}>
        {label}
      </span>
    </MotionLink>
  );
}

function BrowseHome() {
  return (
    <motion.div layout="position" className="flex flex-col gap-6">
      <motion.div layout="position">
        <h1 style={{ margin: 0, fontSize: 26, fontWeight: 800, letterSpacing: "-0.02em", color: "var(--color-text-hi)" }}>Search</h1>
        <p style={{ margin: "6px 0 0", fontSize: 13.5, color: "var(--color-text-dim)" }}>
          Find any song, artist, or album, or dive into a mood below.
        </p>
      </motion.div>

      <motion.section layout="position">
        <h2 style={{ margin: "0 0 14px", fontSize: 17, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Browse all</h2>
        <div style={{ display: "grid", gap: "clamp(10px, 1.4vw, 16px)", gridTemplateColumns: "repeat(auto-fill, minmax(clamp(150px, 18vw, 200px), 1fr))" }}>
          {BROWSE.map((label, i) => (
            <motion.div
              key={label}
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: Math.min(i, 12) * 0.03, ease: [0.23, 1, 0.32, 1] }}
            >
              <BrowseCard label={label} />
            </motion.div>
          ))}
        </div>
      </motion.section>
    </motion.div>
  );
}

export default function Search() {
  useReflowPulse();
  const loggedIn        = useAuthStore((s) => s.loggedIn);
  const [params]        = useSearchParams();
  const query           = (params.get("q") ?? "").trim();
  const [cat, setCat]   = useState<Category>("all");
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const toggleLike      = useToggleLike();

  const { data, isLoading, error } = useSearch(query);
  const trackIds = data?.tracks.map((t) => t.id) ?? [];
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = new Set(savedIds);

  if (!loggedIn) {
    return (
      <div>
        <h1 style={{ margin: "0 0 12px", fontSize: 26, fontWeight: 700, color: "var(--color-text-hi)" }}>Search</h1>
        <p style={{ color: "var(--color-text-dim)" }}>
          <Link to="/" style={{ color: "var(--color-accent)", textDecoration: "none" }}>Login</Link>{" "}
          to search Spotify.
        </p>
      </div>
    );
  }

  if (!query) return <BrowseHome />;

  const hasResults = data && (data.tracks.length || data.artists.length || data.albums.length || data.playlists.length);
  const show = (c: Category) => cat === "all" || cat === c;

  return (
    <motion.div layout="position" className="flex flex-col gap-6">
      <motion.h1 layout="position" style={{ margin: 0, fontSize: 22, fontWeight: 700, color: "var(--color-text-hi)" }}>
        Results for <span style={{ color: "var(--color-accent)" }}>“{query}”</span>
      </motion.h1>

      {/* category chips */}
      <motion.div layout="position" style={{ display: "flex", gap: 8 }}>
        {CATEGORIES.map((c) => {
          const on = cat === c;
          return (
            <button
              key={c}
              onClick={() => setCat(c)}
              style={{
                padding: "6px 16px", borderRadius: 99, border: "none",
                background: on ? "var(--color-text-hi)" : "var(--color-surface)",
                color: on ? "#0a0a0c" : "var(--color-text)",
                fontSize: 13, fontWeight: 600, cursor: "pointer",
                textTransform: "capitalize", transition: "background 0.12s, color 0.12s",
              }}
            >
              {c}
            </button>
          );
        })}
      </motion.div>

      {isLoading && <motion.p layout="position" style={{ fontSize: 13, color: "var(--color-text-dim)" }}>Searching…</motion.p>}
      {error && <motion.p layout="position" style={{ fontSize: 13, color: "var(--color-danger)" }}>{errMsg(error)}</motion.p>}
      {!isLoading && !error && !hasResults && (
        <motion.p layout="position" style={{ fontSize: 13, color: "var(--color-text-dim)" }}>No results for “{query}”</motion.p>
      )}

      {data && (
        <>
          {show("artists") && data.artists.length > 0 && (
            <motion.section layout="position">
              <h2 className="text-lg font-bold mb-3">Artists</h2>
              <ArtistGrid>
                {data.artists.map((a) => <ArtistCard key={a.id} artist={a} />)}
              </ArtistGrid>
            </motion.section>
          )}

          {show("albums") && data.albums.length > 0 && (
            <motion.section layout="position">
              <h2 className="text-lg font-bold mb-3">Albums</h2>
              <AlbumGrid>
                {data.albums.map((al) => <AlbumCard key={al.id} album={al} />)}
              </AlbumGrid>
            </motion.section>
          )}

          {show("playlists") && data.playlists.length > 0 && (
            <motion.section layout="position">
              <h2 className="text-lg font-bold mb-3">Playlists</h2>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(clamp(118px, 15vw, 160px), 1fr))", gap: "clamp(10px, 1.5vw, 16px)", width: "100%" }}>
                {data.playlists.map((pl) => <PlaylistResultCard key={pl.id} playlist={pl} />)}
              </div>
            </motion.section>
          )}

          {show("songs") && data.tracks.length > 0 && (
            <motion.section layout="position">
              <h2 className="text-lg font-bold mb-3">Songs</h2>
              <div className="flex flex-col">
                {data.tracks.map((t, i) => (
                  <TrackRow
                    key={t.id}
                    track={t}
                    showAlbum
                    liked={likedSet.has(t.id)}
                    onPlay={() => {
                      const start = playContext(data.tracks, i, "search");
                      if (start) { setCurrentTrack(start); playTrack(start.id).catch(console.error); }
                    }}
                    onQueue={(track) => enqueue(track)}
                    onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
                  />
                ))}
              </div>
            </motion.section>
          )}
        </>
      )}
    </motion.div>
  );
}
