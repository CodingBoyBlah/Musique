import { useState, useMemo } from "react";
import { Link, Navigate, useSearchParams, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useSearch } from "../hooks/useSearch";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { CoverArt } from "../components/ui/CoverArt";
import { TrackRow } from "../components/ui/TrackRow";
import { CirclePlayButton } from "../components/ui/CirclePlayButton";
import { getArtist, getAlbum } from "../api/spotify";
import { playTrack, pausePlayback, resumePlayback } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { errMsg } from "../lib/err";
import type {
  PlaylistCard as PlaylistCardType,
  ArtistItem,
  AlbumItem,
  TrackItem,
  SearchResults,
} from "../types/spotify";

const CATEGORIES = ["all", "songs", "artists", "albums", "playlists"] as const;
type Category = (typeof CATEGORIES)[number];

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };
const MotionLink = motion.create(Link);

// ─── Top Result resolution & card ───────────────────────────────────────────

type TopResult =
  | { type: "artist"; item: ArtistItem }
  | { type: "track"; item: TrackItem }
  | { type: "album"; item: AlbumItem };

function getTopResult(data: SearchResults, query: string): TopResult | null {
  const q = query.trim().toLowerCase();
  if (!q) return null;

  // 1. Exact matches
  const exactArtist = data.artists.find((a) => a.name.toLowerCase() === q);
  if (exactArtist) return { type: "artist", item: exactArtist };

  const exactTrack = data.tracks.find((t) => t.name.toLowerCase() === q);
  if (exactTrack) return { type: "track", item: exactTrack };

  const exactAlbum = data.albums.find((al) => al.name.toLowerCase() === q);
  if (exactAlbum) return { type: "album", item: exactAlbum };

  // 2. Starts with query
  const startsArtist = data.artists.find((a) => a.name.toLowerCase().startsWith(q));
  if (startsArtist) return { type: "artist", item: startsArtist };

  const startsTrack = data.tracks.find((t) => t.name.toLowerCase().startsWith(q));
  if (startsTrack) return { type: "track", item: startsTrack };

  const startsAlbum = data.albums.find((al) => al.name.toLowerCase().startsWith(q));
  if (startsAlbum) return { type: "album", item: startsAlbum };

  // 3. Fallback priority: artist -> track -> album
  if (data.artists.length > 0) return { type: "artist", item: data.artists[0] };
  if (data.tracks.length > 0) return { type: "track", item: data.tracks[0] };
  if (data.albums.length > 0) return { type: "album", item: data.albums[0] };

  return null;
}

function TopResultCard({
  result,
  onPlay,
  isPlaying,
}: {
  result: TopResult;
  onPlay: (e: React.MouseEvent) => void;
  isPlaying: boolean;
}) {
  const [hover, setHover] = useState(false);
  const navigate = useNavigate();

  const isArtist = result.type === "artist";
  const isTrack = result.type === "track";
  const isAlbum = result.type === "album";

  const title = result.item.name;
  const image = isArtist
    ? result.item.image_url
    : isTrack
    ? result.item.album?.image_url
    : result.item.image_url;

  const subtitle = isArtist
    ? "Artist"
    : isTrack
    ? result.item.artists.map((a) => a.name).join(", ")
    : result.item.artists.map((a) => a.name).join(", ");

  const badge = isArtist ? "Artist" : isTrack ? "Song" : "Album";

  const targetPath = isArtist
    ? `/artist/${result.item.id}`
    : isAlbum
    ? `/album/${result.item.id}`
    : result.item.album?.id
    ? `/album/${result.item.album.id}`
    : undefined;

  function handleClick() {
    if (isTrack) {
      onPlay({} as React.MouseEvent);
    } else if (targetPath) {
      navigate(targetPath);
    }
  }

  return (
    <motion.div
      layout="position"
      transition={{ layout: REFLOW }}
      onClick={handleClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -2 }}
      style={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        gap: 16,
        padding: "clamp(16px, 2vw, 22px)",
        borderRadius: 14,
        background: hover
          ? "var(--color-surface-hover, rgba(255,255,255,0.10))"
          : "var(--color-surface, rgba(255,255,255,0.05))",
        border: "1px solid rgba(255, 255, 255, 0.06)",
        boxShadow: hover ? "0 12px 32px rgba(0,0,0,0.38)" : "0 2px 10px rgba(0,0,0,0.18)",
        cursor: "pointer",
        transition: "background 0.18s ease, box-shadow 0.18s ease",
        overflow: "hidden",
        userSelect: "none",
        height: "100%",
        boxSizing: "border-box",
      }}
    >
      <div style={{ display: "flex", alignItems: "flex-start" }}>
        <div
          style={{
            width: "clamp(84px, 9.5vw, 108px)",
            height: "clamp(84px, 9.5vw, 108px)",
            borderRadius: isArtist ? "50%" : 10,
            overflow: "hidden",
            flexShrink: 0,
            boxShadow: isArtist
              ? "0 8px 24px rgba(0,0,0,0.45)"
              : "0 6px 18px rgba(0,0,0,0.35)",
            outline: "1px solid rgba(255,255,255,0.1)",
            outlineOffset: -1,
          }}
        >
          <CoverArt
            url={image ?? null}
            alt={title}
            size={108}
            rounded={isArtist}
            style={{ width: "100%", height: "100%" }}
          />
        </div>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6, minWidth: 0, paddingRight: 58 }}>
        <h3
          className="line-clamp-2"
          style={{
            margin: 0,
            fontSize: "clamp(22px, 2.8vw, 30px)",
            fontWeight: 800,
            letterSpacing: "-0.03em",
            color: "var(--color-text-hi)",
            lineHeight: 1.15,
          }}
        >
          {title}
        </h3>

        <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
          <span
            style={{
              padding: "3px 10px",
              borderRadius: 999,
              background: "rgba(255, 255, 255, 0.12)",
              fontSize: 11.5,
              fontWeight: 700,
              letterSpacing: "0.03em",
              textTransform: "uppercase",
              color: "var(--color-text-hi)",
            }}
          >
            {badge}
          </span>
          {subtitle && subtitle !== badge && (
            <span
              className="line-clamp-1"
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: "var(--color-text-dim)",
              }}
            >
              {subtitle}
            </span>
          )}
        </div>
      </div>

      <div style={{ position: "absolute", right: 18, bottom: 18, zIndex: 2 }}>
        <CirclePlayButton
          isPlaying={isPlaying}
          visible={hover || isPlaying}
          onClick={onPlay}
          size={46}
          iconSize={19}
          ariaLabel={isPlaying ? `Pause ${title}` : `Play ${title}`}
        />
      </div>
    </motion.div>
  );
}

// ─── Playlist card ──────────────────────────────────────────────────────────

function PlaylistResultCard({ playlist }: { playlist: PlaylistCardType }) {
  const [hover, setHover] = useState(false);
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

// ─── Search Page ────────────────────────────────────────────────────────────

export default function Search() {
  const { loggedIn, login, loggingIn } = useAuth();
  const [params]        = useSearchParams();
  const query           = (params.get("q") ?? "").trim();
  const [cat, setCat]   = useState<Category>("all");
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const isPlaying       = usePlayerStore((s) => s.isPlaying);
  const setPlaying      = usePlayerStore((s) => s.setPlaying);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const toggleLike      = useToggleLike();

  const { data, isLoading, error } = useSearch(query);
  const trackIds = data?.tracks.map((t) => t.id) ?? [];
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = new Set(savedIds);

  const topResult = useMemo(() => {
    if (!data) return null;
    return getTopResult(data, query);
  }, [data, query]);

  const isTopResultPlaying = Boolean(
    isPlaying &&
      topResult &&
      ((topResult.type === "track" && currentTrack?.id === topResult.item.id) ||
        (topResult.type === "artist" && currentTrack?.artists.some((a) => a.id === topResult.item.id)) ||
        (topResult.type === "album" && currentTrack?.album?.id === topResult.item.id))
  );

  async function handlePlayTopResult(e?: React.MouseEvent) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (!topResult) return;

    if (isTopResultPlaying) {
      pausePlayback().then(() => setPlaying(false)).catch(() => {});
      return;
    }

    if (topResult.type === "track") {
      if (currentTrack?.id === topResult.item.id && !isPlaying) {
        resumePlayback().then(() => setPlaying(true)).catch(() => {});
        return;
      }
      const idx = data?.tracks.findIndex((t) => t.id === topResult.item.id) ?? -1;
      const start = playContext(data?.tracks ?? [topResult.item], idx >= 0 ? idx : 0, "search");
      const trackToPlay = start || topResult.item;
      setCurrentTrack(trackToPlay);
      playTrack(trackToPlay.id).then(() => setPlaying(true)).catch(console.error);
      return;
    }

    if (topResult.type === "artist") {
      try {
        const full = await getArtist(topResult.item.id);
        const tracks = full?.top_tracks ?? [];
        if (tracks.length > 0) {
          const start = playContext(tracks, 0, topResult.item.id);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(console.error);
          }
        }
      } catch (err) {
        console.error(err);
      }
      return;
    }

    if (topResult.type === "album") {
      try {
        const full = await getAlbum(topResult.item.id);
        const tracks = full?.tracks ?? [];
        if (tracks.length > 0) {
          const start = playContext(tracks, 0, topResult.item.id);
          if (start) {
            setCurrentTrack(start);
            playTrack(start.id).then(() => setPlaying(true)).catch(console.error);
          }
        }
      } catch (err) {
        console.error(err);
      }
      return;
    }
  }

  if (!loggedIn) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
        <h1 style={{ margin: 0, fontSize: 26, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>
          Search
        </h1>
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
          title="Sign in to search Spotify"
          description="Log in with your Spotify account to search for songs, albums, and artists."
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
                color: "#fff",
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

  if (!query) return <Navigate to="/" replace />;

  const hasResults = Boolean(
    data &&
      (data.tracks.length > 0 ||
        data.artists.length > 0 ||
        data.albums.length > 0 ||
        data.playlists.length > 0)
  );
  const show = (c: Category) => cat === "all" || cat === c;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "clamp(24px, 3.2vw, 36px)",
        width: "100%",
        maxWidth: "100%",
        boxSizing: "border-box",
      }}
    >
      {/* Search query header */}
      <motion.div
        layout="position"
        transition={{ layout: REFLOW }}
        style={{ display: "flex", flexDirection: "column", gap: 4 }}
      >
        <p
          style={{
            margin: 0,
            fontSize: 13,
            fontWeight: 600,
            letterSpacing: "0.02em",
            color: "var(--color-text-dim)",
          }}
        >
          Search results
        </p>
        <h1
          style={{
            margin: 0,
            fontSize: "clamp(22px, 3.2vw, 32px)",
            fontWeight: 800,
            letterSpacing: "-0.03em",
            color: "var(--color-text-hi)",
            textWrap: "balance",
          } as React.CSSProperties}
        >
          Results for <span style={{ color: "var(--color-accent)" }}>“{query}”</span>
        </h1>
      </motion.div>

      {/* category filter chips */}
      <motion.div
        layout="position"
        transition={{ layout: REFLOW }}
        role="tablist"
        aria-label="Filter categories"
        style={{
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          gap: "clamp(6px, 1vw, 10px)",
        }}
      >
        {CATEGORIES.map((c) => {
          const on = cat === c;
          return (
            <motion.button
              key={c}
              layout="position"
              transition={{ layout: REFLOW }}
              role="tab"
              aria-selected={on}
              whileHover={{ scale: 1.04 }}
              whileTap={{ scale: 0.96 }}
              onClick={() => setCat(c)}
              style={{
                padding: "7px 18px",
                borderRadius: 999,
                border: on
                  ? "1px solid rgba(255, 255, 255, 0.15)"
                  : "1px solid rgba(255, 255, 255, 0.08)",
                background: on
                  ? "var(--color-text-hi, #ffffff)"
                  : "var(--color-surface, rgba(255,255,255,0.06))",
                color: on ? "#0a0a0c" : "var(--color-text, rgba(255,255,255,0.85))",
                fontSize: 13,
                fontWeight: 600,
                cursor: "pointer",
                textTransform: "capitalize",
                boxShadow: on ? "0 2px 10px rgba(0,0,0,0.25)" : "none",
                transition:
                  "background 0.16s ease, color 0.16s ease, border-color 0.16s ease, box-shadow 0.16s ease",
                userSelect: "none",
                outline: "none",
              }}
            >
              {c}
            </motion.button>
          );
        })}
      </motion.div>

      {isLoading && (
        <motion.p
          layout="position"
          transition={{ layout: REFLOW }}
          style={{ fontSize: 13.5, color: "var(--color-text-dim)", margin: 0 }}
        >
          Searching…
        </motion.p>
      )}

      {error && (
        <motion.p
          layout="position"
          transition={{ layout: REFLOW }}
          style={{ fontSize: 13.5, color: "var(--color-danger)", margin: 0 }}
        >
          {errMsg(error)}
        </motion.p>
      )}

      {!isLoading && !error && !hasResults && (
        <motion.div layout="position" transition={{ layout: REFLOW }}>
          <EmptyState
            title={`No results found for “${query}”`}
            description="Please make sure your words are spelled correctly, or use fewer or different keywords."
          />
        </motion.div>
      )}

      {data && (
        <>
          {/* TOP RESULT & SONGS ROW (when in 'all' view) */}
          {cat === "all" && (topResult || data.tracks.length > 0) && (
            <motion.div
              layout="position"
              transition={{ layout: REFLOW }}
              style={{
                display: "grid",
                gridTemplateColumns:
                  topResult && data.tracks.length > 0
                    ? "repeat(auto-fit, minmax(min(100%, 320px), 1fr))"
                    : "1fr",
                gap: "clamp(16px, 2.2vw, 28px)",
                alignItems: "stretch",
              }}
            >
              {topResult && (
                <motion.section
                  layout="position"
                  transition={{ layout: REFLOW }}
                  style={{ display: "flex", flexDirection: "column", height: "100%" }}
                >
                  <h2
                    style={{
                      margin: "0 0 14px",
                      fontSize: "clamp(18px, 2vw, 22px)",
                      fontWeight: 700,
                      letterSpacing: "-0.01em",
                      color: "var(--color-text-hi)",
                    }}
                  >
                    Top result
                  </h2>
                  <div style={{ flex: 1, minHeight: 0 }}>
                    <TopResultCard
                      result={topResult}
                      onPlay={handlePlayTopResult}
                      isPlaying={isTopResultPlaying}
                    />
                  </div>
                </motion.section>
              )}

              {data.tracks.length > 0 && (
                <motion.section
                  layout="position"
                  transition={{ layout: REFLOW }}
                  style={{ display: "flex", flexDirection: "column", height: "100%" }}
                >
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      marginBottom: 14,
                    }}
                  >
                    <h2
                      style={{
                        margin: 0,
                        fontSize: "clamp(18px, 2vw, 22px)",
                        fontWeight: 700,
                        letterSpacing: "-0.01em",
                        color: "var(--color-text-hi)",
                      }}
                    >
                      Songs
                    </h2>
                    {data.tracks.length > 4 && (
                      <button
                        onClick={() => setCat("songs")}
                        style={{
                          background: "none",
                          border: "none",
                          color: "var(--color-text-dim)",
                          fontSize: 12.5,
                          fontWeight: 600,
                          cursor: "pointer",
                          padding: "4px 8px",
                          borderRadius: 6,
                          transition: "color 0.15s ease",
                        }}
                        onMouseEnter={(e) =>
                          ((e.currentTarget as HTMLButtonElement).style.color =
                            "var(--color-text-hi)")
                        }
                        onMouseLeave={(e) =>
                          ((e.currentTarget as HTMLButtonElement).style.color =
                            "var(--color-text-dim)")
                        }
                      >
                        Show all
                      </button>
                    )}
                  </div>
                  <div className="flex flex-col">
                    {data.tracks.slice(0, 4).map((t, i) => (
                      <TrackRow
                        key={t.id}
                        track={t}
                        index={i}
                        showAlbum
                        liked={likedSet.has(t.id)}
                        onPlay={() => {
                          const start = playContext(data.tracks, i, "search");
                          if (start) {
                            setCurrentTrack(start);
                            playTrack(start.id).catch(console.error);
                          }
                        }}
                        onQueue={(track) => enqueue(track)}
                        onToggleLike={(track) =>
                          toggleLike.mutate({
                            id: track.id,
                            liked: likedSet.has(track.id),
                          })
                        }
                      />
                    ))}
                  </div>
                </motion.section>
              )}
            </motion.div>
          )}

          {/* FULL SONGS LIST (when 'songs' category is selected) */}
          {cat === "songs" && data.tracks.length > 0 && (
            <motion.section layout="position" transition={{ layout: REFLOW }}>
              <h2
                style={{
                  margin: "0 0 14px",
                  fontSize: "clamp(18px, 2vw, 22px)",
                  fontWeight: 700,
                  letterSpacing: "-0.01em",
                  color: "var(--color-text-hi)",
                }}
              >
                Songs
              </h2>
              <div className="flex flex-col">
                {data.tracks.map((t, i) => (
                  <TrackRow
                    key={t.id}
                    track={t}
                    index={i}
                    showAlbum
                    liked={likedSet.has(t.id)}
                    onPlay={() => {
                      const start = playContext(data.tracks, i, "search");
                      if (start) {
                        setCurrentTrack(start);
                        playTrack(start.id).catch(console.error);
                      }
                    }}
                    onQueue={(track) => enqueue(track)}
                    onToggleLike={(track) =>
                      toggleLike.mutate({
                        id: track.id,
                        liked: likedSet.has(track.id),
                      })
                    }
                  />
                ))}
              </div>
            </motion.section>
          )}

          {/* ARTISTS GRID */}
          {show("artists") && data.artists.length > 0 && (
            <motion.section layout="position" transition={{ layout: REFLOW }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  marginBottom: 14,
                }}
              >
                <h2
                  style={{
                    margin: 0,
                    fontSize: "clamp(18px, 2vw, 22px)",
                    fontWeight: 700,
                    letterSpacing: "-0.01em",
                    color: "var(--color-text-hi)",
                  }}
                >
                  Artists
                </h2>
                {cat === "all" && data.artists.length > 7 && (
                  <button
                    onClick={() => setCat("artists")}
                    style={{
                      background: "none",
                      border: "none",
                      color: "var(--color-text-dim)",
                      fontSize: 12.5,
                      fontWeight: 600,
                      cursor: "pointer",
                      padding: "4px 8px",
                      borderRadius: 6,
                      transition: "color 0.15s ease",
                    }}
                    onMouseEnter={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-hi)")
                    }
                    onMouseLeave={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-dim)")
                    }
                  >
                    Show all
                  </button>
                )}
              </div>
              <motion.div layout="position" transition={{ layout: REFLOW }}>
                <ArtistGrid>
                  {(cat === "all" ? data.artists.slice(0, 7) : data.artists).map((a) => (
                    <ArtistCard key={a.id} artist={a} />
                  ))}
                </ArtistGrid>
              </motion.div>
            </motion.section>
          )}

          {/* ALBUMS GRID */}
          {show("albums") && data.albums.length > 0 && (
            <motion.section layout="position" transition={{ layout: REFLOW }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  marginBottom: 14,
                }}
              >
                <h2
                  style={{
                    margin: 0,
                    fontSize: "clamp(18px, 2vw, 22px)",
                    fontWeight: 700,
                    letterSpacing: "-0.01em",
                    color: "var(--color-text-hi)",
                  }}
                >
                  Albums
                </h2>
                {cat === "all" && data.albums.length > 7 && (
                  <button
                    onClick={() => setCat("albums")}
                    style={{
                      background: "none",
                      border: "none",
                      color: "var(--color-text-dim)",
                      fontSize: 12.5,
                      fontWeight: 600,
                      cursor: "pointer",
                      padding: "4px 8px",
                      borderRadius: 6,
                      transition: "color 0.15s ease",
                    }}
                    onMouseEnter={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-hi)")
                    }
                    onMouseLeave={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-dim)")
                    }
                  >
                    Show all
                  </button>
                )}
              </div>
              <motion.div layout="position" transition={{ layout: REFLOW }}>
                <AlbumGrid>
                  {(cat === "all" ? data.albums.slice(0, 7) : data.albums).map((al) => (
                    <AlbumCard key={al.id} album={al} />
                  ))}
                </AlbumGrid>
              </motion.div>
            </motion.section>
          )}

          {/* PLAYLISTS GRID */}
          {show("playlists") && data.playlists.length > 0 && (
            <motion.section layout="position" transition={{ layout: REFLOW }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  marginBottom: 14,
                }}
              >
                <h2
                  style={{
                    margin: 0,
                    fontSize: "clamp(18px, 2vw, 22px)",
                    fontWeight: 700,
                    letterSpacing: "-0.01em",
                    color: "var(--color-text-hi)",
                  }}
                >
                  Playlists
                </h2>
                {cat === "all" && data.playlists.length > 7 && (
                  <button
                    onClick={() => setCat("playlists")}
                    style={{
                      background: "none",
                      border: "none",
                      color: "var(--color-text-dim)",
                      fontSize: 12.5,
                      fontWeight: 600,
                      cursor: "pointer",
                      padding: "4px 8px",
                      borderRadius: 6,
                      transition: "color 0.15s ease",
                    }}
                    onMouseEnter={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-hi)")
                    }
                    onMouseLeave={(e) =>
                      ((e.currentTarget as HTMLButtonElement).style.color =
                        "var(--color-text-dim)")
                    }
                  >
                    Show all
                  </button>
                )}
              </div>
              <motion.div
                layout="position"
                transition={{ layout: REFLOW }}
                style={{
                  display: "grid",
                  gridTemplateColumns:
                    "repeat(auto-fill, minmax(clamp(118px, 15vw, 160px), 1fr))",
                  gap: "clamp(10px, 1.5vw, 16px)",
                  width: "100%",
                }}
              >
                {(cat === "all" ? data.playlists.slice(0, 7) : data.playlists).map((pl) => (
                  <PlaylistResultCard key={pl.id} playlist={pl} />
                ))}
              </motion.div>
            </motion.section>
          )}
        </>
      )}
    </div>
  );
}
