import { memo, useState, useId } from "react";
import { Link } from "react-router-dom";
import { motion, LayoutGroup } from "framer-motion";
import { useQueryClient } from "@tanstack/react-query";
import type { AlbumItem } from "../../types/spotify";
import { CoverArt } from "./CoverArt";
import { ReleaseCountdown } from "./ReleaseCountdown";
import { releaseYear, isUpcoming } from "../../utils/fmt";
import { prefetchAlbum } from "../../lib/prefetch";
import { CirclePlayButton } from "./CirclePlayButton";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { getAlbum } from "../../api/spotify";
import { playTrack } from "../../api/playback";
import { transportPlay, transportPause } from "../../hooks/usePlayerControls";
import { gpuLayer, zTransform, REFLOW_SPRING, getGridItemTransition } from "../../lib/motion";
import "../../styles/ui.css";
import { useReflowPulse } from "../../hooks/useReflowPulse";
import { TILE_GRID, TILE_PAD } from "../../lib/layout";

const MotionLink = motion.create(Link);

interface Props {
  album: AlbumItem;
  size?: number;
  index?: number;
  style?: React.CSSProperties;
}

export function AlbumGrid({ children }: { children: React.ReactNode }) {
  const layoutGroupId = useId();
  return (
    <LayoutGroup id={layoutGroupId}>
      <motion.div
        layout="position"
        transition={{ layout: REFLOW_SPRING }}
        data-rail-lock="flip"
        style={TILE_GRID}
      >
        {children}
      </motion.div>
    </LayoutGroup>
  );
}

function AlbumCardImpl({ album, size = 160, index = 0, style }: Props) {
  useReflowPulse();
  const imgSize = size - 24;
  const [hover, setHover] = useState(false);
  // the album is being fetched after a play press - shown on the button at
  // once, so a slow network never reads as a dead click
  const [pending, setPending] = useState(false);
  const qc = useQueryClient();

  const isCurrentAlbum = usePlayerStore((s) => Boolean(album.id && s.currentTrack?.album?.id === album.id));
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const isThisAlbumPlaying = isCurrentAlbum && isPlaying;

  async function handlePlayAlbum(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (isThisAlbumPlaying) {
      transportPause();
      return;
    }
    if (isCurrentAlbum && !isPlaying) {
      transportPlay();
      return;
    }
    if (pending) return;
    setPending(true);
    try {
      const full = await getAlbum(album.id);
      const tracks = full?.tracks ?? [];
      if (tracks.length > 0) {
        const start = useQueueStore.getState().playContext(tracks, 0, album.id, `spotify:album:${album.id}`);
        if (start) {
          usePlayerStore.getState().setCurrentTrack(start);
          playTrack(start.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(() => {});
        }
      }
    } catch (err) {
      console.error(err);
    } finally {
      setPending(false);
    }
  }

  return (
    <MotionLink
      to={`/album/${album.id}`}
      layout="position"
      className="card-link"
      transformTemplate={zTransform}
      onMouseEnter={() => { setHover(true); prefetchAlbum(qc, album.id); }}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setHover(true)}
      onBlur={() => setHover(false)}
      whileHover={{ y: -3 }}
      whileTap={{ scale: 0.98 }}
      transition={{
        type: "spring",
        stiffness: 480,
        damping: 36,
        ...getGridItemTransition(index),
      }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 8,
        padding: TILE_PAD,
        borderRadius: 12,
        width: "100%",
        boxSizing: "border-box",
        textDecoration: "none",
        color: "inherit",
        background: hover ? "var(--color-surface-hover)" : "transparent",
        transition: "background 0.18s ease",
        position: "relative",
        cursor: "pointer",
        minWidth: 0,
        overflow: "hidden",
        ...gpuLayer,
        ...style,
      }}
    >
      <div
        style={{
          position: "relative",
          width: "100%",
          aspectRatio: "1 / 1",
          borderRadius: 8,
          overflow: "hidden",
          boxShadow: hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)",
          transition: "box-shadow 0.25s ease",
          flexShrink: 0,
        }}
      >
        <CoverArt url={album.image_url} alt={album.name} size={imgSize} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />

        {/* Hover play button with blur and scale icon morph from playground */}
        <CirclePlayButton
          isPlaying={isThisAlbumPlaying}
          visible={hover || isThisAlbumPlaying || pending}
          pending={pending}
          onClick={handlePlayAlbum}
          size={40}
          iconSize={16}
          style={{ position: "absolute", right: 8, bottom: 8 }}
          ariaLabel={isThisAlbumPlaying ? `Pause ${album.name}` : `Play ${album.name}`}
        />
      </div>

      <span
        style={{
          display: "block",
          fontSize: 13.5,
          fontWeight: 600,
          letterSpacing: "-0.012em",
          lineHeight: "17px",
          height: 17,
          color: isThisAlbumPlaying ? "var(--color-accent)" : "var(--color-text-hi)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          width: "100%",
          minWidth: 0,
          maxWidth: "100%",
          flexShrink: 0,
        }}
      >
        {album.name}
      </span>

      <div
        style={{
          height: 15,
          lineHeight: "15px",
          width: "100%",
          minWidth: 0,
          maxWidth: "100%",
          overflow: "hidden",
          flexShrink: 0,
        }}
      >
        {isUpcoming(album.release_date) ? (
          <ReleaseCountdown date={album.release_date!} />
        ) : (
          <span
            className="t-caption"
            style={{
              fontSize: 12,
              lineHeight: "15px",
              height: 15,
              display: "block",
              color: "var(--color-text-dim)",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
              width: "100%",
              minWidth: 0,
              maxWidth: "100%",
            }}
          >
            {releaseYear(album.release_date)} • {album.artists?.map((a) => a.name).join(", ") || album.album_type}
          </span>
        )}
      </div>
    </MotionLink>
  );
}

export const AlbumCard = memo(AlbumCardImpl);
