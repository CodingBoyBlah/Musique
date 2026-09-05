import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { ListMusic, RefreshCw, Pin, PinOff } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import { useMyPlaylists, useSyncLibrary } from "../hooks/useLibrary";
import { usePinsStore } from "../store/pins.store";
import { useContextMenu, type MenuEntry } from "../components/ui/ContextMenu";
import type { PlaylistSummary } from "../types/library";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };
const MotionLink = motion.create(Link);

function PlaylistCard({
  playlist, onContextMenu,
}: {
  playlist: PlaylistSummary;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  const [hover, setHover] = useState(false);

  return (
    <MotionLink
      to={`/playlist/${playlist.id}`}
      layout="position"
      transition={{ layout: REFLOW }}
      onContextMenu={onContextMenu}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      whileHover={{ y: -3 }}
      whileTap={{ scale: 0.98 }}
      style={{
        display:        "flex",
        flexDirection:  "column",
        gap:            10,
        padding:        "clamp(10px, 1.2vw, 14px)",
        borderRadius:   14,
        width:          "100%",
        boxSizing:      "border-box",
        textDecoration: "none",
        color:          "inherit",
        background:     hover ? "var(--color-surface-hover)" : "transparent",
        transition:     "background 0.18s ease",
        position:       "relative",
        cursor:         "pointer",
      }}
    >
      <div style={{ width: "100%", aspectRatio: "1 / 1", borderRadius: 10, overflow: "hidden", position: "relative" }}>
        {playlist.image_url ? (
          <img
            src={playlist.image_url}
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
          lineHeight: 1.3,
        }}
      >
        {playlist.name}
      </p>
      <p
        style={{
          margin: 0,
          fontSize: 12,
          color: "var(--color-text-dim)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {playlist.total_tracks} {playlist.total_tracks === 1 ? "song" : "songs"}
      </p>
    </MotionLink>
  );
}

export default function Playlists() {
  const { loggedIn, login, loggingIn } = useAuth();
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
      <div style={{ display: "flex", flexDirection: "column", gap: "clamp(20px, 2.5vw, 28px)" }}>
        <h1 style={{ margin: 0, fontSize: "clamp(22px, 2.5vw, 26px)", fontWeight: 700, letterSpacing: "-0.02em", color: "var(--color-text-hi)" }}>
          Playlists
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
          title="Sign in to view your playlists"
          description="Log in with Spotify to access and play your saved playlists."
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

  return (
    <motion.div layout="position" style={{ display: "flex", flexDirection: "column", gap: "clamp(20px, 2.5vw, 28px)" }}>
      <motion.div layout="position" style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12 }}>
        <h1 style={{ margin: 0, fontSize: "clamp(22px, 2.5vw, 26px)", fontWeight: 700, letterSpacing: "-0.02em", color: "var(--color-text-hi)" }}>
          Playlists
        </h1>
        <button
          onClick={() => sync()}
          disabled={isPending}
          style={{
            display:     "flex",
            alignItems:  "center",
            gap:         6,
            padding:     "6px 14px",
            borderRadius: 99,
            border:      "1px solid rgba(255,255,255,0.12)",
            background:  "rgba(255,255,255,0.06)",
            color:       isPending ? "rgba(255,255,255,0.35)" : "rgba(255,255,255,0.80)",
            fontSize:    12,
            fontWeight:  500,
            cursor:      isPending ? "default" : "pointer",
            transition:  "background 0.12s",
          }}
          onMouseEnter={(e) => {
            if (!isPending) (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.10)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.06)";
          }}
        >
          <RefreshCw size={12} strokeWidth={2} style={{ animation: isPending ? "spin 1s linear infinite" : "none" }} />
          {isPending ? "Syncing…" : "Sync"}
        </button>
      </motion.div>

      {isLoading && (
        <motion.p layout="position" style={{ fontSize: 13, color: "var(--color-text-dim)" }}>Loading playlists…</motion.p>
      )}

      {!isLoading && playlists.length === 0 && (
        <motion.div layout="position" style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", paddingTop: 80, gap: 12 }}>
          <ListMusic size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />
          <p style={{ margin: 0, fontSize: 15, fontWeight: 600, color: "rgba(255,255,255,0.75)" }}>No playlists yet</p>
          <p style={{ margin: 0, fontSize: 12, color: "var(--color-text-dim)" }}>
            Click Sync to load your playlists from Spotify
          </p>
        </motion.div>
      )}

      {playlists.length > 0 && (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(clamp(130px, 14vw, 170px), 1fr))",
            gap: "clamp(10px, 1.4vw, 16px)",
            width: "100%",
          }}
        >
          {playlists.map((p: PlaylistSummary) => (
            <PlaylistCard key={p.id} playlist={p} onContextMenu={openMenu(cardMenu(p))} />
          ))}
        </div>
      )}
      {menuEl}
    </motion.div>
  );
}
