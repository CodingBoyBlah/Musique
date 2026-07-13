import { memo } from "react";
import { motion } from "framer-motion";
import { Play, Plus, Heart, Music, Disc3, User, Link2, Globe, ListPlus, Trash2 } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import type { TrackItem } from "../../types/spotify";
import { fmtMs } from "../../utils/fmt";
import { gpuLayer, zTransform } from "../../lib/motion";
import { prefetchArtist, prefetchAlbum } from "../../lib/prefetch";
import { useContextMenu, type MenuEntry } from "./ContextMenu";
import { shareSpotifyLink, shareUniversalLink } from "../../lib/share";
import { useAddToPlaylistStore } from "../../store/addToPlaylist.store";

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
  const cover = showCover ? track.album?.image_url : null;
  const qc = useQueryClient();
  const navigate = useNavigate();
  const openAddToPlaylist = useAddToPlaylistStore((s) => s.open);
  const { open: openMenu, element: menuEl } = useContextMenu();

  /* right-click menu -- every action that fits a track, built from the handlers
  this row got plus navigation + share (always available) */
  const menuEntries: MenuEntry[] = [];
  if (onPlay)  menuEntries.push({ label: "Play", icon: <Play size={14} />, onSelect: () => onPlay(track) });
  if (onQueue) menuEntries.push({ label: "Add to queue", icon: <Plus size={14} />, onSelect: () => onQueue(track) });
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
  menuEntries.push({ label: "Copy Spotify link",   icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("track", track.id) });
  menuEntries.push({ label: "Copy universal link", icon: <Globe size={14} />, onSelect: () => shareUniversalLink("track", track.id) });

  
  const baseBg = index != null && index % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent";

  return (
    <>
    <motion.div
      layout
      style={{
        display: "flex", alignItems: "center", gap: 12,
        padding: "9px 12px", borderRadius: 8, transition: "background 0.1s",
        background: baseBg,
        
        contentVisibility: "auto",
        /*
        aiuto lets the browser remember each rows REAL rendered hight and REUSES it instead of snapping evrery offscreen row to a guessed height
        */
       
        containIntrinsicSize: "auto 60px",
      } as React.CSSProperties}
      onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = "var(--color-surface-hover)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = baseBg; }}
      onContextMenu={openMenu(menuEntries)}
      className="group"
    >
      {(index != null || onPlay) && (
        <div style={{ position: "relative", width: 28, height: 30, flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
          {index != null && (
            <span className="row-idx" style={{ fontSize: 13, color: "var(--color-text-muted)", fontVariantNumeric: "tabular-nums" }}>
              {index + 1}
            </span>
          )}
          {onPlay && (
            <div
              className={index != null ? "row-play" : undefined}
              style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center" }}
            >
              <ActionBtn onClick={(e) => { stop(e); onPlay(track); }} title={`Play ${track.name}`}>
                <Play size={14} strokeWidth={2} style={{ marginLeft: 1 }} />
              </ActionBtn>
            </div>
          )}
        </div>
      )}

      {showCover && (
        cover ? (
          <img src={cover} alt="" loading="lazy" decoding="async" style={{ width: 42, height: 42, borderRadius: 6, objectFit: "cover", flexShrink: 0 }} />
        ) : (
          <div style={{ width: 42, height: 42, borderRadius: 6, background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
            <Music size={16} style={{ color: "var(--color-text-dim)" }} />
          </div>
        )
      )}

      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ margin: 0, fontSize: 14, fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "rgba(255,255,255,0.90)" }}>
          {track.name}
        </p>
        <p style={{ margin: 0, fontSize: 12.5, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: "rgba(255,255,255,0.45)" }}>
          {track.explicit && (
            <span style={{ display: "inline-block", marginRight: 4, padding: "0 3px", borderRadius: 2, fontSize: 9, fontWeight: 700, background: "rgba(255,255,255,0.18)", color: "rgba(255,255,255,0.55)" }}>
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
        <ActionBtn
          onClick={(e) => { stop(e); onToggleLike(track); }}
          title={liked ? "Remove from Liked Songs" : "Save to Liked Songs"}
          className={liked ? "" : "queue-btn"}
          active={liked}
          accent={liked}
        >
          <Heart size={15} strokeWidth={2.5} fill={liked ? "currentColor" : "none"} />
        </ActionBtn>
      )}

      {onQueue && (
        <ActionBtn onClick={(e) => { stop(e); onQueue(track); }} title="Add to queue" className="queue-btn">
          <Plus size={15} strokeWidth={2.5} />
        </ActionBtn>
      )}

      <span style={{ fontSize: 12.5, flexShrink: 0, color: "rgba(255,255,255,0.35)", fontVariantNumeric: "tabular-nums" }}>
        {fmtMs(track.duration_ms)}
      </span>
    </motion.div>
    {menuEl}
    </>
  );
}

/* memoised: rows only re-render when their own props change, not on every
parent re-render (eg the per-second progress tick elsewhere) */
export const TrackRow = memo(TrackRowImpl);
