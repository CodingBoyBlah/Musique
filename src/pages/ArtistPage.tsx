import { useState } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Check, UserPlus, Share2, Play, Shuffle } from "lucide-react";
import { useArtist } from "../hooks/useArtist";
import { CoverArt } from "../components/ui/CoverArt";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { TrackRow } from "../components/ui/TrackRow";
import { Loader } from "../components/ui/Loader";
import { useContextMenu } from "../components/ui/ContextMenu";
import { shareSpotifyLink, shareUniversalLink } from "../lib/share";
import { Link2, Globe } from "lucide-react";
import {
  useIsArtistFollowed,
  useToggleFollow,
  useSavedTrackIds,
  useToggleLike,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";
import { gpuLayer, zTransform } from "../lib/motion";
import { errMsg } from "../lib/err";

const TOP_TRACKS_COLLAPSED = 5;

export default function ArtistPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useArtist(id);
  const { data: following = false } = useIsArtistFollowed(id);
  const toggleFollow = useToggleFollow();

  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue = useQueueStore((s) => s.enqueue);
  const playContext = useQueueStore((s) => s.playContext);
  const playContextShuffled = useQueueStore((s) => s.playContextShuffled);
  const toggleLike = useToggleLike();

  const [showAllTop, setShowAllTop] = useState(false);
  const { open: openMenu, element: menuEl } = useContextMenu();
  const shareEntries = [
    {
      label: "Copy Spotify link",
      icon: <Link2 size={14} />,
      onSelect: () => id && shareSpotifyLink("artist", id),
    },
    {
      label: "Copy universal link",
      icon: <Globe size={14} />,
      onSelect: () => id && shareUniversalLink("artist", id),
    },
  ];

  const topTracks = data?.top_tracks ?? [];
  const { data: savedIds = [] } = useSavedTrackIds(topTracks.map((t) => t.id));
  const likedSet = new Set(savedIds);

  if (isLoading) return <Loader label="Loading artist" />;

  if (error) {
    return (
      <div className="flex flex-col gap-4">
        <p className="text-sm" style={{ color: "var(--color-danger)" }}>
          {errMsg(error)}
        </p>
      </div>
    );
  }
  if (!data) return null;

  const shownTop = showAllTop
    ? topTracks
    : topTracks.slice(0, TOP_TRACKS_COLLAPSED);

  function startTop(index: number) {
    const start = playContext(
      showAllTop ? topTracks : topTracks.slice(0, TOP_TRACKS_COLLAPSED),
      index,
      `artist-top-${data!.id}`,
    );
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
    }
  }

  function playAll() {
    const start = playContext(topTracks, 0, `artist-top-${data!.id}`);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
    }
  }

  function shuffleAll() {
    const start = playContextShuffled(topTracks, `artist-top-${data!.id}`);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
    }
  }

  return (
    <div className="flex flex-col gap-8">
      {/* header - wraps + scales down on narrow widths instead of squishing */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-end",
          gap: 24,
          flexWrap: "wrap",
          minWidth: 0,
        }}
      >
        <div style={{ flexShrink: 0 }}>
          <CoverArt url={data.image_url} alt={data.name} size={200} rounded />
        </div>
        <div
          className="flex flex-col gap-2 min-w-0"
          style={{ flex: "1 1 260px" }}
        >
          <p
            className="text-[10px] font-bold uppercase tracking-widest"
            style={{ color: "var(--color-text-dim)" }}
          >
            Artist
          </p>
          <h1
            className="font-black"
            style={{
              fontSize: "clamp(28px, 5vw, 52px)",
              lineHeight: 1.05,
              letterSpacing: "-0.02em",
            }}
          >
            {data.name}
          </h1>
          {data.popularity != null && (
            <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>
              Popularity: {data.popularity} / 100
            </p>
          )}

          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              marginTop: 4,
              flexWrap: "wrap",
              rowGap: 10,
            }}
          >
            {topTracks.length > 0 && (
              <motion.button
                onClick={playAll}
                title="Play"
                whileHover={{ scale: 1.04 }}
                whileTap={{ scale: 0.96 }}
                transformTemplate={zTransform}
                style={{
                  ...gpuLayer,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: 8,
                  height: 44,
                  padding: "0 24px",
                  borderRadius: 99,
                  border: "none",
                  background: "var(--color-accent)",
                  color: "#fff",
                  fontSize: 14,
                  fontWeight: 700,
                  cursor: "pointer",
                  flexShrink: 0,
                }}
              >
                <Play size={17} fill="currentColor" strokeWidth={0} /> Play
              </motion.button>
            )}

            {topTracks.length > 0 && (
              <motion.button
                onClick={shuffleAll}
                title="Shuffle play"
                whileHover={{ scale: 1.04 }}
                whileTap={{ scale: 0.96 }}
                transformTemplate={zTransform}
                style={{
                  ...gpuLayer,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: 8,
                  height: 44,
                  padding: "0 18px",
                  borderRadius: 99,
                  border: "1px solid var(--color-border)",
                  background: "var(--color-surface)",
                  color: "var(--color-text-hi)",
                  fontSize: 14,
                  fontWeight: 600,
                  cursor: "pointer",
                  flexShrink: 0,
                }}
              >
                <Shuffle size={16} /> Shuffle
              </motion.button>
            )}

            <motion.button
              onClick={() => id && toggleFollow.mutate({ id, following })}
              whileHover={{ scale: 1.04 }}
              whileTap={{ scale: 0.96 }}
              transformTemplate={zTransform}
              style={{
                ...gpuLayer,
                display: "flex",
                alignItems: "center",
                gap: 8,
                height: 44,
                padding: "0 18px",
                width: "fit-content",
                borderRadius: 99,
                flexShrink: 0,
                border: following ? "1px solid var(--color-border)" : "none",
                background: following ? "transparent" : "var(--color-accent)",
                color: following ? "var(--color-text-hi)" : "#fff",
                fontSize: 13,
                fontWeight: 700,
                cursor: "pointer",
              }}
            >
              {following ? (
                <>
                  <Check size={15} strokeWidth={2.5} /> Following
                </>
              ) : (
                <>
                  <UserPlus size={15} strokeWidth={2.5} /> Follow
                </>
              )}
            </motion.button>

            <motion.button
              onClick={(e) => openMenu(shareEntries)(e)}
              onContextMenu={openMenu(shareEntries)}
              title="Share"
              whileHover={{ scale: 1.06 }}
              whileTap={{ scale: 0.94 }}
              transformTemplate={zTransform}
              style={{
                ...gpuLayer,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: 44,
                height: 44,
                flexShrink: 0,
                borderRadius: "50%",
                border: "1px solid var(--color-border)",
                background: "transparent",
                color: "var(--color-text)",
                cursor: "pointer",
              }}
            >
              <Share2 size={16} />
            </motion.button>
          </div>

          {data.genres.length > 0 && (
            <div className="flex flex-wrap" style={{ gap: 8, marginTop: 6 }}>
              {data.genres.slice(0, 5).map((g) => (
                <span
                  key={g}
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    padding: "4px 12px",
                    borderRadius: 99,
                    background: "var(--color-surface)",
                    border: "1px solid var(--color-border)",
                    color: "var(--color-text-dim)",
                    fontSize: 11.5,
                    fontWeight: 600,
                    lineHeight: 1.2,
                    letterSpacing: "0.01em",
                    textTransform: "capitalize",
                    whiteSpace: "nowrap",
                  }}
                >
                  {g}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* top tracks */}
      {topTracks.length > 0 && (
        <section>
          <h2 className="text-xl font-bold mb-4">Popular</h2>
          <div>
            {shownTop.map((t, i) => (
              <motion.div
                key={t.id}
                layout="position"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{
                  duration: 0.28,
                  delay: Math.min(i, 6) * 0.03,
                  ease: [0.23, 1, 0.32, 1],
                }}
              >
                <TrackRow
                  track={t}
                  index={i}
                  showAlbum
                  liked={likedSet.has(t.id)}
                  onPlay={() => startTop(i)}
                  onQueue={(track) => enqueue(track)}
                  onToggleLike={(track) =>
                    toggleLike.mutate({
                      id: track.id,
                      liked: likedSet.has(track.id),
                    })
                  }
                />
              </motion.div>
            ))}
          </div>
          {topTracks.length > TOP_TRACKS_COLLAPSED && (
            <button
              onClick={() => setShowAllTop((v) => !v)}
              style={{
                marginTop: 8,
                padding: "6px 4px",
                border: "none",
                background: "transparent",
                color: "var(--color-text-dim)",
                fontSize: 13,
                fontWeight: 600,
                cursor: "pointer",
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLButtonElement).style.color =
                  "var(--color-text-hi)";
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLButtonElement).style.color =
                  "var(--color-text-dim)";
              }}
            >
              {showAllTop ? "Show less" : "Show more"}
            </button>
          )}
        </section>
      )}

      {/* discography (albums) */}
      {data.albums.length > 0 && (
        <section>
          <h2 className="text-xl font-bold mb-4">Discography</h2>
          <AlbumGrid>
            {data.albums.map((al) => (
              <AlbumCard key={al.id} album={al} />
            ))}
          </AlbumGrid>
        </section>
      )}

      {/* singles & EPs */}
      {data.singles.length > 0 && (
        <section>
          <h2 className="text-xl font-bold mb-4">Singles & EPs</h2>
          <AlbumGrid>
            {data.singles.map((al) => (
              <AlbumCard key={al.id} album={al} />
            ))}
          </AlbumGrid>
        </section>
      )}
      {menuEl}
    </div>
  );
}
