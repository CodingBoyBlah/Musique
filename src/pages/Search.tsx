import { useState, useMemo, useEffect, memo } from "react";
import { Link, Navigate, useSearchParams, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useSearch } from "../hooks/useSearch";
import { useAuth } from "../hooks/useAuth";
import { EmptyState } from "../components/ui/EmptyState";
import { SignInPrompt } from "../components/ui/SignInPrompt";
import { SectionTitle, ShowAllButton } from "../components/ui/SectionTitle";
import { TopResultSkeleton, TrackRowsSkeleton, CardGridSkeleton } from "../components/ui/Skeletons";
import { AlbumCard, AlbumGrid } from "../components/ui/AlbumCard";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { Shelf } from "../components/ui/Shelf";
import { CoverArt } from "../components/ui/CoverArt";
import { MediaTile } from "../components/ui/MediaTile";
import { EpisodeRow } from "../components/ui/EpisodeRow";
import { usePlayEpisodes } from "../hooks/usePlayEpisodes";
import { EPISODE_PREFIX } from "../utils/episode";
import type { EpisodeItem } from "../types/podcast";
import { TrackRow } from "../components/ui/TrackRow";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { AnimatedPlayPause } from "../components/playground/AnimatedIcons";
import { getArtist, getAlbum } from "../api/spotify";
import { playTrack } from "../api/playback";
import { transportPlay, transportPause } from "../hooks/usePlayerControls";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { useUIStore } from "../store/ui.store";
import { useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { coverUrl } from "../lib/coverUrl";
import { releaseYear } from "../utils/fmt";
import { errMsg } from "../lib/err";
import { toast } from "../store/toast.store";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { EASE_OUT, PRESS, PRESS_TRANSITION, REFLOW_SPRING } from "../lib/motion";
import type {
  PlaylistCard as PlaylistCardType,
  ArtistItem,
  AlbumItem,
  TrackItem,
  SearchResults,
} from "../types/spotify";

const CATEGORIES = ["all", "songs", "artists", "albums", "playlists", "podcasts"] as const;
type Category = (typeof CATEGORIES)[number];
const CATEGORY_LABEL: Record<Category, string> = {
  all: "All",
  songs: "Songs",
  artists: "Artists",
  albums: "Albums",
  playlists: "Playlists",
  podcasts: "Podcasts",
};
const LABEL_TO_CATEGORY = Object.fromEntries(
  CATEGORIES.map((c) => [CATEGORY_LABEL[c], c]),
) as Record<string, Category>;

// songs shown beside the top result before "Show all"
const SONGS_PREVIEW = 4;


// ─── Top Result resolution & card ───────────────────────────────────────────

type TopResult =
  | { type: "artist"; item: ArtistItem }
  | { type: "track"; item: TrackItem }
  | { type: "album"; item: AlbumItem };

function getTopResult(data: SearchResults, query: string): TopResult | null {
  const q = query.trim().toLowerCase();
  if (!q) return null;

  // 1. Exact matches
  const exactArtist = data.artists.find((a) => a.name.toLowerCase() === q);
  if (exactArtist) return { type: "artist", item: exactArtist };

  const exactTrack = data.tracks.find((t) => t.name.toLowerCase() === q);
  if (exactTrack) return { type: "track", item: exactTrack };

  const exactAlbum = data.albums.find((al) => al.name.toLowerCase() === q);
  if (exactAlbum) return { type: "album", item: exactAlbum };

  // 2. Starts with query
  const startsArtist = data.artists.find((a) => a.name.toLowerCase().startsWith(q));
  if (startsArtist) return { type: "artist", item: startsArtist };

  const startsTrack = data.tracks.find((t) => t.name.toLowerCase().startsWith(q));
  if (startsTrack) return { type: "track", item: startsTrack };

  const startsAlbum = data.albums.find((al) => al.name.toLowerCase().startsWith(q));
  if (startsAlbum) return { type: "album", item: startsAlbum };

  // 3. Fallback priority: artist -> track -> album
  if (data.artists.length > 0) return { type: "artist", item: data.artists[0] };
  if (data.tracks.length > 0) return { type: "track", item: data.tracks[0] };
  if (data.albums.length > 0) return { type: "album", item: data.albums[0] };

  return null;
}

function topResultImage(r: TopResult): string | null {
  if (r.type === "track") return r.item.album?.image_url ?? null;
  return r.item.image_url ?? null;
}

/* The top result, laid out like the album page's header: artwork on the left,
eyebrow / title / byline / Play pill on the right. The old card put a small
image in one corner and the name in the other, and most of it was empty. */
const TopResultCard = memo(function TopResultCard({
  result,
  onPlay,
  isPlaying,
  pending,
}: {
  result: TopResult;
  onPlay: (e?: React.MouseEvent) => void;
  isPlaying: boolean;
  pending: boolean;
}) {
  const [hover, setHover] = useState(false);
  const [focused, setFocused] = useState(false);
  const navigate = useNavigate();

  const isArtist = result.type === "artist";
  const isTrack = result.type === "track";

  const title = result.item.name;
  const image = topResultImage(result);

  const eyebrow =
    result.type === "artist" ? "Artist" : result.type === "track" ? "Song" : result.item.album_type || "Album";

  // who made it, then where it lives (song) or when it came out (album)
  const byline = result.type === "artist" ? null : result.item.artists.map((a) => a.name).join(", ");
  const detail =
    result.type === "track"
      ? result.item.album?.name
      : result.type === "album"
      ? releaseYear(result.item.release_date)
      : null;

  const targetPath =
    result.type === "artist"
      ? `/artist/${result.item.id}`
      : result.type === "album"
      ? `/album/${result.item.id}`
      : result.item.album?.id
      ? `/album/${result.item.album.id}`
      : undefined;

  // a song's card plays it; an artist's or album's card opens it
  function handleClick() {
    if (isTrack) {
      onPlay();
    } else if (targetPath) {
      navigate(targetPath);
    }
  }

  /* the card is one big mouse target, but its keyboard stops are the two real
  controls inside it - the title (open / play) and the Play pill - so there is
  no focusable wrapper holding a nested <button>. focusing either lights the
  card the way hovering does. */
  const lit = hover || focused;
  const titleStyle: React.CSSProperties = {
    color: "inherit",
    textDecoration: "none",
    font: "inherit",
    letterSpacing: "inherit",
    textAlign: "left",
    background: "none",
    border: "none",
    padding: 0,
    cursor: "pointer",
    borderRadius: 4,
  };

  return (
    <motion.div
      layout="position"
      transition={{ layout: REFLOW_SPRING, scale: PRESS_TRANSITION }}
      onClick={handleClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      whileTap={{ scale: 0.99 }}
      aria-busy={pending || undefined}
      style={{
        position: "relative",
        display: "flex",
        alignItems: "center",
        height: "100%",
        minHeight: 220,
        boxSizing: "border-box",
        padding: "clamp(18px, 2.2vw, 26px)",
        borderRadius: 16,
        overflow: "hidden",
        isolation: "isolate",
        background: "var(--color-surface)",
        border: "1px solid rgba(255, 255, 255, 0.07)",
        boxShadow: lit ? "0 14px 36px rgba(0, 0, 0, 0.4)" : "0 2px 10px rgba(0, 0, 0, 0.18)",
        cursor: pending ? "progress" : "pointer",
        transition: "box-shadow 0.2s ease",
        userSelect: "none",
      }}
    >
      {/* the card takes the artwork's colour: the same blurred bloom the
          album and playlist headers cast on the page, held inside the card
          and darkened toward the corner so the type always reads */}
      {image && (
        <div
          aria-hidden
          style={{
            position: "absolute",
            inset: 0,
            zIndex: -1,
            backgroundImage: `url(${coverUrl(image, 64) ?? image})`,
            backgroundSize: "cover",
            backgroundPosition: "center",
            filter: "blur(56px) saturate(1.25)",
            transform: "scale(1.5)",
            opacity: lit ? 0.22 : 0.15,
            transition: "opacity 0.25s ease",
          }}
        />
      )}
      <div
        aria-hidden
        style={{
          position: "absolute",
          inset: 0,
          zIndex: -1,
          background: "linear-gradient(160deg, rgba(0, 0, 0, 0) 20%, rgba(0, 0, 0, 0.38) 100%)",
        }}
      />

      <div
        style={{
          display: "flex",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "clamp(16px, 2.2vw, 24px)",
          width: "100%",
          minWidth: 0,
        }}
      >
        <div
          style={{
            width: "clamp(104px, 11vw, 144px)",
            height: "clamp(104px, 11vw, 144px)",
            borderRadius: isArtist ? "50%" : 12,
            overflow: "hidden",
            flexShrink: 0,
            boxShadow: "0 18px 40px rgba(0, 0, 0, 0.5)",
          }}
        >
          <CoverArt
            url={image}
            alt={title}
            size={144}
            rounded={isArtist}
            style={{ width: "100%", height: "100%", borderRadius: "inherit" }}
          />
        </div>

        <div className="flex flex-col min-w-0" style={{ flex: "1 1 180px", gap: 6 }}>
          <p
            className="font-bold uppercase"
            style={{ margin: 0, fontSize: 11, letterSpacing: "0.06em", color: "rgba(255, 255, 255, 0.6)" }}
          >
            {eyebrow}
          </p>

          <h3
            className="line-clamp-2 break-words"
            title={title}
            style={{
              margin: 0,
              fontSize: "clamp(1.5rem, 2.6vw, 2.25rem)",
              fontWeight: 900,
              lineHeight: 1.06,
              letterSpacing: "-0.028em",
              color: "#ffffff",
            }}
          >
            {isTrack ? (
              <button
                type="button"
                className="focus-ring"
                onClick={(e) => { e.stopPropagation(); onPlay(e); }}
                style={titleStyle}
              >
                {title}
              </button>
            ) : targetPath ? (
              <Link
                to={targetPath}
                className="focus-ring"
                onClick={(e) => e.stopPropagation()}
                style={titleStyle}
              >
                {title}
              </Link>
            ) : (
              title
            )}
          </h3>

          {(byline || detail) && (
            <div
              className="line-clamp-1"
              style={{ fontSize: 14, fontWeight: 500, color: "rgba(255, 255, 255, 0.65)" }}
            >
              {byline && <span style={{ fontWeight: 700, color: "#ffffff" }}>{byline}</span>}
              {byline && detail && (
                <span aria-hidden style={{ color: "rgba(255, 255, 255, 0.35)", fontSize: 10, margin: "0 6px" }}>•</span>
              )}
              {detail}
            </div>
          )}

          {/* the album page's Play pill, always shown: the one thing you'd do
              with a top result shouldn't hide behind a hover */}
          <motion.button
            type="button"
            className="focus-ring"
            onClick={(e) => { e.stopPropagation(); onPlay(e); }}
            aria-label={isPlaying ? `Pause ${title}` : `Play ${title}`}
            whileTap={PRESS}
            transition={PRESS_TRANSITION}
            style={{
              marginTop: 10,
              alignSelf: "flex-start",
              display: "flex",
              alignItems: "center",
              gap: 7,
              height: 36,
              padding: "0 18px 0 16px",
              borderRadius: 99,
              border: "none",
              background: "#ffffff",
              color: "#000000",
              fontSize: 13.5,
              fontWeight: 700,
              letterSpacing: "-0.01em",
              cursor: pending ? "progress" : "pointer",
              opacity: pending ? 0.7 : 1,
              boxShadow: isPlaying
                ? "0 0 0 4px rgba(255, 255, 255, 0.25), 0 4px 16px rgba(0, 0, 0, 0.35)"
                : "0 2px 10px rgba(255, 255, 255, 0.20)",
              transition: "opacity 0.15s ease, box-shadow 0.2s ease",
            }}
          >
            <AnimatedPlayPause isPlaying={isPlaying} size={15} strokeWidth={0} fill="currentColor" />
            <span>{isPlaying ? "Pause" : "Play"}</span>
          </motion.button>
        </div>
      </div>
    </motion.div>
  );
});

// ─── Episodes ───────────────────────────────────────────────────────────────

function SearchEpisodes({ episodes }: { episodes: EpisodeItem[] }) {
  const play = usePlayEpisodes(episodes, "search-episodes");
  const currentId = usePlayerStore((s) => s.currentId);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  return (
    <div>
      {episodes.map((ep, i) => {
        const active = currentId === `${EPISODE_PREFIX}${ep.id}`;
        return <EpisodeRow key={ep.id} episode={ep} active={active} playing={active && isPlaying} onPlay={() => play(i)} showShow />;
      })}
    </div>
  );
}

// ─── Playlist card ──────────────────────────────────────────────────────────

// dressed exactly like AlbumCard so a shelf of playlists sits flush with a
// shelf of albums: same padding, radius, artwork shadow and type
const PlaylistResultCard = memo(function PlaylistResultCard({ playlist, index = 0 }: { playlist: PlaylistCardType; index?: number }) {
  return (
    <MediaTile
      to={`/playlist/${playlist.id}`}
      imageUrl={playlist.image_url}
      title={playlist.name}
      subtitle={playlist.owner_name ? `By ${playlist.owner_name}` : "Playlist"}
      index={index}
    />
  );
});

function PlaylistGrid({ children }: { children: React.ReactNode }) {
  return (
    <motion.div
      layout="position"
      transition={{ layout: REFLOW_SPRING }}
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(clamp(120px, 14vw, 175px), 1fr))",
        gap: "clamp(10px, 1.5vw, 16px)",
        width: "100%",
      }}
    >
      {children}
    </motion.div>
  );
}

// ─── Search Page ────────────────────────────────────────────────────────────

export default function Search() {
  useReflowPulse();
  const { loggedIn } = useAuth();
  const [params]        = useSearchParams();
  const query           = (params.get("q") ?? "").trim();
  const [cat, setCat]   = useState<Category>("all");
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const isPlaying       = usePlayerStore((s) => s.isPlaying);
  const enqueue         = useQueueStore((s) => s.enqueue);
  const playContext     = useQueueStore((s) => s.playContext);
  const setPageTint     = useUIStore((s) => s.setPageTint);
  const toggleLike      = useToggleLike();

  const { data, isLoading, error, refetch } = useSearch(query);
  // top-result play fetches the artist's / album's tracks first; show that
  // it was heard from the click, not from when the network answers
  const [topPending, setTopPending] = useState(false);
  const trackIds = useMemo(() => data?.tracks.map((t) => t.id) ?? [], [data?.tracks]);
  const { data: savedIds = [] } = useSavedTrackIds(trackIds);
  const likedSet = useMemo(() => new Set(savedIds), [savedIds]);

  const topResult = useMemo(() => {
    if (!data) return null;
    return getTopResult(data, query);
  }, [data, query]);

  // the page takes the top result's colour, the way an album or artist page
  // takes its artwork's
  const tintUrl = topResult ? topResultImage(topResult) : null;
  useEffect(() => {
    setPageTint(tintUrl);
    return () => setPageTint(null);
  }, [tintUrl, setPageTint]);

  const isTopResultPlaying = usePlayerStore((s) => {
    if (!s.isPlaying || !topResult || !s.currentTrack) return false;
    if (topResult.type === "track") return s.currentTrack.id === topResult.item.id;
    if (topResult.type === "artist") return s.currentTrack.artists.some((a) => a.id === topResult.item.id);
    if (topResult.type === "album") return s.currentTrack.album?.id === topResult.item.id;
    return false;
  });

  async function handlePlayTopResult(e?: React.MouseEvent) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (!topResult) return;

    if (isTopResultPlaying) {
      transportPause();
      return;
    }

    if (topResult.type === "track") {
      const curTrack = usePlayerStore.getState().currentTrack;
      if (curTrack?.id === topResult.item.id && !isPlaying) {
        transportPlay();
        return;
      }
      const idx = data?.tracks.findIndex((t) => t.id === topResult.item.id) ?? -1;
      const start = playContext(data?.tracks ?? [topResult.item], idx >= 0 ? idx : 0, "search");
      const trackToPlay = start || topResult.item;
      setCurrentTrack(trackToPlay);
      playTrack(trackToPlay.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(console.error);
      return;
    }

    if (topPending) return;
    setTopPending(true);
    try {
      const tracks =
        topResult.type === "artist"
          ? (await getArtist(topResult.item.id))?.top_tracks ?? []
          : (await getAlbum(topResult.item.id))?.tracks ?? [];
      if (tracks.length === 0) {
        toast.info(`Nothing to play from ${topResult.item.name}`);
        return;
      }
      const start = playContext(tracks, 0, topResult.item.id);
      if (start) {
        setCurrentTrack(start);
        playTrack(start.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(console.error);
      }
    } catch (err) {
      toast.error(`Couldn't play ${topResult.item.name}: ${errMsg(err)}`);
    } finally {
      setTopPending(false);
    }
  }

  if (!loggedIn) {
    return (
      <SignInPrompt
        heading="Search"
        title="Sign in to search Spotify"
        description="Log in with your Spotify account to search for songs, albums, and artists."
      />
    );
  }

  if (!query) return <Navigate to="/" replace />;

  const hasResults = Boolean(
    data &&
      (data.tracks.length > 0 ||
        data.artists.length > 0 ||
        data.albums.length > 0 ||
        data.playlists.length > 0)
  );

  function playFromSearch(i: number) {
    if (!data) return;
    const start = playContext(data.tracks, i, "search");
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(console.error);
    }
  }

  const songRows = (tracks: TrackItem[]) =>
    tracks.map((t, i) => (
      <TrackRow
        key={t.id}
        track={t}
        index={i}
        showAlbum
        liked={likedSet.has(t.id)}
        onPlay={() => playFromSearch(i)}
        onQueue={(track) => enqueue(track)}
        onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}
      />
    ));

  const showAll = (c: Category) => <ShowAllButton onClick={() => setCat(c)} />;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "clamp(28px, 3.6vw, 40px)",
        width: "100%",
        maxWidth: "100%",
        boxSizing: "border-box",
      }}
    >
      {/* header: an eyebrow and the query as the title, set like the album
          page's header, with the category switch on the same line when there
          is room. The query used to be painted in the accent colour, which
          the cover-tinted theme turned an arbitrary pink. */}
      <motion.header
        layout="position"
        transition={{ layout: REFLOW_SPRING }}
        style={{
          display: "flex",
          alignItems: "flex-end",
          justifyContent: "space-between",
          flexWrap: "wrap",
          gap: "12px 24px",
          paddingTop: 12,
        }}
      >
        <div className="flex flex-col min-w-0" style={{ gap: 8, flex: "1 1 260px" }}>
          <p
            className="font-bold uppercase"
            style={{ color: "var(--color-text-dim)", margin: 0, fontSize: 11, letterSpacing: "0.06em" }}
          >
            Search results
          </p>
          <h1
            className="font-black line-clamp-2 break-words"
            title={query}
            style={{
              margin: 0,
              fontSize: "clamp(24px, 3.8vw, 40px)",
              lineHeight: 1.06,
              letterSpacing: "-0.028em",
              color: "#ffffff",
            }}
          >
            “{query}”
          </h1>
        </div>

        <div role="group" aria-label="Filter results" style={{ maxWidth: "100%" }}>
          <SegmentedControl
            options={CATEGORIES.map((c) => CATEGORY_LABEL[c])}
            value={CATEGORY_LABEL[cat]}
            onChange={(label) => setCat(LABEL_TO_CATEGORY[label] ?? "all")}
            layoutId="search-category"
          />
        </div>
      </motion.header>

      {isLoading && (
        <div role="status" aria-label="Searching">
          {cat === "all" ? (
            <TopResultSkeleton />
          ) : cat === "songs" ? (
            <TrackRowsSkeleton count={10} />
          ) : (
            <CardGridSkeleton count={10} round={cat === "artists"} />
          )}
        </div>
      )}

      {error && !data && (
        <EmptyState
          title="Search didn't go through"
          description={errMsg(error)}
          action={
            <button type="button" className="btn-pill" onClick={() => refetch()}>
              Try again
            </button>
          }
        />
      )}

      {!isLoading && !error && !hasResults && (
        <EmptyState
          title={`No results for “${query}”`}
          description="Check the spelling, or try fewer or different words."
        />
      )}

      {data && hasResults && (
        // a category switch reads as the results updating, not a new page:
        // a short fade-up on the new set, no exit to wait on
        <motion.div
          key={cat}
          initial={{ opacity: 0, y: 6 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.22, ease: EASE_OUT }}
          style={{ display: "flex", flexDirection: "column", gap: "clamp(28px, 3.6vw, 40px)" }}
        >
          {cat === "all" && (
            <>
              {(topResult || data.tracks.length > 0) && (
                <motion.div
                  layout="position"
                  transition={{ layout: REFLOW_SPRING }}
                  style={{
                    display: "grid",
                    gridTemplateColumns:
                      topResult && data.tracks.length > 0
                        ? "repeat(auto-fit, minmax(min(100%, 340px), 1fr))"
                        : "1fr",
                    gap: "clamp(20px, 2.6vw, 32px)",
                    alignItems: "stretch",
                  }}
                >
                  {topResult && (
                    <section aria-labelledby="search-top" style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
                      <SectionTitle id="search-top">Top result</SectionTitle>
                      <div style={{ flex: 1, minHeight: 0 }}>
                        <TopResultCard
                          result={topResult}
                          onPlay={handlePlayTopResult}
                          isPlaying={isTopResultPlaying}
                          pending={topPending}
                        />
                      </div>
                    </section>
                  )}

                  {data.tracks.length > 0 && (
                    <section aria-labelledby="search-songs" style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
                      <SectionTitle
                        id="search-songs"
                        right={data.tracks.length > SONGS_PREVIEW && showAll("songs")}
                      >
                        Songs
                      </SectionTitle>
                      <div className="flex flex-col">{songRows(data.tracks.slice(0, SONGS_PREVIEW))}</div>
                    </section>
                  )}
                </motion.div>
              )}

              {/* the rest as shelves, the same rows Home and the artist page
                  use; "Show all" opens the full grid for that kind */}
              <Shelf
                id="search-artists"
                title="Artists"
                items={data.artists}
                getKey={(a) => a.id}
                renderItem={(a, i) => <ArtistCard artist={a} index={i} />}
                extra={showAll("artists")}
              />
              <Shelf
                id="search-albums"
                title="Albums"
                items={data.albums}
                getKey={(al) => al.id}
                renderItem={(al, i) => <AlbumCard album={al} index={i} />}
                extra={showAll("albums")}
              />
              <Shelf
                id="search-playlists"
                title="Playlists"
                items={data.playlists}
                getKey={(pl) => pl.id}
                renderItem={(pl, i) => <PlaylistResultCard playlist={pl} index={i} />}
                extra={showAll("playlists")}
              />
              <Shelf
                id="search-shows"
                title="Podcasts"
                items={data.shows ?? []}
                getKey={(s) => s.id}
                renderItem={(s, i) => <MediaTile to={`/show/${s.id}`} imageUrl={s.image_url} title={s.name} subtitle={s.publisher} index={i} />}
                extra={showAll("podcasts")}
              />
            </>
          )}

          {cat === "songs" && (
            data.tracks.length > 0 ? (
              <section>{songRows(data.tracks)}</section>
            ) : (
              <EmptyState title="No songs match" description="Try All to see other kinds of results." />
            )
          )}

          {cat === "artists" && (
            data.artists.length > 0 ? (
              <ArtistGrid>
                {data.artists.map((a, i) => <ArtistCard key={a.id} artist={a} index={i} />)}
              </ArtistGrid>
            ) : (
              <EmptyState title="No artists match" description="Try All to see other kinds of results." />
            )
          )}

          {cat === "albums" && (
            data.albums.length > 0 ? (
              <AlbumGrid>
                {data.albums.map((al, i) => <AlbumCard key={al.id} album={al} index={i} />)}
              </AlbumGrid>
            ) : (
              <EmptyState title="No albums match" description="Try All to see other kinds of results." />
            )
          )}

          {cat === "podcasts" && (
            (data.shows?.length ?? 0) + (data.episodes?.length ?? 0) > 0 ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 28 }}>
                {(data.shows?.length ?? 0) > 0 && (
                  <AlbumGrid>
                    {data.shows!.map((s, i) => (
                      <MediaTile key={s.id} to={`/show/${s.id}`} imageUrl={s.image_url} title={s.name} subtitle={s.publisher} index={i} />
                    ))}
                  </AlbumGrid>
                )}
                {(data.episodes?.length ?? 0) > 0 && (
                  <section aria-labelledby="search-episodes">
                    <SectionTitle id="search-episodes">Episodes</SectionTitle>
                    <SearchEpisodes episodes={data.episodes!} />
                  </section>
                )}
              </div>
            ) : (
              <EmptyState title="No podcasts match" description="Try All to see other kinds of results." />
            )
          )}

          {cat === "playlists" && (
            data.playlists.length > 0 ? (
              <PlaylistGrid>
                {data.playlists.map((pl, i) => <PlaylistResultCard key={pl.id} playlist={pl} index={i} />)}
              </PlaylistGrid>
            ) : (
              <EmptyState title="No playlists match" description="Try All to see other kinds of results." />
            )
          )}
        </motion.div>
      )}
    </div>
  );
}
