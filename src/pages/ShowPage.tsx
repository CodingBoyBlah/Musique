import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Heart, Link2, Search, X } from "@/lib/icons";
import { getShow, getShowEpisodes, isShowSaved, saveShow, unsaveShow } from "../api/podcasts";
import { PageHeader } from "../components/ui/PageHeader";
import { PlayActions } from "../components/ui/PlayActions";
import { ExpandableDescription } from "../components/ui/ExpandableDescription";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { EpisodeRow } from "../components/ui/EpisodeRow";
import { Dropdown, type DropdownOption } from "../components/ui/Dropdown";
import { Tooltip } from "../components/ui/Tooltip";
import { useContextMenu } from "../components/ui/ContextMenu";
import { usePlayEpisodes } from "../hooks/usePlayEpisodes";
import { usePlayerStore } from "../store/player.store";
import { shareSpotifyLink } from "../lib/share";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { EPISODE_PREFIX, episodeToTrack } from "../utils/episode";
import { resumeOrPlay, playTrack } from "../api/playback";
import { PRESS, PRESS_TRANSITION } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { EpisodeItem } from "../types/podcast";
import type { TrackItem } from "../types/spotify";

type Order = "newest" | "oldest" | "unplayed";
const ORDERS: DropdownOption<Order>[] = [
  { value: "newest", label: "Newest" },
  { value: "oldest", label: "Oldest" },
  { value: "unplayed", label: "Unplayed first" },
];

export default function ShowPage() {
  useReflowPulse();
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["show", id],
    queryFn: () => getShow(id!),
    enabled: !!id,
    staleTime: 120_000,
  });
  const { data: saved = false } = useQuery({
    queryKey: ["library", "show-saved", id],
    queryFn: () => isShowSaved(id!),
    enabled: !!id,
  });

  // the first page comes with the show; further pages append here
  const [more, setMore] = useState<EpisodeItem[]>([]);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);
  useEffect(() => {
    setMore([]);
    setNextOffset(data?.next_offset ?? null);
  }, [data]);

  const [query, setQuery] = useState("");
  const [order, setOrder] = useState<Order>("newest");
  const all = useMemo(() => [...(data?.episodes ?? []), ...more], [data?.episodes, more]);
  const view = useMemo(() => {
    const q = query.trim().toLowerCase();
    let arr = q ? all.filter((e) => e.name.toLowerCase().includes(q) || (e.description ?? "").toLowerCase().includes(q)) : all;
    if (order === "oldest") arr = [...arr].reverse();
    if (order === "unplayed") arr = [...arr].sort((a, b) => Number(a.fully_played) - Number(b.fully_played));
    return arr;
  }, [all, query, order]);

  const play = usePlayEpisodes(view, `show-${id}`);
  const currentId = usePlayerStore((s) => s.currentId);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  // Play = the newest episode you haven't finished, picking up where you were
  const tracks: TrackItem[] = useMemo(() => {
    const firstUnplayed = all.findIndex((e) => !e.fully_played && e.is_playable);
    return all.slice(Math.max(0, firstUnplayed)).map(episodeToTrack);
  }, [all]);

  if (isLoading) return <Loader label="Loading podcast" />;
  if (error || !data) {
    return (
      <EmptyState
        title="Couldn't load this podcast"
        description={error ? errMsg(error) : undefined}
        action={<button type="button" className="btn-pill" onClick={() => refetch()}>Try again</button>}
      />
    );
  }

  async function toggleSave() {
    const key = ["library", "show-saved", id];
    qc.setQueryData(key, !saved);
    try {
      await (saved ? unsaveShow(id!) : saveShow(id!));
      qc.invalidateQueries({ queryKey: ["library", "shows"] });
    } catch (e) {
      qc.setQueryData(key, saved);
      toast.error(`Couldn't update your library: ${errMsg(e)}`);
    }
  }

  async function loadMore() {
    if (nextOffset == null || loadingMore) return;
    setLoadingMore(true);
    try {
      const page = await getShowEpisodes(id!, nextOffset);
      setMore((m) => [...m, ...page.episodes]);
      setNextOffset(page.next_offset);
    } catch (e) {
      toast.error(`Couldn't load more episodes: ${errMsg(e)}`);
    } finally {
      setLoadingMore(false);
    }
  }

  function startFrom(track: TrackItem) {
    const ep = all.find((e) => `${EPISODE_PREFIX}${e.id}` === track.id);
    const pos = ep && !ep.fully_played ? ep.resume_position_ms ?? 0 : 0;
    (pos > 0 ? resumeOrPlay(track.id, pos) : playTrack(track.id)).catch((e) => toast.error(`Couldn't play episode: ${errMsg(e)}`));
  }

  const meta = [
    data.total_episodes != null ? `${data.total_episodes.toLocaleString()} episodes` : null,
    data.languages.length > 0 ? data.languages.slice(0, 2).join(", ").toUpperCase() : null,
    data.explicit ? "Explicit" : null,
  ].filter(Boolean);

  return (
    <div className="flex flex-col" onContextMenu={openMenu([
      { label: saved ? "Unfollow" : "Follow", icon: <Heart size={14} active={saved} />, onSelect: toggleSave },
      { label: "Copy Spotify link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("show", data.id) },
    ])}>
      <PageHeader imageUrl={data.image_url} eyebrow="Podcast" title={data.name}>
        {data.description && <ExpandableDescription text={data.description} />}
        <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>
          <span style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>{data.publisher}</span>
          {meta.map((m) => (
            <span key={m} className="tnum" style={{ textTransform: "uppercase", letterSpacing: "0.05em", fontWeight: 600 }}> · {m}</span>
          ))}
        </p>
        <PlayActions
          tracks={tracks}
          contextId={`show-${data.id}`}
          pinItem={{ id: data.id, name: data.name, image_url: data.image_url, type: "show" }}
          onStart={startFrom}
          hideShuffle
          accessory={
            <Tooltip label={saved ? "Unfollow podcast" : "Follow podcast"} side="top">
              <motion.button
                type="button"
                onClick={toggleSave}
                aria-pressed={saved}
                className="ghost-pill focus-ring"
                data-on={saved}
                whileTap={PRESS}
                transition={PRESS_TRANSITION}
                style={{ height: 36, padding: "0 16px", borderRadius: 99, color: "#ffffff", fontSize: 13, fontWeight: 600, display: "flex", alignItems: "center", gap: 6, cursor: "pointer", flexShrink: 0 }}
              >
                <Heart size={14} strokeWidth={2.2} active={saved} />
                <span>{saved ? "Following" : "Follow"}</span>
              </motion.button>
            </Tooltip>
          }
        />
      </PageHeader>

      <section>
        <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, padding: "0 2px 14px" }}>
          <div
            className="focus-within-ring"
            style={{
              display: "flex", alignItems: "center", gap: 8, height: 32, flex: "0 1 280px", minWidth: 0, padding: "0 11px",
              borderRadius: 8, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)",
            }}
          >
            <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Find in episodes"
              aria-label="Find in episodes"
              spellCheck={false}
              style={{ flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none", background: "transparent", color: "var(--color-text-hi)", fontSize: 13, fontFamily: "inherit" }}
            />
            {query && (
              <button onClick={() => setQuery("")} aria-label="Clear" className="btn-icon" style={{ width: 20, height: 20, borderRadius: 6, padding: 0 }}>
                <X size={13} strokeWidth={2.4} />
              </button>
            )}
          </div>
          <div style={{ marginLeft: "auto" }}>
            <Dropdown value={order} options={ORDERS} onChange={setOrder} align="right" minWidth={160} title="Sort episodes" />
          </div>
        </div>

        {view.length === 0 ? (
          <p className="t-caption" style={{ color: "var(--color-text-dim)", padding: "8px 2px" }}>
            {all.length === 0 ? "No episodes available in your region." : "No episodes match your filter."}
          </p>
        ) : (
          view.map((ep, i) => {
            const active = currentId === `${EPISODE_PREFIX}${ep.id}`;
            return <EpisodeRow key={ep.id} episode={ep} index={i} active={active} playing={active && isPlaying} onPlay={() => play(i)} />;
          })
        )}
        {nextOffset != null && !query && (
          <div style={{ display: "flex", justifyContent: "center", padding: "16px 0" }}>
            <button type="button" className="btn-pill" onClick={loadMore} disabled={loadingMore}>
              {loadingMore ? "Loading..." : "Load more episodes"}
            </button>
          </div>
        )}
      </section>
      {menuEl}
    </div>
  );
}
