import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { ListMusic, RefreshCw, Pin, PinOff } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import { useMyPlaylists, useSyncLibrary } from "../hooks/useLibrary";
import { usePinsStore } from "../store/pins.store";
import { useContextMenu, type MenuEntry } from "../components/ui/ContextMenu";
import type { PlaylistSummary } from "../types/library";

function PlaylistCard({
  playlist, onContextMenu,
}: {
  playlist: PlaylistSummary;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  return (
    <Link
      to={`/playlist/${playlist.id}`}
      onContextMenu={onContextMenu}
      style={{
        display:       "flex",
        flexDirection: "column",
        gap:           8,
        padding:       12,
        borderRadius:  10,
        width:         156,
        textDecoration: "none",
        color:         "inherit",
        transition:    "background 0.12s",
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "var(--color-surface-hover)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLAnchorElement).style.background = "transparent"; }}
    >
      {playlist.image_url ? (
        <img
          src={playlist.image_url}
          alt={playlist.name}
          style={{ width: 132, height: 132, objectFit: "cover", borderRadius: 6 }}
        />
      ) : (
        <div style={{ width: 132, height: 132, borderRadius: 6, background: "rgba(124,111,255,0.18)", display: "flex", alignItems: "center", justifyContent: "center" }}>
          <ListMusic size={40} strokeWidth={1.5} style={{ color: "rgba(124,111,255,0.55)" }} />
        </div>
      )}
      <p style={{ margin: 0, fontSize: 13, fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {playlist.name}
      </p>
      <p style={{ margin: 0, fontSize: 12, color: "rgba(255,255,255,0.40)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {playlist.total_tracks} {playlist.total_tracks === 1 ? "song" : "songs"}
      </p>
    </Link>
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
      <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, color: "rgba(255,255,255,0.92)" }}>Playlists</h1>
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
    <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, color: "rgba(255,255,255,0.92)" }}>Playlists</h1>
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
      </div>

      {isLoading && (
        <p style={{ fontSize: 13, color: "rgba(255,255,255,0.40)" }}>Loading playlists…</p>
      )}

      {!isLoading && playlists.length === 0 && (
        <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", paddingTop: 80, gap: 12 }}>
          <ListMusic size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />
          <p style={{ margin: 0, fontSize: 15, fontWeight: 600, color: "rgba(255,255,255,0.75)" }}>No playlists yet</p>
          <p style={{ margin: 0, fontSize: 12, color: "rgba(255,255,255,0.28)" }}>
            Click Sync to load your playlists from Spotify
          </p>
        </div>
      )}

      {playlists.length > 0 && (
        <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
          {playlists.map((p: PlaylistSummary) => (
            <PlaylistCard key={p.id} playlist={p} onContextMenu={openMenu(cardMenu(p))} />
          ))}
        </div>
      )}
      {menuEl}
    </div>
  );
}
