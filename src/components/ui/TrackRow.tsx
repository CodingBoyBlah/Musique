import { memo, useState } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { motion, AnimatePresence } from "framer-motion";
import { Play, Plus, Heart, Music, Disc3, User, Link2, Globe, ListPlus, Trash2, Check } from "@/lib/icons";
import { Link, useNavigate } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import type { TrackItem } from "../../types/spotify";
import { fmtMs } from "../../utils/fmt";
import { gpuLayer, zTransform } from "../../lib/motion";
import { prefetchArtist, prefetchAlbum } from "../../lib/prefetch";
import { useContextMenu, type MenuEntry } from "./ContextMenu";
import { shareSpotifyLink, shareUniversalLink } from "../../lib/share";
import { useAddToPlaylistStore } from "../../store/addToPlaylist.store";
import { useYtMatchStore } from "../../store/ytMatch.store";
import { usePlaybackBackend } from "../../hooks/usePlaybackBackend";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { toast } from "../../store/toast.store";
import { transportPlay, transportPause } from "../../hooks/usePlayerControls";
import { AnimatedPlayPause, AnimatedHeart } from "../playground/AnimatedIcons";
import { Tooltip } from "./Tooltip";

interface Props {
  track:         TrackItem;
  index?:        number;
  showAlbum?:    boolean;
  showCover?:    boolean;   // per-track cover art (off for album pages, shared art)
  liked?:        boolean;
  onPlay?:       (track: TrackItem) => void;
  onQueue?:      (track: TrackItem) => void;
  onToggleLike?: (track: TrackItem) => void;
  // when set, the context menu shows "Remove from this playlist"
  onRemoveFromPlaylist?: (track: TrackItem) => void;
}

const stop = (e: React.MouseEvent) => e.stopPropagation();

// prominent animated row action button
function ActionBtn({
  children, onClick, title, active, className, accent,
}: {
  children: React.ReactNode;
  onClick:  (e: React.MouseEvent) => void;
  title:    string;
  active?:  boolean;
  className?: string;
  accent?:  boolean;
}) {
  return (
    <motion.button
      onClick={onClick}
      title={title}
      className={className}
      whileHover={{ scale: 1.18 }}
      whileTap={{ scale: 0.86 }}
      transition={{ type: "spring", stiffness: 420, damping: 22 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        display: "flex", alignItems: "center", justifyContent: "center",
        width: 30, height: 30, borderRadius: "50%", border: "none",
        background: active ? "var(--color-accent-dim)" : "transparent",
        color: active ? "var(--color-accent)" : (accent ? "var(--color-accent)" : "var(--color-text-dim)"),
        cursor: "pointer", flexShrink: 0,
      }}
      onMouseEnter={(e) => { if (!active) (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)"; }}
      onMouseLeave={(e) => { if (!active) (e.currentTarget as HTMLButtonElement).style.color = accent ? "var(--color-accent)" : "var(--color-text-dim)"; }}
    >
      {children}
    </motion.button>
  );
}

function TrackRowImpl({
  track, index, showAlbum = false, showCover = true, liked, onPlay, onQueue, onToggleLike,
  onRemoveFromPlaylist,
}: Props) {
  const [hover, setHover] = useState(false);
  const cover = showCover ? track.album?.image_url : null;
  const qc = useQueryClient();
  const navigate = useNavigate();
  const openAddToPlaylist = useAddToPlaylistStore((s) => s.open);
  const openYtMatch = useYtMatchStore((s) => s.open);
  // Cached hard and shared across every row, so this is one query for the
  // whole list rather than per-row work.
  const { data: backend } = usePlaybackBackend();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const isThisCurrent = usePlayerStore((s) => s.currentTrack?.id === track.id);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const isThisPlaying = Boolean(isThisCurrent && isPlaying);

  function handlePlayToggle(e?: React.MouseEvent) {
    if (e) {
      e.stopPropagation();
    }
    if (isThisPlaying) {
      transportPause();
    } else if (isThisCurrent && !isPlaying) {
      transportPlay();
    } else if (onPlay) {
      onPlay(track);
    }
  }

  const isQueued = useQueueStore((s) => s.queue.some((t) => t.id === track.id));
  const [justAdded, setJustAdded] = useState(false);

  function handleEnqueue(e?: React.MouseEvent) {
    if (e) stop(e);
    if (!onQueue) return;
    onQueue(track);
    setJustAdded(true);
    setTimeout(() => setJustAdded(false), 2200);
    toast(`Added "${track.name}" to queue`);
  }

  /* right-click menu -- every action that fits a track, built from the handlers
  this row got plus navigation + share (always available) */
  const menuEntries: MenuEntry[] = [];
  if (onPlay)  menuEntries.push({ label: isThisPlaying ? "Pause" : "Play", icon: <Play size={14} />, onSelect: () => handlePlayToggle() });
  if (onQueue) menuEntries.push({
    label: isQueued ? "In queue (add again)" : "Add to queue",
    icon: isQueued ? <Check size={14} style={{ color: "var(--color-accent)" }} /> : <Plus size={14} />,
    onSelect: () => handleEnqueue(),
  });
  if (onToggleLike) menuEntries.push({
    label: liked ? "Remove from Liked Songs" : "Save to Liked Songs",
    icon: <Heart size={14} fill={liked ? "currentColor" : "none"} />,
    onSelect: () => onToggleLike(track),
  });
  menuEntries.push({ label: "Add to playlist…", icon: <ListPlus size={14} />, onSelect: () => openAddToPlaylist(track) });
  if (onRemoveFromPlaylist) menuEntries.push({
    label: "Remove from this playlist",
    icon: <Trash2 size={14} />,
    danger: true,
    onSelect: () => onRemoveFromPlaylist(track),
  });
  if (track.artists[0]) menuEntries.push({ label: "Go to artist", icon: <User size={14} />, onSelect: () => navigate(`/artist/${track.artists[0].id}`) });
  if (track.album) menuEntries.push({ label: "Go to album", icon: <Disc3 size={14} />, onSelect: () => navigate(`/album/${track.album!.id}`) });
  // Only meaningful while audio actually comes from YouTube. This is the
  // escape hatch for a bad automatic match - matching refuses rather than
  // guessing, so a track that won't play needs somewhere to be corrected.
  if (backend?.active === "youtube") {
    menuEntries.push({
      label: "Change YouTube source…",
      icon: <Music size={14} />,
      onSelect: () => openYtMatch(track.id, track.name),
    });
  }
  menuEntries.push({ label: "Copy Spotify link",   icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("track", track.id) });
  menuEntries.push({ label: "Copy universal link", icon: <Globe size={14} />, onSelect: () => shareUniversalLink("track", track.id) });

  const baseBg = index != null && index % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent";

  return (
    <>
      <div
        role={onPlay ? "button" : undefined}
        tabIndex={onPlay ? 0 : undefined}
        onClick={() => { if (onPlay) handlePlayToggle(); }}
        onKeyDown={(e) => {
          if (onPlay && (e.key === "Enter" || e.key === " ")) {
            e.preventDefault();
            handlePlayToggle();
          }
        }}
        onMouseEnter={() => setHover(true)}
        onMouseLeave={() => setHover(false)}
        onContextMenu={openMenu(menuEntries)}
        className="group"
        style={{
          position: "relative",
          zIndex: hover ? 40 : 1,
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "8px 12px",
          margin: "1.5px 0",
          borderRadius: 8,
          transition: "background 0.12s ease",
          background: hover ? "var(--color-surface-hover, rgba(255,255,255,0.06))" : baseBg,
          cursor: onPlay ? "pointer" : "default",
          userSelect: "none",
        } as React.CSSProperties}
      >
        {/* Left Slot: Track Number or Play/Pause Button with blur+scale morph */}
        {(index != null || onPlay) && (
          <div style={{ position: "relative", width: 28, height: 28, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
            {hover || isThisPlaying ? (
              <motion.button
                whileHover={{ scale: 1.15 }}
                whileTap={{ scale: 0.88 }}
                onClick={(e) => {
                  stop(e);
                  handlePlayToggle();
                }}
                aria-label={isThisPlaying ? `Pause ${track.name}` : `Play ${track.name}`}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  width: 28,
                  height: 28,
                  borderRadius: "50%",
                  border: "none",
                  background: "transparent",
                  color: isThisPlaying ? "var(--color-accent)" : "#ffffff",
                  cursor: "pointer",
                  padding: 0,
                }}
              >
                <AnimatedPlayPause
                  isPlaying={isThisPlaying}
                  size={15}
                  strokeWidth={2.4}
                  fill={isThisPlaying ? "var(--color-accent)" : "#ffffff"}
                />
              </motion.button>
            ) : (
              <span
                style={{
                  fontSize: 13,
                  color: isThisCurrent ? "var(--color-accent)" : "var(--color-text-muted)",
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {index != null ? index + 1 : ""}
              </span>
            )}
          </div>
        )}

        {showCover && (
          cover ? (
            <img src={coverUrl(cover, 38) ?? cover} alt="" loading="lazy" decoding="async" style={{ width: 38, height: 38, borderRadius: 6, objectFit: "cover", flexShrink: 0, boxShadow: "0 2px 6px rgba(0,0,0,0.25)" }} />
          ) : (
            <div style={{ width: 38, height: 38, borderRadius: 6, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
              <Music size={16} style={{ color: "var(--color-text-dim)" }} />
            </div>
          )
        )}

        {/* Title & Artist with Live 3-Bar Equalizer */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <p
              style={{
                margin: 0,
                fontSize: 13.5,
                fontWeight: 600,
                letterSpacing: "-0.012em",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                color: isThisCurrent ? "var(--color-accent)" : "rgba(255,255,255,0.92)",
              }}
            >
              {track.name}
            </p>
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

          <p style={{ margin: "2px 0 0", fontSize: 12, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "var(--color-text-dim)" }}>
            {track.explicit && (
              <span style={{ display: "inline-block", marginRight: 5, padding: "1px 4px", borderRadius: 3, fontSize: 9.5, fontWeight: 700, background: "rgba(255,255,255,0.12)", color: "rgba(255,255,255,0.70)" }}>
                E
              </span>
            )}
            {track.artists.map((a, i) => (
              <span key={a.id}>
                {i > 0 && ", "}
                <Link
                  to={`/artist/${a.id}`}
                  onClick={stop}
                  style={{ color: "inherit", textDecoration: "none" }}
                  onMouseEnter={(e) => { (e.currentTarget as HTMLAnchorElement).style.textDecoration = "underline"; prefetchArtist(qc, a.id); }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLAnchorElement).style.textDecoration = "none"; }}
                >
                  {a.name}
                </Link>
              </span>
            ))}
            {showAlbum && track.album && (
              <>
                {" · "}
                <Link
                  to={`/album/${track.album.id}`}
                  onClick={stop}
                  style={{ color: "inherit", textDecoration: "none" }}
                  onMouseEnter={(e) => { (e.currentTarget as HTMLAnchorElement).style.textDecoration = "underline"; prefetchAlbum(qc, track.album!.id); }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLAnchorElement).style.textDecoration = "none"; }}
                >
                  {track.album.name}
                </Link>
              </>
            )}
          </p>
        </div>

        {onToggleLike && (
          <Tooltip label={liked ? "Remove from Liked Songs" : "Save to Liked Songs"} side="top">
            <ActionBtn
              onClick={(e) => { stop(e); onToggleLike(track); }}
              title=""
              className={liked ? "" : "queue-btn"}
              active={liked}
              accent={liked}
            >
              <AnimatedHeart liked={Boolean(liked)} size={15} />
            </ActionBtn>
          </Tooltip>
        )}

        {onQueue && (
          <Tooltip label={isQueued || justAdded ? "In queue" : "Add to queue"} side="top">
            <div style={{ position: "relative", display: "inline-flex", alignItems: "center", justifyContent: "center" }}>
              <ActionBtn
                onClick={handleEnqueue}
                title=""
                className={isQueued || justAdded ? "" : "queue-btn"}
                active={isQueued || justAdded}
                accent={isQueued || justAdded}
              >
                <AnimatePresence mode="wait" initial={false}>
                  {isQueued || justAdded ? (
                    <motion.span
                      key="check"
                      initial={{ scale: 0.3, rotate: -45, opacity: 0 }}
                      animate={{ scale: 1, rotate: 0, opacity: 1 }}
                      exit={{ scale: 0.3, rotate: 45, opacity: 0 }}
                      transition={{ type: "spring", stiffness: 480, damping: 24 }}
                      style={{ display: "flex", alignItems: "center", justifyContent: "center" }}
                    >
                      <Check size={14} strokeWidth={2.8} />
                    </motion.span>
                  ) : (
                    <motion.span
                      key="plus"
                      initial={{ scale: 0.3, opacity: 0 }}
                      animate={{ scale: 1, opacity: 1 }}
                      exit={{ scale: 0.3, opacity: 0 }}
                      transition={{ duration: 0.15 }}
                      style={{ display: "flex", alignItems: "center", justifyContent: "center" }}
                    >
                      <Plus size={15} strokeWidth={2.5} />
                    </motion.span>
                  )}
                </AnimatePresence>
              </ActionBtn>

              {/* Radiant ripple pulse ring on click */}
              {justAdded && (
                <motion.span
                  initial={{ scale: 0.8, opacity: 0.85 }}
                  animate={{ scale: 2.2, opacity: 0 }}
                  transition={{ duration: 0.5, ease: "easeOut" }}
                  style={{
                    position: "absolute",
                    inset: 0,
                    borderRadius: "50%",
                    border: "2px solid var(--color-accent)",
                    pointerEvents: "none",
                  }}
                />
              )}
            </div>
          </Tooltip>
        )}

        <span style={{ fontSize: 12.5, flexShrink: 0, color: "rgba(255,255,255,0.35)", fontVariantNumeric: "tabular-nums" }}>
          {fmtMs(track.duration_ms)}
        </span>
      </div>
      {menuEl}
    </>
  );
}

/* memoised: rows only re-render when their own track, index, liked or display props change,
avoiding cascade re-renders from inline function props or unrelated store changes */
export const TrackRow = memo(TrackRowImpl, (prev, next) => {
  return (
    prev.track.id === next.track.id &&
    prev.index === next.index &&
    prev.liked === next.liked &&
    prev.showCover === next.showCover &&
    prev.showAlbum === next.showAlbum
  );
});
