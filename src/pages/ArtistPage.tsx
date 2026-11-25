import { useState, useMemo } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { Check, UserPlus, Link2, Globe, Radio } from "@/lib/icons";
import { playStation } from "../utils/radio";
import { useArtist } from "../hooks/useArtist";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getArtistExtras, getArtistOverview } from "../api/internal";
import { MediaTile } from "../components/ui/MediaTile";
import { ArtistTour } from "../components/ui/ArtistTour";
import { ArtistAbout } from "../components/ui/ArtistAbout";
import { AlbumCard } from "../components/ui/AlbumCard";
import { ArtistCard } from "../components/ui/ArtistCard";
import { TrackRow, useTrackRowActions } from "../components/ui/TrackRow";
import { PageHeader } from "../components/ui/PageHeader";
import { CoverArt } from "../components/ui/CoverArt";
import { PlayActions } from "../components/ui/PlayActions";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { SectionTitle, ShowAllButton } from "../components/ui/SectionTitle";
import { Shelf } from "../components/ui/Shelf";
import { useContextMenu } from "../components/ui/ContextMenu";
import { Tooltip } from "../components/ui/Tooltip";
import { serviceLinkLabel, shareSpotifyLink, shareUniversalLink } from "../lib/share";
import {
  useIsArtistFollowed,
  useToggleFollow,
  useSavedTrackIds,
  useToggleLike,
} from "../hooks/useLibrary";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { useSpeedDialStore } from "../store/speedDial.store";
import { playTrack } from "../api/playback";
import { EASE_OUT, PRESS, PRESS_TRANSITION, REFLOW_SPRING } from "../lib/motion";
import { errMsg } from "../lib/err";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { isUpcoming, releaseDateMs } from "../utils/fmt";
import { prefetchAlbum } from "../lib/prefetch";
import type { AlbumItem } from "../types/spotify";
import "../styles/artist.css";

const TOP_TRACKS_COLLAPSED = 5;

export default function ArtistPage() {
  useReflowPulse();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useArtist(id);
  const { data: following = false } = useIsArtistFollowed(id);
  // bio / fans also like / appears on come from spotify's internal metadata,
  // separately, so the page never waits on them
  const { data: extras } = useQuery({
    queryKey: ["artist-extras", id],
    queryFn: () => getArtistExtras(id!),
    enabled: !!id,
    staleTime: 30 * 60_000,
    retry: false,
  });
  const { data: overview } = useQuery({
    queryKey: ["artist-overview", id],
    queryFn: () => getArtistOverview(id!),
    enabled: !!id,
    staleTime: 30 * 60_000,
    retry: false,
  });
  const toggleFollow = useToggleFollow();

  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const enqueue = useQueueStore((s) => s.enqueue);
  const playContext = useQueueStore((s) => s.playContext);
  const toggleLike = useToggleLike();

  const [showAllTop, setShowAllTop] = useState(false);
  const { open: openMenu, element: menuEl } = useContextMenu();

  const topTracks = data?.top_tracks ?? [];
  const topTrackIds = useMemo(() => topTracks.map((t) => t.id), [topTracks]);
  const { data: savedIds = [] } = useSavedTrackIds(topTrackIds);
  const likedSet = useMemo(() => new Set(savedIds), [savedIds]);
  const trackActions = useTrackRowActions(likedSet, toggleLike);
  // the newest release that is already out (an announced one shows its
  // countdown in the Albums / Singles shelves instead)
  const latest = useMemo(() => {
    let best: AlbumItem | null = null;
    let bestT = -Infinity;
    for (const al of [...(data?.albums ?? []), ...(data?.singles ?? [])]) {
      const t = releaseDateMs(al.release_date);
      if (t == null || isUpcoming(al.release_date)) continue;
      if (t > bestT) { best = al; bestT = t; }
    }
    return best;
  }, [data?.albums, data?.singles]);
  const galleryImages = useMemo(
    () =>
      [...(overview?.gallery ?? []), ...(extras?.gallery ?? []), ...(data?.image_url ? [data.image_url] : [])]
        .filter((v, i, arr) => arr.indexOf(v) === i),
    [overview?.gallery, extras?.gallery, data?.image_url],
  );

  if (isLoading) return <Loader label="Loading artist" />;

  if (error) {
    return (
      <EmptyState
        title="Couldn't load this artist"
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

  // one context id for the header's Play and the rows, so Play reads "Pause"
  // whichever of them started the music
  const contextId = `artist-top-${data.id}`;
  const artistItem = { id: data.id, name: data.name, image_url: data.image_url, type: "artist" as const };
  const shownTop = showAllTop ? topTracks : topTracks.slice(0, TOP_TRACKS_COLLAPSED);
  const genres = data.genres.slice(0, 3);

  const openRadio = (playlistId: string) => navigate(`/playlist/${playlistId}`);
  const shareEntries = [
    { label: "Artist radio", icon: <Radio size={14} />, onSelect: () => { playStation(`spotify:artist:${data.id}`, data.name, openRadio); } },
    { label: serviceLinkLabel(), icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("artist", data.id) },
    { label: "Copy universal link", icon: <Globe size={14} />, onSelect: () => shareUniversalLink("artist", data.id) },
  ];

  // a row plays from itself onward through the whole top list, not just the
  // five on screen - the queue keeps going the way the header's Play would
  function startTop(index: number) {
    const start = playContext(topTracks, index, contextId, `spotify:artist:${data!.id}`);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
      useSpeedDialStore.getState().recordArtist({ id: data!.id, name: data!.name, image_url: data!.image_url });
    }
  }
  trackActions.setHandlers(startTop);

  const followButton = (
    <Tooltip label={following ? "Unfollow artist" : "Follow artist"} side="top">
      <motion.button
        type="button"
        onClick={() => toggleFollow.mutate({ id: data.id, following })}
        aria-pressed={following}
        className="ghost-pill focus-ring"
        data-on={following}
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        style={{
          height: 36,
          padding: "0 16px",
          borderRadius: 99,
          color: "#ffffff",
          fontSize: 13,
          fontWeight: 600,
          display: "flex",
          alignItems: "center",
          gap: 6,
          cursor: "pointer",
          flexShrink: 0,
        }}
      >
        {following ? <Check size={14} strokeWidth={2.4} /> : <UserPlus size={14} strokeWidth={2.2} />}
        <span>{following ? "Following" : "Follow"}</span>
      </motion.button>
    </Tooltip>
  );

  return (
    <div className="flex flex-col" onContextMenu={openMenu(shareEntries)}>
      {/* the same header as every album, playlist and profile page: the
          portrait, the eyebrow, the title, its meta line and the buttons */}
      <PageHeader round imageUrl={data.image_url} eyebrow={overview?.verified ? "Verified artist" : "Artist"} title={data.name}>
        {(overview?.monthly_listeners != null || genres.length > 0) && (
          <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 4, flexWrap: "wrap", fontSize: 14.5 }}>
            {overview?.monthly_listeners != null && (
              <span className="tnum" style={{ fontWeight: 700, color: "#ffffff" }}>
                {overview.monthly_listeners.toLocaleString()} monthly listeners
              </span>
            )}
            {genres.map((g, i) => (
              <span key={g} style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
                {(i > 0 || overview?.monthly_listeners != null) && (
                  <span aria-hidden style={{ color: "rgba(255, 255, 255, 0.35)", fontSize: 10, userSelect: "none" }}>•</span>
                )}
                <span style={{ fontWeight: 500, color: "rgba(255, 255, 255, 0.65)", textTransform: "capitalize" }}>{g}</span>
              </span>
            ))}
          </div>
        )}
        <PlayActions
          tracks={topTracks}
          contextId={contextId}
          pinItem={artistItem}
          accessory={
            <>
              {followButton}
              <Tooltip label="Artist radio" side="top">
                <motion.button
                  type="button"
                  aria-label={`${data.name} radio`}
                  className="ghost-pill focus-ring"
                  onClick={() => playStation(`spotify:artist:${data.id}`, data.name, openRadio)}
                  whileTap={PRESS}
                  transition={PRESS_TRANSITION}
                  style={{ height: 36, width: 36, borderRadius: 99, color: "#ffffff", display: "flex", alignItems: "center", justifyContent: "center", cursor: "pointer", flexShrink: 0 }}
                >
                  <Radio size={15} strokeWidth={2.1} />
                </motion.button>
              </Tooltip>
            </>
          }
        />
      </PageHeader>

      <div style={{ display: "flex", flexDirection: "column", gap: "clamp(28px, 4vw, 44px)", paddingTop: 8 }}>
        {(topTracks.length > 0 || latest) && (
          <div className="artist-top">
            <div className="artist-top__grid" data-latest={!!latest && topTracks.length > 0}>
              {latest && (
                <section aria-labelledby="artist-latest">
                  <SectionTitle id="artist-latest">Latest release</SectionTitle>
                  <LatestRelease album={latest} />
                </section>
              )}
              {topTracks.length > 0 && (
                <section aria-labelledby="artist-popular" style={{ minWidth: 0 }}>
                  <SectionTitle
                    id="artist-popular"
                    right={
                      topTracks.length > TOP_TRACKS_COLLAPSED && (
                        <ShowAllButton
                          onClick={() => setShowAllTop((v) => !v)}
                          label={showAllTop ? "Show less" : "Show all"}
                        />
                      )
                    }
                  >
                    Popular
                  </SectionTitle>
                  {/* rows pull out by their own 12px padding so the numbers
                      line up with the section title instead of indenting */}
                  <div style={{ margin: "0 -12px" }}>
                    {shownTop.map((t, i) => (
                      <motion.div
                        key={t.id}
                        layout="position"
                        initial={{ opacity: 0, y: 8 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{
                          layout: REFLOW_SPRING,
                          duration: 0.28,
                          delay: Math.min(i % TOP_TRACKS_COLLAPSED, 6) * 0.03,
                          ease: EASE_OUT,
                        }}
                        style={{ position: "relative" }}
                        whileHover={{ zIndex: 40 }}
                      >
                        <TrackRow
                          track={t}
                          index={i}
                          showAlbum
                          striped={false}
                          liked={likedSet.has(t.id)}
                          onPlay={trackActions.onPlay}
                          onQueue={enqueue}
                          onToggleLike={trackActions.onToggleLike}
                        />
                      </motion.div>
                    ))}
                  </div>
                </section>
              )}
            </div>
          </div>
        )}

        <Shelf
          id="artist-albums"
          title="Albums"
          items={data.albums}
          getKey={(al) => al.id}
          renderItem={(al, i) => <AlbumCard album={al} index={i} />}
        />
        <Shelf
          id="artist-singles"
          title="Singles & EPs"
          items={data.singles}
          getKey={(al) => al.id}
          renderItem={(al, i) => <AlbumCard album={al} index={i} />}
        />
        <Shelf
          id="artist-appears-on"
          title="Appears on"
          items={extras?.appears_on ?? []}
          getKey={(al) => al.id}
          renderItem={(al, i) => <AlbumCard album={al} index={i} />}
        />
        <Shelf
          id="artist-featuring"
          title={`Featuring ${data.name}`}
          items={overview?.featuring ?? []}
          getKey={(p) => p.id}
          renderItem={(p, i) => <MediaTile to={`/playlist/${p.id}`} imageUrl={p.image_url} title={p.name} subtitle={p.subtitle} index={i} />}
        />
        <Shelf
          id="artist-discovered-on"
          title="Discovered on"
          items={overview?.discovered_on ?? []}
          getKey={(p) => p.id}
          renderItem={(p, i) => <MediaTile to={`/playlist/${p.id}`} imageUrl={p.image_url} title={p.name} subtitle={p.subtitle} index={i} />}
        />
        <Shelf
          id="artist-related"
          title="Fans also like"
          items={data.related_artists?.length ? data.related_artists : extras?.related ?? []}
          getKey={(ar) => ar.id}
          renderItem={(ar, i) => <ArtistCard artist={ar} index={i} />}
        />
        <ArtistAbout
          name={data.name}
          biography={extras?.biography ?? null}
          images={galleryImages}
          stats={{
            monthlyListeners: overview?.monthly_listeners,
            followers: overview?.followers,
            worldRank: overview?.world_rank,
            activeYears: extras?.active_years,
          }}
          cities={overview?.top_cities}
          links={overview?.external_links}
        />
        <ArtistTour concerts={overview?.concerts ?? []} merch={overview?.merch ?? []} />
      </div>
      {menuEl}
    </div>
  );
}

// "14 Nov 2025" when Spotify gives the day, "2025" when it only gives the year
function fmtRelease(date: string): string {
  const t = releaseDateMs(date);
  if (t == null) return date;
  const parts = date.split("-").length;
  const opts: Intl.DateTimeFormatOptions =
    parts >= 3 ? { day: "numeric", month: "short", year: "numeric" }
    : parts === 2 ? { month: "short", year: "numeric" }
    : { year: "numeric" };
  return new Intl.DateTimeFormat(undefined, opts).format(t);
}

function LatestRelease({ album }: { album: AlbumItem }) {
  const qc = useQueryClient();
  return (
    <Link
      to={`/album/${album.id}`}
      className="latest-release pressable focus-ring"
      onPointerEnter={() => prefetchAlbum(qc, album.id)}
      onFocus={() => prefetchAlbum(qc, album.id)}
    >
      <div className="latest-release__cover">
        <CoverArt
          url={album.image_url}
          alt={album.name}
          size={260}
          style={{ width: "100%", height: "100%", borderRadius: 0 }}
        />
      </div>
      <div className="latest-release__text">
        {album.release_date && <p className="latest-release__date">{fmtRelease(album.release_date)}</p>}
        <p className="latest-release__name" title={album.name}>{album.name}</p>
        <p className="latest-release__type">{album.album_type}</p>
      </div>
    </Link>
  );
}
