import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Pin } from "lucide-react";
import { usePlaylist } from "../hooks/usePlaylist";
import { TrackRow } from "../components/ui/TrackRow";
import { PlayActions } from "../components/ui/PlayActions";
import { PageHeader } from "../components/ui/PageHeader";
import { Loader } from "../components/ui/Loader";
import { useTrackTools } from "../components/ui/TrackToolbar";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useAuthStore } from "../store/auth.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { useContextMenu } from "../components/ui/ContextMenu";
import { useQueryClient } from "@tanstack/react-query";
import { removeTrackFromPlaylist } from "../api/library";
import { toast } from "../store/toast.store";
import type { TrackItem } from "../types/spotify";
import { errMsg } from "../lib/err";

export default function PlaylistPage() {
  const { id }                     = useParams<{ id: string }>();
  const { data, isLoading, error } = usePlaylist(id);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const pins      = usePinsStore((s) => s.pins);
  const togglePin = usePinsStore((s) => s.togglePin);
  const toggleLike = useToggleLike();
  const displayName = useAuthStore((s) => s.displayName);
  const qc = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const tracks = data?.tracks ?? [];
  const { data: savedIds = [] } = useSavedTrackIds(tracks.map((t) => t.id));
  const likedSet = new Set(savedIds);

  const { view, keys, toolbar } = useTrackTools(tracks, "Playlist order");

  if (isLoading) {
    return <Loader label="Loading playlist" />;
  }
  if (error) {
    return <p className="text-sm" style={{ color: "var(--color-danger)" }}>{errMsg(error)}</p>;
  }
  if (!data) return null;

  const pinned  = pins.some((p) => p.id === data.id);
  const pinItem = { id: data.id, name: data.name, image_url: data.image_url, type: "playlist" as const };

  // only the owner can remove tracks. owner_name is the only owner signal we get on the frontend, so compare it to the logged-in users display name.
  const isOwner = !!displayName && data.owner_name === displayName;
  function removeFromPlaylist(track: TrackItem) {
    removeTrackFromPlaylist(data!.id, track.id)
      .then(() => {
        toast(`Removed from ${data!.name}`);
        qc.invalidateQueries({ queryKey: ["playlist", data!.id] });
      })
      .catch(() => toast("Couldn't remove track"));
  }

  // play within whatever order's on screen right now (filtered/sorted view)
  function startAt(index: number) {
    const start = playContext(view, index, data!.id);
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(console.error); }
  }

  return (
    <div
      className="flex flex-col"
      onContextMenu={openMenu([
        { label: pinned ? "Unpin from sidebar" : "Pin to sidebar", icon: <Pin size={14} />, onSelect: () => togglePin(pinItem) },
      ])}
    >
      <PageHeader imageUrl={data.image_url} eyebrow="Playlist" title={data.name}>
        {data.description && (
          <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>{data.description}</p>
        )}
        <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>
          {data.owner_name && <>{data.owner_name} · </>}
          <span style={{ textTransform: "uppercase", letterSpacing: "0.05em", fontWeight: 600 }}>
            {data.total_tracks} {data.total_tracks === 1 ? "track" : "tracks"}
          </span>
        </p>
        <PlayActions tracks={tracks} contextId={data.id} pinItem={pinItem} />
      </PageHeader>

      <section>
        {toolbar}
        {view.map((t, i) => (
          <motion.div
            key={keys[i]}
            layout="position"
            transition={{ type: "spring", stiffness: 520, damping: 42, mass: 0.7 }}
          >
            <TrackRow
              track={t}
              index={i}
              showAlbum
              liked={likedSet.has(t.id)}
              onPlay={() => startAt(i)}
              onQueue={(track) => enqueue(track)}
              onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
              onRemoveFromPlaylist={isOwner ? removeFromPlaylist : undefined}
            />
          </motion.div>
        ))}
        {view.length === 0 && (
          <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>
            No tracks match your filter.
          </p>
        )}
      </section>
      {menuEl}
    </div>
  );
}
