import { useState } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Check, UserPlus, Share2, Shuffle } from "lucide-react";
import { useArtist } from "../hooks/useArtist";
import { CoverArt } from "../components/ui/CoverArt";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { TrackRow } from "../components/ui/TrackRow";
import { Loader } from "../components/ui/Loader";
import { useContextMenu } from "../components/ui/ContextMenu";
import { Tooltip } from "../components/ui/Tooltip";
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
import { playTrack, pausePlayback } from "../api/playback";
import { gpuLayer, zTransform } from "../lib/motion";
import { errMsg } from "../lib/err";
import { AnimatedPlayPause } from "../components/playground/AnimatedIcons";

const TOP_TRACKS_COLLAPSED = 5;

export default function ArtistPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useArtist(id);
  const { data: following = false } = useIsArtistFollowed(id);
  const toggleFollow = useToggleFollow();

  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const isCompact = lyricsOpen || queueOpen;
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

  const isContextPlaying = isPlaying && !!currentTrack && topTracks.some((t) => t.id === currentTrack.id);

  function playAll() {
    if (isContextPlaying) {
      pausePlayback().catch(console.error);
      return;
    }
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
      {/* header - smoothly scales down with spring physics when lyrics/queue rail opens */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-end",
          gap: isCompact ? 16 : 24,
          flexWrap: "wrap",
          minWidth: 0,
          transition: "gap 0.35s cubic-bezier(0.23, 1, 0.32, 1)",
        }}
      >
        <motion.div
          initial={false}
          animate={{
            width: isCompact ? 144 : 200,
            height: isCompact ? 144 : 200,
          }}
          transition={{ type: "spring", stiffness: 340, damping: 34 }}
          style={{
            width: isCompact ? 144 : 200,
            height: isCompact ? 144 : 200,
            flexShrink: 0,
            borderRadius: "50%",
            overflow: "hidden",
          }}
        >
          <div style={{ width: "100%", height: "100%" }}>
            <CoverArt url={data.image_url} alt={data.name} size={200} rounded style={{ width: "100%", height: "100%" }} />
          </div>
        </motion.div>

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

          <motion.h1
            initial={false}
            animate={{
              fontSize: isCompact ? 32 : 48,
            }}
            transition={{ type: "spring", stiffness: 340, damping: 34 }}
            className="font-black"
            style={{
              fontSize: isCompact ? 32 : 48,
              lineHeight: 1.05,
              letterSpacing: "-0.02em",
              color: "#ffffff",
            }}
          >
            {data.name}
          </motion.h1>

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
              <Tooltip label={isContextPlaying ? "Pause" : "Play top tracks"} side="top">
                <motion.button
                  initial={false}
                  onClick={playAll}
                  whileHover={{ scale: 1.04 }}
                  whileTap={{ scale: 0.96 }}
                  animate={{
                    width: isCompact ? 44 : 114,
                    paddingLeft: isCompact ? 0 : 20,
                    paddingRight: isCompact ? 0 : 20,
                  }}
                  transition={{ type: "spring", stiffness: 320, damping: 30 }}
                  transformTemplate={zTransform}
                  style={{
                    ...gpuLayer,
                    width: isCompact ? 44 : 114,
                    paddingLeft: isCompact ? 0 : 20,
                    paddingRight: isCompact ? 0 : 20,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    height: 44,
                    minWidth: 44,
                    borderRadius: 99,
                    border: "none",
                    background: "var(--color-accent)",
                    color: "var(--color-accent-text, #ffffff)",
                    fontSize: 14,
                    fontWeight: 700,
                    letterSpacing: "-0.01em",
                    cursor: "pointer",
                    flexShrink: 0,
                    boxShadow: isContextPlaying
                      ? "0 0 0 4px var(--color-accent-dim), 0 4px 18px var(--color-accent-dim)"
                      : "0 4px 18px -2px var(--color-accent-dim)",
                    transition: "box-shadow 0.2s, background 0.15s",
                    overflow: "hidden",
                    whiteSpace: "nowrap",
                  }}
                >
                  <AnimatedPlayPause
                    isPlaying={isContextPlaying}
                    size={16}
                    strokeWidth={0}
                    fill="currentColor"
                  />
                  <motion.span
                    initial={false}
                    animate={{
                      maxWidth: isCompact ? 0 : 54,
                      opacity: isCompact ? 0 : 1,
                      filter: isCompact ? "blur(6px)" : "blur(0px)",
                      scale: isCompact ? 0.75 : 1,
                      marginLeft: isCompact ? 0 : 8,
                    }}
                    transition={{
                      maxWidth: { type: "spring", stiffness: 320, damping: 30 },
                      marginLeft: { type: "spring", stiffness: 320, damping: 30 },
                      opacity: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                      filter: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                      scale: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                    }}
                    style={{
                      maxWidth: isCompact ? 0 : 54,
                      opacity: isCompact ? 0 : 1,
                      marginLeft: isCompact ? 0 : 8,
                      display: "inline-block",
                      overflow: "hidden",
                      whiteSpace: "nowrap",
                      willChange: "transform, filter, opacity, max-width",
                    }}
                  >
                    {isContextPlaying ? "Pause" : "Play"}
                  </motion.span>
                </motion.button>
              </Tooltip>
            )}

            {topTracks.length > 0 && (
              <Tooltip label="Shuffle play" side="top">
                <motion.button
                  initial={false}
                  onClick={shuffleAll}
                  whileHover={{ scale: 1.04 }}
                  whileTap={{ scale: 0.96 }}
                  animate={{
                    width: isCompact ? 44 : 114,
                    paddingLeft: isCompact ? 0 : 18,
                    paddingRight: isCompact ? 0 : 18,
                  }}
                  transition={{ type: "spring", stiffness: 320, damping: 30 }}
                  transformTemplate={zTransform}
                  style={{
                    ...gpuLayer,
                    width: isCompact ? 44 : 114,
                    paddingLeft: isCompact ? 0 : 18,
                    paddingRight: isCompact ? 0 : 18,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    height: 44,
                    minWidth: 44,
                    borderRadius: 99,
                    border: "1px solid rgba(255, 255, 255, 0.16)",
                    background: "rgba(255, 255, 255, 0.08)",
                    color: "#ffffff",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.01em",
                    cursor: "pointer",
                    flexShrink: 0,
                    overflow: "hidden",
                    whiteSpace: "nowrap",
                    transition: "border 0.2s, background 0.2s",
                  }}
                >
                  <Shuffle size={16} strokeWidth={2.2} />
                  <motion.span
                    initial={false}
                    animate={{
                      maxWidth: isCompact ? 0 : 56,
                      opacity: isCompact ? 0 : 1,
                      filter: isCompact ? "blur(6px)" : "blur(0px)",
                      scale: isCompact ? 0.75 : 1,
                      marginLeft: isCompact ? 0 : 8,
                    }}
                    transition={{
                      maxWidth: { type: "spring", stiffness: 320, damping: 30 },
                      marginLeft: { type: "spring", stiffness: 320, damping: 30 },
                      opacity: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                      filter: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                      scale: {
                        duration: isCompact ? 0.15 : 0.24,
                        delay: isCompact ? 0 : 0.06,
                        ease: [0.23, 1, 0.32, 1],
                      },
                    }}
                    style={{
                      maxWidth: isCompact ? 0 : 56,
                      opacity: isCompact ? 0 : 1,
                      marginLeft: isCompact ? 0 : 8,
                      display: "inline-block",
                      overflow: "hidden",
                      whiteSpace: "nowrap",
                      willChange: "transform, filter, opacity, max-width",
                    }}
                  >
                    Shuffle
                  </motion.span>
                </motion.button>
              </Tooltip>
            )}

            <Tooltip label={following ? "Unfollow artist" : "Follow artist"} side="top">
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
            </Tooltip>

            <Tooltip label="Share options" side="top">
              <motion.button
                onClick={(e) => openMenu(shareEntries)(e)}
                onContextMenu={openMenu(shareEntries)}
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
            </Tooltip>
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
