import { Link, useSearchParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Heart, Users, RefreshCw, Disc3 } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { MusiqueLogo } from "../components/ui/MusiqueLogo";
import {
  useLikedSongs,
  useLikedSongsCount,
  useSavedAlbums,
  useFollowedArtists,
  useSyncLibrary,
  useLibraryStatus,
  useToggleLike,
  useSavedTrackIds,
} from "../hooks/useLibrary";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { TrackRow } from "../components/ui/TrackRow";
import { useSortTools } from "../components/ui/SortToolbar";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };

const TABS = [
  { key: "songs",   label: "Songs"   },
  { key: "albums",  label: "Albums"  },
  { key: "artists", label: "Artists" },
] as const;
type TabKey = (typeof TABS)[number]["key"];

// sync button

function SyncButton() {
  const { mutate, isPending } = useSyncLibrary();
  const { data: status } = useLibraryStatus();

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
      {status?.last_synced && (
        <span style={{ fontSize: 12, color: "rgba(255,255,255,0.30)" }}>
          Synced {new Date(status.last_synced).toLocaleTimeString()}
        </span>
      )}
      <button
        onClick={() => mutate()}
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
  );
}

// liked songs

function LikedSongsTab() {
  const { data: tracks = [], isLoading } = useLikedSongs(200, 0);
  const { data: count = 0 } = useLikedSongsCount();
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue     = useQueueStore((s) => s.enqueue);
  const playContext = useQueueStore((s) => s.playContext);
  const toggleLike  = useToggleLike();

  const { view, toolbar } = useSortTools(tracks, "Find in songs");
  const { data: savedIds = [] } = useSavedTrackIds(tracks.map((t) => t.id));
  const likedSet = new Set(savedIds);

  if (isLoading) {
    return <p style={{ fontSize: 13, color: "rgba(255,255,255,0.40)" }}>Loading liked songs…</p>;
  }

  if (tracks.length === 0) {
    return (
      <EmptyState
        icon={<Heart size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Songs you like will appear here"
        hint="Click Sync to load your liked songs from Spotify"
        action={
          <Link to="/" style={{ padding: "8px 20px", borderRadius: 99, background: "var(--color-accent)", color: "#fff", fontSize: 13, fontWeight: 600, textDecoration: "none", marginTop: 4 }}>
            Find songs
          </Link>
        }
      />
    );
  }

  const handlePlay = (index: number) => {
    const start = playContext(view, index, "liked");
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  };

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        style={{ margin: "0 0 12px", fontSize: 12, color: "rgba(255,255,255,0.35)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
      >
        {count || tracks.length} TRACKS
      </motion.p>
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        {toolbar}
      </motion.div>
      <div style={{ display: "flex", flexDirection: "column" }}>
        {view.map((t, i) => (
          <TrackRow
            key={t.id}
            track={t}
            index={i}
            showAlbum
            liked={likedSet.has(t.id)}
            onPlay={() => handlePlay(i)}
            onQueue={(track) => enqueue(track)}
            onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
          />
        ))}
      </div>
      {view.length === 0 && (
        <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>No songs match your filter.</p>
      )}
    </motion.div>
  );
}

// albums

function AlbumsTab() {
  const { data: albums = [], isLoading } = useSavedAlbums();
  const { view, toolbar } = useSortTools(albums, "Find in albums");

  if (isLoading) {
    return <p style={{ fontSize: 13, color: "rgba(255,255,255,0.40)" }}>Loading albums…</p>;
  }

  if (albums.length === 0) {
    return (
      <EmptyState
        icon={<Disc3 size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Albums you save will appear here"
        hint="Click Sync to load your saved albums from Spotify"
      />
    );
  }

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        style={{ margin: "0 0 12px", fontSize: 12, color: "rgba(255,255,255,0.35)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
      >
        {albums.length} ALBUMS
      </motion.p>
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        {toolbar}
      </motion.div>
      <AlbumGrid>
        {view.map((al) => <AlbumCard key={al.id} album={al} />)}
      </AlbumGrid>
      {view.length === 0 && (
        <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>No albums match your filter.</p>
      )}
    </motion.div>
  );
}

// artists

function ArtistsTab() {
  const { data: artists = [], isLoading } = useFollowedArtists();
  const { view, toolbar } = useSortTools(artists, "Find in artists");

  if (isLoading) {
    return <p style={{ fontSize: 13, color: "rgba(255,255,255,0.40)" }}>Loading artists…</p>;
  }

  if (artists.length === 0) {
    return (
      <EmptyState
        icon={<Users size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Artists you follow will appear here"
        hint="Click Sync to load followed artists from Spotify"
      />
    );
  }

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        style={{ margin: "0 0 12px", fontSize: 12, color: "rgba(255,255,255,0.35)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
      >
        {artists.length} ARTISTS
      </motion.p>
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        {toolbar}
      </motion.div>
      <ArtistGrid>
        {view.map((a) => <ArtistCard key={a.id} artist={a} />)}
      </ArtistGrid>
      {view.length === 0 && (
        <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>No artists match your filter.</p>
      )}
    </motion.div>
  );
}

// library page

export default function Library() {
  const { loggedIn, login, loggingIn } = useAuth();
  const [params, setParams] = useSearchParams();
  const tab = (params.get("tab") as TabKey) || "songs";

  function selectTab(key: TabKey) {
    setParams({ tab: key }, { replace: true });
  }

  if (!loggedIn) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
        <h1 style={{ margin: 0, fontSize: 26, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>
          Your Library
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
          title="Sign in to view your library"
          description="Log in with Spotify to access your saved playlists, albums, and tracks."
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
    <motion.div
      layout="position"
      transition={{ layout: REFLOW }}
      style={{ display: "flex", flexDirection: "column", gap: 18 }}
    >
      <motion.div
        layout="position"
        transition={{ layout: REFLOW }}
        style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12, flexWrap: "wrap", rowGap: 12 }}
      >
        <h1 style={{ margin: 0, fontSize: 26, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>
          Your Library
        </h1>
        <SyncButton />
      </motion.div>

      {/* segmented tabs */}
      <motion.div
        layout="position"
        transition={{ layout: REFLOW }}
        role="tablist"
        aria-label="Library sections"
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 4,
          padding: 3,
          borderRadius: 99,
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          width: "fit-content",
          maxWidth: "100%",
          overflowX: "auto",
        }}
      >
        {TABS.map((t) => {
          const on = tab === t.key;
          return (
            <button
              key={t.key}
              role="tab"
              aria-selected={on}
              onClick={() => selectTab(t.key)}
              style={{
                position: "relative",
                padding: "6px 18px",
                borderRadius: 99,
                border: "none",
                background: "transparent",
                color: on ? "var(--color-text-hi)" : "var(--color-text-dim)",
                fontSize: 13,
                fontWeight: on ? 600 : 500,
                cursor: "pointer",
                transition: "color 0.15s ease",
                zIndex: 1,
                userSelect: "none",
                whiteSpace: "nowrap",
              }}
            >
              {on && (
                <motion.div
                  layoutId="library-tab-indicator"
                  transition={{ type: "spring", stiffness: 400, damping: 34 }}
                  style={{
                    position: "absolute",
                    inset: 0,
                    borderRadius: 99,
                    background: "var(--color-active)",
                    boxShadow: "0 1px 3px rgba(0,0,0,0.25)",
                    zIndex: -1,
                  }}
                />
              )}
              {t.label}
            </button>
          );
        })}
      </motion.div>

      <motion.div
        key={tab}
        layout="position"
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{
          opacity: { duration: 0.26, ease: [0.23, 1, 0.32, 1] },
          y: { duration: 0.26, ease: [0.23, 1, 0.32, 1] },
          layout: REFLOW,
        }}
      >
        {tab === "songs"   && <LikedSongsTab />}
        {tab === "albums"  && <AlbumsTab />}
        {tab === "artists" && <ArtistsTab />}
      </motion.div>
    </motion.div>
  );
}
