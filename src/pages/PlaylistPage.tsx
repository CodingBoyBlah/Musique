import { useMemo } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Pin } from "@/lib/icons";
import { usePlaylist } from "../hooks/usePlaylist";
import { TrackRow } from "../components/ui/TrackRow";
import { PlayActions } from "../components/ui/PlayActions";
import { PageHeader } from "../components/ui/PageHeader";
import { ExpandableDescription } from "../components/ui/ExpandableDescription";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { useTrackTools } from "../components/ui/TrackToolbar";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import { useAuthStore } from "../store/auth.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { useContextMenu } from "../components/ui/ContextMenu";
import { useQueryClient } from "@tanstack/react-query";
import { removeTrackFromPlaylist, addTrackToPlaylist } from "../api/library";
import { toast } from "../store/toast.store";
import type { TrackItem } from "../types/spotify";
import { errMsg } from "../lib/err";
import { useReflowPulse } from "../hooks/useReflowPulse";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 37 };

export default function PlaylistPage() {
  // same as every other page: re-render on panel/resize so rows glide
  useReflowPulse();
  const { id }                     = useParams<{ id: string }>();
  const { data, isLoading, error, refetch } = usePlaylist(id);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const pinned = usePinsStore((s) => Boolean(data?.id && s.pins.some((p) => p.id === data.id)));
  const togglePin = usePinsStore((s) => s.togglePin);
  const toggleLike = useToggleLike();
  const displayName = useAuthStore((s) => s.displayName);
  const qc = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const tracks = data?.tracks ?? [];
  const trackIds = useMemo(() => tracks.map((t) => t.id), [tracks]);
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = useMemo(() => new Set(savedIds), [savedIds]);

  const { view, keys, toolbar } = useTrackTools(tracks, "Playlist order");

  if (isLoading) {
    return <Loader label="Loading playlist" />;
  }
  if (error && !data) {
    return (
      <EmptyState
        title="Couldn't load this playlist"
        description={errMsg(error)}
        action={
          <button type="button" className="btn-pill" onClick={() => refetch()}>
            Try again
          </button>
        }
      />
    );
  }
  if (!data) return null;

  const pinItem = { id: data.id, name: data.name, image_url: data.image_url, type: "playlist" as const };

  // only the owner can remove tracks. owner_name is the only owner signal we get on the frontend, so compare it to the logged-in users display name.
  const isOwner = !!displayName && data.owner_name === displayName;
  /* a slip here is easy (it sits in a row menu next to harmless actions), so
  the toast carries an Undo that puts the track back at its old position,
  rather than a confirm dialog in front of every removal. */
  function removeFromPlaylist(track: TrackItem) {
    const playlistId = data!.id;
    const playlistName = data!.name;
    // Spotify removes every occurrence of the uri, so remember them all.
    // re-inserting in ascending order puts each copy back at its old index.
    const positions = data!.tracks.flatMap((t, i) => (t.id === track.id ? [i] : []));
    const refresh = () => qc.invalidateQueries({ queryKey: ["playlist", playlistId] });
    removeTrackFromPlaylist(playlistId, track.id)
      .then(() => {
        refresh();
        toast(`Removed from ${playlistName}`, {
          action: {
            label: "Undo",
            onClick: () => {
              const restore = positions.length
                ? positions.reduce<Promise<void>>(
                    (p, pos) => p.then(() => addTrackToPlaylist(playlistId, track.id, pos)),
                    Promise.resolve(),
                  )
                : addTrackToPlaylist(playlistId, track.id);
              restore
                .then(() => {
                  refresh();
                  toast(`Restored to ${playlistName}`);
                })
                .catch((e) => toast.error(`Couldn't restore track: ${errMsg(e)}`));
            },
          },
        });
      })
      .catch((e) => toast.error(`Couldn't remove track: ${errMsg(e)}`));
  }

  // play within whatever order's on screen right now (filtered/sorted view)
  function startAt(index: number) {
    const start = playContext(view, index, data!.id);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
      if (data) {
        useSpeedDialStore.getState().recordPlaylist({ id: data.id, name: data.name, image_url: data.image_url });
      }
    }
  }

  return (
    <div
      className="flex flex-col"
      onContextMenu={openMenu([
        { label: pinned ? "Unpin from sidebar" : "Pin to sidebar", icon: <Pin size={14} active={pinned} />, onSelect: () => togglePin(pinItem) },
      ])}
    >
      <PageHeader imageUrl={data.image_url} eyebrow="Playlist" title={data.name}>
        {data.description && (
          <ExpandableDescription text={data.description} />
        )}
        <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>
          {data.owner_name && <>{data.owner_name} · </>}
          <span className="tnum" style={{ textTransform: "uppercase", letterSpacing: "0.05em", fontWeight: 600 }}>
            {data.total_tracks} {data.total_tracks === 1 ? "track" : "tracks"}
          </span>
        </p>
        <PlayActions tracks={tracks} contextId={data.id} pinItem={pinItem} />
      </PageHeader>

      <section>
        {toolbar}
        <motion.div
          layout="position"
          transition={{ layout: REFLOW }}
          className="flex flex-col"
        >
          {view.map((t, i) => (
            <TrackRow
              key={keys[i]}
              track={t}
              index={i}
              showAlbum
              liked={likedSet.has(t.id)}
              onPlay={() => startAt(i)}
              onQueue={(track) => enqueue(track)}
              onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
              onRemoveFromPlaylist={isOwner ? removeFromPlaylist : undefined}
            />
          ))}
          {view.length === 0 && (
            <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>
              No tracks match your filter.
            </p>
          )}
        </motion.div>
      </section>
      {menuEl}
    </div>
  );
}
