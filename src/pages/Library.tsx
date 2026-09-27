import { useMemo } from "react";
import { useSearchParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Heart, Users, Disc3, Mic } from "@/lib/icons";
import { useQuery } from "@tanstack/react-query";
import { getSavedShows } from "../api/podcasts";
import { MediaTile } from "../components/ui/MediaTile";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { SignInPrompt } from "../components/ui/SignInPrompt";
import { TrackRowsSkeleton, CardGridSkeleton } from "../components/ui/Skeletons";
import { SyncButton } from "../components/ui/SyncButton";
import {
  useLikedSongs,
  useLikedSongsCount,
  useSavedAlbums,
  useFollowedArtists,
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
import { useSpeedDialStore } from "../store/speedDial.store";
import { useReflowPulse } from "../hooks/useReflowPulse";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 37 };

const TAB_KEYS = ["songs", "albums", "artists", "podcasts"] as const;
type TabKey = (typeof TAB_KEYS)[number];

// liked songs

function LikedSongsTab() {
  const { data: tracks = [], isLoading } = useLikedSongs(200, 0);
  const { data: count = 0 } = useLikedSongsCount();
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue     = useQueueStore((s) => s.enqueue);
  const playContext = useQueueStore((s) => s.playContext);
  const toggleLike  = useToggleLike();

  const { view, toolbar } = useSortTools(tracks, "Find in songs");
  const trackIds = useMemo(() => tracks.map((t) => t.id), [tracks]);
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = useMemo(() => new Set(savedIds), [savedIds]);

  if (isLoading) {
    return <div role="status" aria-label="Loading liked songs"><TrackRowsSkeleton count={10} /></div>;
  }

  if (tracks.length === 0) {
    return (
      <EmptyState
        icon={<Heart size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Songs you like will appear here"
        description="Like a song anywhere in the app, or pull in the ones you've already liked on Spotify."
        action={<SyncButton showStatus={false} prominent />}
      />
    );
  }

  const handlePlay = (index: number) => {
    const start = playContext(view, index, "liked");
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
      useSpeedDialStore.getState().recordLikedSongs();
    }
  };

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        className="tnum"
        style={{ margin: "0 0 12px", fontSize: 12, color: "var(--color-text-dim)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
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
    return <div role="status" aria-label="Loading albums"><CardGridSkeleton /></div>;
  }

  if (albums.length === 0) {
    return (
      <EmptyState
        icon={<Disc3 size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Albums you save will appear here"
        description="Pull in the albums you've saved on Spotify."
        action={<SyncButton showStatus={false} prominent />}
      />
    );
  }

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        className="tnum"
        style={{ margin: "0 0 12px", fontSize: 12, color: "var(--color-text-dim)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
      >
        {albums.length} ALBUMS
      </motion.p>
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        {toolbar}
      </motion.div>
      <AlbumGrid>
        {view.map((al, i) => <AlbumCard key={al.id} album={al} index={i} />)}
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
    return <div role="status" aria-label="Loading artists"><CardGridSkeleton round /></div>;
  }

  if (artists.length === 0) {
    return (
      <EmptyState
        icon={<Users size={44} strokeWidth={1.5} style={{ color: "rgba(255,255,255,0.18)" }} />}
        title="Artists you follow will appear here"
        description="Pull in the artists you follow on Spotify."
        action={<SyncButton showStatus={false} prominent />}
      />
    );
  }

  return (
    <motion.div layout="position" transition={{ layout: REFLOW }}>
      <motion.p
        layout="position"
        transition={{ layout: REFLOW }}
        className="tnum"
        style={{ margin: "0 0 12px", fontSize: 12, color: "var(--color-text-dim)", textTransform: "uppercase", letterSpacing: "0.06em", fontWeight: 600 }}
      >
        {artists.length} ARTISTS
      </motion.p>
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        {toolbar}
      </motion.div>
      <ArtistGrid>
        {view.map((a, i) => <ArtistCard key={a.id} artist={a} index={i} />)}
      </ArtistGrid>
      {view.length === 0 && (
        <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>No artists match your filter.</p>
      )}
    </motion.div>
  );
}

// library page

// shows you follow
function PodcastsTab() {
  const { data: shows = [], isLoading } = useQuery({
    queryKey: ["library", "shows"],
    queryFn: getSavedShows,
    staleTime: 300_000,
  });
  if (isLoading) return <CardGridSkeleton count={10} />;
  if (shows.length === 0) {
    return (
      <EmptyState
        icon={<Mic size={22} />}
        title="No podcasts yet"
        description="Follow a show and it lands here. Search finds podcasts too."
      />
    );
  }
  return (
    <AlbumGrid>
      {shows.map((s, i) => (
        <MediaTile key={s.id} to={`/show/${s.id}`} imageUrl={s.image_url} title={s.name} subtitle={s.publisher} index={i} />
      ))}
    </AlbumGrid>
  );
}

export default function Library() {
  useReflowPulse();
  const { loggedIn } = useAuth();
  const [params] = useSearchParams();
  const rawTab = params.get("tab");
  const tab: TabKey = (TAB_KEYS as readonly string[]).includes(rawTab ?? "") ? (rawTab as TabKey) : "songs";

  if (!loggedIn) {
    return (
      <SignInPrompt
        heading="Your library"
        title="Sign in to view your library"
        description="Log in with Spotify to access your saved playlists, albums, and tracks."
      />
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
        <h1 className="t-title" style={{ margin: 0, color: "var(--color-text-hi)" }}>
          Your library
        </h1>
        <SyncButton />
      </motion.div>

      {/* tab switches are frequent: a quick cross-fade only, no slide - motion
          on every switch reads as latency */}
      <motion.div
        key={tab}
        layout="position"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{
          opacity: { duration: 0.15, ease: [0.23, 1, 0.32, 1] },
          layout: REFLOW,
        }}
      >
        {tab === "songs"   && <LikedSongsTab />}
        {tab === "albums"  && <AlbumsTab />}
        {tab === "artists" && <ArtistsTab />}
        {tab === "podcasts" && <PodcastsTab />}
      </motion.div>
    </motion.div>
  );
}
