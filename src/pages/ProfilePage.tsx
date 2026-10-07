import { useState, useMemo } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, UserPlus, Link2 } from "@/lib/icons";
import {
  getUserProfile,
  getUserFollowers,
  getUserFollowing,
  isUserFollowed,
  setUserFollowed,
  type ProfileCard,
} from "../api/profile";
import { PageHeader } from "../components/ui/PageHeader";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { Shelf } from "../components/ui/Shelf";
import { MediaTile } from "../components/ui/MediaTile";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { SectionTitle } from "../components/ui/SectionTitle";
import { TrackRow } from "../components/ui/TrackRow";
import { Tooltip } from "../components/ui/Tooltip";
import { useContextMenu } from "../components/ui/ContextMenu";
import { useTopArtists, useTopTracks, useSavedTrackIds, useToggleLike } from "../hooks/useLibrary";
import { useAuthStore } from "../store/auth.store";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";
import { shareSpotifyLink } from "../lib/share";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { PRESS, PRESS_TRANSITION } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";

type Tab = "overview" | "followers" | "following";

function PeopleGrid({ people }: { people: ProfileCard[] }) {
  return (
    <ArtistGrid>
      {people.map((p, i) => (
        <MediaTile
          key={`${p.kind}-${p.id}`}
          to={p.kind === "artist" ? `/artist/${p.id}` : `/user/${p.id}`}
          imageUrl={p.image_url}
          title={p.name}
          subtitle={p.kind === "artist" ? "Artist" : p.followers != null ? `${p.followers.toLocaleString()} followers` : "Profile"}
          index={i}
          round
        />
      ))}
    </ArtistGrid>
  );
}

function FollowList({ userId, which }: { userId: string | null; which: "followers" | "following" }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["profile", which, userId],
    queryFn: () => (which === "followers" ? getUserFollowers(userId) : getUserFollowing(userId)),
    staleTime: 300_000,
  });
  if (isLoading) return <Loader label={`Loading ${which}`} />;
  if (error) return <p className="t-caption" style={{ color: "var(--color-text-dim)" }}>{errMsg(error)}</p>;
  if (!data || data.length === 0) {
    return <p className="t-caption" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>{which === "followers" ? "No followers yet." : "Not following anyone yet."}</p>;
  }
  return <PeopleGrid people={data} />;
}

export default function ProfilePage() {
  useReflowPulse();
  const { id } = useParams<{ id: string }>();
  const myId = useAuthStore((s) => s.userId);
  const isMe = !id || id === myId;
  const profileId = isMe ? null : id!;
  const [tab, setTab] = useState<Tab>("overview");
  const qc = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();

  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["profile", "view", profileId ?? "me"],
    queryFn: () => getUserProfile(profileId),
    staleTime: 300_000,
  });
  const { data: following = false } = useQuery({
    queryKey: ["profile", "is-followed", profileId],
    queryFn: () => isUserFollowed(profileId!),
    enabled: !!profileId,
  });

  // your own page gets your listening too
  const { data: topArtists = [] } = useTopArtists("short_term");
  const { data: topTracks = [] } = useTopTracks("short_term");
  const shownTop = useMemo(() => (isMe ? topTracks.slice(0, 5) : []), [isMe, topTracks]);
  const shownTrackIds = useMemo(() => shownTop.map((t) => t.id), [shownTop]);
  const { data: savedIds = [] } = useSavedTrackIds(shownTrackIds);
  const liked = useMemo(() => new Set(savedIds), [savedIds]);
  const toggleLike = useToggleLike();
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);
  const enqueue = useQueueStore((s) => s.enqueue);

  if (isLoading) return <Loader label="Loading profile" />;
  if (error || !data) {
    return (
      <EmptyState
        title="Couldn't load this profile"
        description={error ? errMsg(error) : undefined}
        action={<button type="button" className="btn-pill" onClick={() => refetch()}>Try again</button>}
      />
    );
  }

  async function toggleFollow() {
    const key = ["profile", "is-followed", profileId];
    qc.setQueryData(key, !following);
    try {
      await setUserFollowed(profileId!, !following);
      qc.invalidateQueries({ queryKey: ["profile", "view"] });
    } catch (e) {
      qc.setQueryData(key, following);
      toast.error(`Couldn't update follow: ${errMsg(e)}`);
    }
  }

  const stat = (n: number | null | undefined, one: string, many: string, to?: Tab) => {
    if (n == null) return null;
    const text = `${n.toLocaleString()} ${n === 1 ? one : many}`;
    return to ? (
      <button
        type="button"
        className="focus-ring"
        onClick={() => setTab(tab === to ? "overview" : to)}
        aria-pressed={tab === to}
        style={{ background: "none", border: "none", padding: 0, cursor: "pointer", font: "inherit", color: tab === to ? "var(--color-text-hi)" : "inherit", textDecoration: tab === to ? "underline" : "none" }}
      >
        {text}
      </button>
    ) : <span>{text}</span>;
  };

  const stats = [
    stat(data.total_public_playlists ?? data.playlists.length, "public playlist", "public playlists"),
    stat(data.followers, "follower", "followers", "followers"),
    stat(data.following, "following", "following", "following"),
  ].filter(Boolean);

  const shareEntries = [
    { label: "Copy profile link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("user", data.id) },
  ];

  return (
    <div className="flex flex-col" onContextMenu={openMenu(shareEntries)}>
      <PageHeader round imageUrl={data.image_url} eyebrow={data.is_verified ? "Verified profile" : "Profile"} title={data.name}>
        <p className="text-sm tnum" style={{ color: "var(--color-text-dim)", display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
          {stats.map((s, i) => (
            <span key={i} style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
              {i > 0 && <span aria-hidden>·</span>}
              {s}
            </span>
          ))}
        </p>
        <div style={{ display: "flex", gap: 10, marginTop: 8 }}>
          {!isMe && (
            <Tooltip label={following ? "Unfollow" : "Follow"} side="top">
              <motion.button
                type="button"
                onClick={toggleFollow}
                aria-pressed={following}
                className="ghost-pill focus-ring"
                data-on={following}
                whileTap={PRESS}
                transition={PRESS_TRANSITION}
                style={{ height: 36, padding: "0 16px", borderRadius: 99, color: "#ffffff", fontSize: 13, fontWeight: 600, display: "flex", alignItems: "center", gap: 6, cursor: "pointer" }}
              >
                {following ? <Check size={14} strokeWidth={2.4} /> : <UserPlus size={14} strokeWidth={2.2} />}
                <span>{following ? "Following" : "Follow"}</span>
              </motion.button>
            </Tooltip>
          )}
          <motion.button
            type="button"
            className="ghost-pill focus-ring"
            onClick={() => shareSpotifyLink("user", data.id)}
            whileTap={PRESS}
            transition={PRESS_TRANSITION}
            style={{ height: 36, padding: "0 16px", borderRadius: 99, color: "#ffffff", fontSize: 13, fontWeight: 600, display: "flex", alignItems: "center", gap: 6, cursor: "pointer" }}
          >
            <Link2 size={14} strokeWidth={2.2} />
            <span>Copy link</span>
          </motion.button>
        </div>
      </PageHeader>

      <div style={{ display: "flex", flexDirection: "column", gap: "clamp(28px, 4vw, 44px)", paddingTop: 8 }}>
        {tab !== "overview" ? (
          <section aria-labelledby="profile-people">
            <SectionTitle id="profile-people">{tab === "followers" ? "Followers" : "Following"}</SectionTitle>
            <FollowList userId={profileId} which={tab} />
          </section>
        ) : (
          <>
            {isMe && topArtists.length > 0 && (
              <Shelf
                id="profile-top-artists"
                title="Top artists this month"
                items={topArtists.slice(0, 12)}
                getKey={(a) => a.id}
                renderItem={(a, i) => <ArtistCard artist={a} index={i} />}
              />
            )}
            {shownTop.length > 0 && (
              <section aria-labelledby="profile-top-tracks">
                <SectionTitle id="profile-top-tracks">Top tracks this month</SectionTitle>
                {shownTop.map((t, i) => (
                  <TrackRow
                    key={t.id}
                    track={t}
                    index={i}
                    showAlbum
                    liked={liked.has(t.id)}
                    onPlay={() => {
                      const start = playContext(topTracks, i, "profile-top");
                      if (start) {
                        setCurrentTrack(start);
                        playTrack(start.id).catch(() => {});
                      }
                    }}
                    onQueue={(track) => enqueue(track)}
                    onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: liked.has(track.id) })}
                  />
                ))}
              </section>
            )}
            <Shelf
              id="profile-recent-artists"
              title="Recently played artists"
              items={data.recently_played_artists}
              getKey={(a) => a.id}
              renderItem={(a, i) => <ArtistCard artist={a} index={i} />}
            />
            <Shelf
              id="profile-playlists"
              title="Public playlists"
              items={data.playlists}
              getKey={(p) => p.id}
              renderItem={(p, i) => (
                <MediaTile to={`/playlist/${p.id}`} imageUrl={p.image_url} title={p.name} subtitle={p.owner_name ? `By ${p.owner_name}` : "Playlist"} index={i} />
              )}
            />
            {data.playlists.length === 0 && data.recently_played_artists.length === 0 && !isMe && (
              <p className="t-caption" style={{ color: "var(--color-text-dim)" }}>Nothing public on this profile yet.</p>
            )}
          </>
        )}
      </div>
      {menuEl}
    </div>
  );
}
