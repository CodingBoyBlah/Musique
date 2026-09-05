import { memo, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
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
import { gpuLayer, zTransform } from "../../lib/motion";

const MotionLink = motion.create(Link);

interface Props {
  album: AlbumItem;
  size?: number;
}

export function AlbumGrid({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(clamp(118px, 15vw, 160px), 1fr))",
        gap: "clamp(10px, 1.5vw, 16px)",
        width: "100%",
      }}
    >
      {children}
    </div>
  );
}

function AlbumCardImpl({ album, size = 160 }: Props) {
  const imgSize = size - 24;
  const [hover, setHover] = useState(false);
  const qc = useQueryClient();

  const currentTrack = usePlayerStore((s) => s.currentTrack);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const setPlaying = usePlayerStore((s) => s.setPlaying);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);

  const isThisAlbumPlaying = Boolean(album.id && currentTrack?.album?.id === album.id && isPlaying);

  async function handlePlayAlbum(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (isThisAlbumPlaying) {
      transportPause();
      return;
    }
    if (currentTrack?.album?.id === album.id && !isPlaying) {
      transportPlay();
      return;
    }
    try {
      const full = await getAlbum(album.id);
      const tracks = full?.tracks ?? [];
      if (tracks.length > 0) {
        const start = playContext(tracks, 0, album.id);
        if (start) {
          setCurrentTrack(start);
          playTrack(start.id).then(() => setPlaying(true)).catch(() => {});
        }
      }
    } catch (err) {
      console.error(err);
    }
  }

  return (
    <MotionLink
      to={`/album/${album.id}`}
      transformTemplate={zTransform}
      onMouseEnter={() => { setHover(true); prefetchAlbum(qc, album.id); }}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      transition={{ type: "spring", stiffness: 520, damping: 44 }}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 8,
        padding: 14,
        borderRadius: 14,
        width: "100%",
        boxSizing: "border-box",
        textDecoration: "none",
        color: "inherit",
        background: hover ? "var(--color-surface-elevated)" : "transparent",
        transition: "background 0.18s ease",
        position: "relative",
        cursor: "pointer",
        minWidth: 0,
        ...gpuLayer,
      }}
    >
      <div style={{ position: "relative", width: "100%", aspectRatio: "1 / 1", borderRadius: 8, overflow: "hidden" }}>
        <CoverArt url={album.image_url} alt={album.name} size={imgSize} style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} />

        {/* Hover play button with blur and scale icon morph from playground */}
        <CirclePlayButton
          isPlaying={isThisAlbumPlaying}
          visible={hover || isThisAlbumPlaying}
          onClick={handlePlayAlbum}
          size={42}
          iconSize={17}
          style={{ position: "absolute", right: 10, bottom: 10 }}
          ariaLabel={isThisAlbumPlaying ? `Pause ${album.name}` : `Play ${album.name}`}
        />
      </div>

      <span
        style={{
          fontSize: 14,
          fontWeight: 500,
          color: isThisAlbumPlaying ? "var(--color-accent)" : "var(--color-text-hi)",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
          maxWidth: "100%",
        }}
      >
        {album.name}
      </span>

      {isUpcoming(album.release_date) ? (
        <ReleaseCountdown date={album.release_date!} />
      ) : (
        <span style={{ fontSize: 12, color: "var(--color-text-dim)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", maxWidth: "100%" }}>
          {releaseYear(album.release_date)} • {album.artists?.map((a) => a.name).join(", ") || album.album_type}
        </span>
      )}
    </MotionLink>
  );
}

export const AlbumCard = memo(AlbumCardImpl);
