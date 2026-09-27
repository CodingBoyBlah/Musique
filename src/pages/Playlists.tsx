import { useEffect, useRef, useState, useId, memo } from "react";
import { coverUrl } from "../lib/coverUrl";
import { Link } from "react-router-dom";
import { motion, LayoutGroup } from "framer-motion";
import { ListMusic, Pin, PinOff } from "@/lib/icons";
import { useQueryClient } from "@tanstack/react-query";
import { getPlaylist } from "../api/spotify";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { SignInPrompt } from "../components/ui/SignInPrompt";
import { SyncButton } from "../components/ui/SyncButton";
import { CardGridSkeleton } from "../components/ui/Skeletons";
import { useMyPlaylists, useSyncLibrary } from "../hooks/useLibrary";
import { usePinsStore } from "../store/pins.store";
import { useContextMenu, type MenuEntry } from "../components/ui/ContextMenu";
import type { PlaylistSummary } from "../types/library";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { getGridItemTransition } from "../lib/motion";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 37 };
const MotionLink = motion.create(Link);

const PlaylistCard = memo(function PlaylistCard({
  playlist, onContextMenu, index = 0,
}: {
  playlist: PlaylistSummary;
  onContextMenu: (e: React.MouseEvent) => void;
  index?: number;
}) {
  useReflowPulse();
  const [hover, setHover] = useState(false);
  const qc = useQueryClient();
  const prefetchTimer = useRef<number | null>(null);

  const handleMouseEnter = () => {
    setHover(true);
    prefetchTimer.current = window.setTimeout(() => {
      qc.prefetchQuery({
        queryKey: ["playlist", playlist.id],
        queryFn: () => getPlaylist(playlist.id),
        staleTime: 120_000,
      });
    }, 40);
  };

  const handleMouseLeave = () => {
    setHover(false);
    if (prefetchTimer.current) {
      window.clearTimeout(prefetchTimer.current);
      prefetchTimer.current = null;
    }
  };

  return (
    <MotionLink
      to={`/playlist/${playlist.id}`}
      layout="position"
      transition={getGridItemTransition(index)}
      onContextMenu={onContextMenu}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      whileHover={{ y: -3 }}
      whileTap={{ scale: 0.98 }}
      style={{
        display:        "flex",
        flexDirection:  "column",
        gap:            10,
        padding:        "clamp(10px, 1.2vw, 14px)",
        borderRadius:   12,
        width:          "100%",
        boxSizing:      "border-box",
        textDecoration: "none",
        color:          "inherit",
        background:     hover ? "var(--color-surface-hover)" : "transparent",
        transition:     "background 0.18s ease",
        position:       "relative",
        cursor:         "pointer",
        minWidth:       0,
        overflow:       "hidden",
      }}
    >
      <div style={{ width: "100%", aspectRatio: "1 / 1", borderRadius: 8, overflow: "hidden", position: "relative", flexShrink: 0 }}>
        {playlist.image_url ? (
          <img
            src={coverUrl(playlist.image_url, 200) ?? playlist.image_url}
            alt={playlist.name}
            loading="lazy"
            decoding="async"
            style={{
              width: "100%",
              height: "100%",
              objectFit: "cover",
              display: "block",
              outline: "1px solid rgba(255, 255, 255, 0.08)",
              outlineOffset: -1,
            }}
          />
        ) : (
          <div
            style={{
              width: "100%",
              height: "100%",
              background: "rgba(124,111,255,0.14)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              outline: "1px solid rgba(124,111,255,0.22)",
              outlineOffset: -1,
            }}
          >
            <ListMusic size={38} strokeWidth={1.5} style={{ color: "rgba(124,111,255,0.6)" }} />
          </div>
        )}
      </div>
      <p
        style={{
          margin: 0,
          fontSize: "clamp(13px, 0.9vw, 14px)",
          fontWeight: 600,
          color: "var(--color-text-hi)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          lineHeight: "18px",
          height: 18,
          width: "100%",
          minWidth: 0,
          maxWidth: "100%",
          flexShrink: 0,
        }}
      >
        {playlist.name}
      </p>
      <p
        className="t-caption tnum"
        style={{
          margin: 0,
          fontSize: 12,
          color: "var(--color-text-dim)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          lineHeight: "15px",
          height: 15,
          width: "100%",
          minWidth: 0,
          maxWidth: "100%",
          flexShrink: 0,
        }}
      >
        {playlist.total_tracks} {playlist.total_tracks === 1 ? "song" : "songs"}
      </p>
    </MotionLink>
  );
});

export default function Playlists() {
  useReflowPulse();
  const layoutGroupId = useId();
  const { loggedIn } = useAuth();
  const { data: playlists = [], isLoading } = useMyPlaylists();
  const { mutate: sync, isPending } = useSyncLibrary();
  const isPinned  = usePinsStore((s) => s.isPinned);
  const togglePin = usePinsStore((s) => s.togglePin);
  const { open: openMenu, element: menuEl } = useContextMenu();
  const autoSynced = useRef(false);

  // first visit with an empty cache: pull the library from spotify once
  useEffect(() => {
    if (loggedIn && !isLoading && playlists.length === 0 && !isPending && !autoSynced.current) {
      autoSynced.current = true;
      sync();
    }
  }, [loggedIn, isLoading, playlists.length, isPending, sync]);

  function cardMenu(p: PlaylistSummary): MenuEntry[] {
    const pinned = isPinned(p.id);
    return [{
      label:  pinned ? "Unpin from sidebar" : "Pin to sidebar",
      icon:   pinned ? <PinOff size={14} /> : <Pin size={14} />,
      onSelect: () => togglePin({ id: p.id, name: p.name, image_url: p.image_url, type: "playlist" }),
    }];
  }

  if (!loggedIn) {
    return (
      <SignInPrompt
        heading="Playlists"
        title="Sign in to view your playlists"
        description="Log in with Spotify to access and play your saved playlists."
      />
    );
  }

  return (
    <motion.div layout="position" style={{ display: "flex", flexDirection: "column", gap: "clamp(20px, 2.5vw, 28px)" }}>
      <motion.div layout="position" style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
        <h1 className="t-title" style={{ margin: 0, color: "var(--color-text-hi)" }}>
          Playlists
        </h1>
        <SyncButton showStatus={false} />
      </motion.div>

      {/* also covers the first-visit auto sync: nothing to show yet, but
          something is on its way */}
      {(isLoading || (isPending && playlists.length === 0)) && (
        <div role="status" aria-label="Loading playlists">
          <CardGridSkeleton minCol="clamp(130px, 14vw, 170px)" />
        </div>
      )}

      {!isLoading && !isPending && playlists.length === 0 && (
        <EmptyState
          icon={<ListMusic size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
          title="No playlists yet"
          description="Pull in the playlists you've made or saved on Spotify."
          action={<SyncButton showStatus={false} prominent />}
        />
      )}

      {playlists.length > 0 && (
        <LayoutGroup id={layoutGroupId}>
          <motion.div
            layout="position"
            transition={{ layout: REFLOW }}
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(clamp(130px, 14vw, 170px), 1fr))",
              gap: "clamp(10px, 1.4vw, 16px)",
              width: "100%",
            }}
          >
            {playlists.map((p: PlaylistSummary, i: number) => (
              <PlaylistCard key={p.id} playlist={p} index={i} onContextMenu={openMenu(cardMenu(p))} />
            ))}
          </motion.div>
        </LayoutGroup>
      )}
      {menuEl}
    </motion.div>
  );
}
