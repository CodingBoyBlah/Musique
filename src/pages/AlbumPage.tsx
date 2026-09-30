import { useMemo } from "react";
import { coverUrl } from "../lib/coverUrl";
import { useParams, Link } from "react-router-dom";
import { motion } from "framer-motion";
import { Pin, Radio } from "@/lib/icons";
import { playStation } from "../utils/radio";
import { useAlbum } from "../hooks/useAlbum";
import { useArtist } from "../hooks/useArtist";
import { AlbumCard } from "../components/ui/AlbumCard";
import { TrackRow } from "../components/ui/TrackRow";
import { PlayActions } from "../components/ui/PlayActions";
import { SaveAlbumButton } from "../components/ui/SaveAlbumButton";
import { PageHeader } from "../components/ui/PageHeader";
import { ExpandableDescription } from "../components/ui/ExpandableDescription";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { SectionTitle } from "../components/ui/SectionTitle";
import { useCarousel, CarouselControls, CarouselTrack } from "../components/ui/Carousel";
import { useTrackTools } from "../components/ui/TrackToolbar";
import { releaseYear } from "../utils/fmt";
import { playTrack } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { usePinsStore } from "../store/pins.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { useContextMenu } from "../components/ui/ContextMenu";
import { errMsg } from "../lib/err";
import { useReflowPulse } from "../hooks/useReflowPulse";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 37 };
const MORE_BY_TILE = "clamp(140px, 16vw, 175px)";

export default function AlbumPage() {
  useReflowPulse();
  const { id }                     = useParams<{ id: string }>();
  const { data, isLoading, error, refetch } = useAlbum(id);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const pinned = usePinsStore((s) => Boolean(data?.id && s.pins.some((p) => p.id === data.id)));
  const togglePin = usePinsStore((s) => s.togglePin);
  const toggleLike = useToggleLike();
  const { open: openMenu, element: menuEl } = useContextMenu();
  const { data: artistDetail } = useArtist(data?.artists[0]?.id);
  const moreBy = useMemo(
    () => (artistDetail?.albums ?? []).filter((a) => a.id !== data?.id).slice(0, 10),
    [artistDetail?.albums, data?.id],
  );
  const carousel = useCarousel([moreBy.length]);

  const tracks = data?.tracks ?? [];
  const trackIds = useMemo(() => tracks.map((t) => t.id), [tracks]);
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = useMemo(() => new Set(savedIds), [savedIds]);

  const { view, keys, toolbar } = useTrackTools(tracks, "Album order");

  if (isLoading) {
    return <Loader label="Loading album" />;
  }
  if (error) {
    return (
      <EmptyState
        title="Couldn't load this album"
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

  const pinItem = { id: data.id, name: data.name, image_url: data.image_url, type: "album" as const };

  function startAt(index: number) {
    const start = playContext(view, index, data!.id, `spotify:album:${data!.id}`);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
      if (data) {
        useSpeedDialStore.getState().recordAlbum({ id: data.id, name: data.name, image_url: data.image_url });
      }
    }
  }

  return (
    <div
      className="flex flex-col"
      onContextMenu={openMenu([
        { label: pinned ? "Unpin from sidebar" : "Pin to sidebar", icon: <Pin size={14} active={pinned} />, onSelect: () => togglePin(pinItem) },
        { label: "Album radio", icon: <Radio size={14} />, onSelect: () => { playStation(`spotify:album:${data.id}`, data.name); } },
      ])}
    >
      <PageHeader imageUrl={data.image_url} eyebrow={data.album_type} title={data.name}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4, flexWrap: "wrap", fontSize: 14.5 }}>
          {artistDetail?.image_url && (
            <img
              src={coverUrl(artistDetail.image_url, 22) ?? artistDetail.image_url}
              alt=""
              style={{ width: 22, height: 22, borderRadius: "50%", objectFit: "cover", flexShrink: 0 }}
            />
          )}
          <div style={{ display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
            <span style={{ fontWeight: 700, color: "#ffffff" }}>
              {data.artists.map((a, i) => (
                <span key={a.id}>
                  {i > 0 && ", "}
                  <Link to={`/artist/${a.id}`} style={{ color: "inherit", textDecoration: "none" }} className="hover:underline focus-ring">
                    {a.name}
                  </Link>
                </span>
              ))}
            </span>
            {releaseYear(data.release_date) && (
              <span style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                <span style={{ color: "rgba(255, 255, 255, 0.35)", fontSize: 10, userSelect: "none" }}>•</span>
                <span style={{ fontWeight: 500, color: "rgba(255, 255, 255, 0.65)" }}>
                  {releaseYear(data.release_date)}
                </span>
              </span>
            )}
          </div>
        </div>
        {data.description && (
          <ExpandableDescription text={data.description} />
        )}
        <PlayActions tracks={tracks} contextId={data.id} pinItem={pinItem} accessory={<SaveAlbumButton id={data.id} />} />
      </PageHeader>

      <section>
        {toolbar}
        <motion.div
          layout="position"
          transition={{ layout: REFLOW }}
          className="flex flex-col"
        >
          {view.map((t, i) => (
            <motion.div
              key={keys[i]}
              layout="position"
              transition={{ layout: REFLOW }}
              style={{ position: "relative" }}
              whileHover={{ zIndex: 40 }}
            >
              <TrackRow
                track={t}
                index={i}
                showCover={false}
                liked={likedSet.has(t.id)}
                onPlay={() => startAt(i)}
                onQueue={(track) => enqueue(track)}
                onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
              />
            </motion.div>
          ))}
          {view.length === 0 && (
            <p className="text-sm" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>
              No tracks match your filter.
            </p>
          )}
        </motion.div>
      </section>

      {/* release metadata footer (Cider Screenshot 5) */}
      <div className="t-caption tnum" style={{ padding: "28px 4px 16px", display: "flex", flexDirection: "column", gap: 4, fontSize: 12, color: "var(--color-text-dim)" }}>
        <p style={{ margin: 0, fontWeight: 500 }}>
          {data.release_date} • {data.total_tracks} {data.total_tracks === 1 ? "track" : "tracks"}
          {tracks.length > 0 && (() => {
            const totalMin = Math.round(tracks.reduce((sum, t) => sum + (t.duration_ms || 0), 0) / 60000);
            return `, ${totalMin} minutes`;
          })()}
        </p>
        <p style={{ margin: 0, fontSize: 11, opacity: 0.7 }}>
          ℗ {releaseYear(data.release_date)} {data.artists.map((a) => a.name).join(", ")}
        </p>
      </div>

      {/* More by this artist: the same shelf Home uses */}
      {artistDetail && moreBy.length > 0 && (
        <section style={{ marginTop: 28 }} aria-labelledby="album-more-by">
          <SectionTitle
            id="album-more-by"
            right={<CarouselControls carousel={carousel} label={`more by ${artistDetail.name}`} />}
          >
            <Link to={`/artist/${artistDetail.id}`} style={{ color: "inherit", textDecoration: "none" }} className="hover:underline focus-ring">
              More by {artistDetail.name} &rsaquo;
            </Link>
          </SectionTitle>
          <CarouselTrack
            carousel={carousel}
            label={`More by ${artistDetail.name}`}
            items={moreBy}
            getKey={(al) => al.id}
            itemWidth={MORE_BY_TILE}
            renderItem={(al, i) => <AlbumCard album={al} index={i} />}
          />
        </section>
      )}
      {menuEl}
    </div>
  );
}
