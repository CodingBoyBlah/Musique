import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Heart, Link2 } from "@/lib/icons";
import { getShow, getShowEpisodes, isShowSaved, saveShow, unsaveShow } from "../api/podcasts";
import { PageHeader } from "../components/ui/PageHeader";
import { ExpandableDescription } from "../components/ui/ExpandableDescription";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { SectionTitle } from "../components/ui/SectionTitle";
import { EpisodeRow } from "../components/ui/EpisodeRow";
import { Tooltip } from "../components/ui/Tooltip";
import { useContextMenu } from "../components/ui/ContextMenu";
import { usePlayEpisodes } from "../hooks/usePlayEpisodes";
import { usePlayerStore } from "../store/player.store";
import { shareSpotifyLink } from "../lib/share";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { EPISODE_PREFIX } from "../utils/episode";
import { PRESS, PRESS_TRANSITION } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { EpisodeItem } from "../types/podcast";

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

  const episodes = useMemo(() => [...(data?.episodes ?? []), ...more], [data?.episodes, more]);
  const play = usePlayEpisodes(episodes, `show-${id}`);
  const currentId = usePlayerStore((s) => s.currentId);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

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

  // "latest" = the newest you haven't finished, which is what Play should mean
  const latestIdx = Math.max(0, episodes.findIndex((e) => !e.fully_played && e.is_playable));

  return (
    <div className="flex flex-col" onContextMenu={openMenu([
      { label: "Copy Spotify link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("show", data.id) },
    ])}>
      <PageHeader imageUrl={data.image_url} eyebrow="Podcast" title={data.name}>
        <p className="text-sm" style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>{data.publisher}</p>
        {data.description && <ExpandableDescription text={data.description} />}
        <p className="text-sm tnum" style={{ color: "var(--color-text-dim)" }}>
          {data.total_episodes != null && <>{data.total_episodes.toLocaleString()} episodes</>}
          {data.languages.length > 0 && <> · {data.languages.slice(0, 2).join(", ").toUpperCase()}</>}
          {data.explicit && <> · Explicit</>}
        </p>
        <div style={{ display: "flex", gap: 10, marginTop: 8 }}>
          <motion.button
            type="button"
            className="btn-primary focus-ring"
            onClick={() => play(latestIdx)}
            disabled={episodes.length === 0}
            whileTap={PRESS}
            transition={PRESS_TRANSITION}
            style={{ height: 36, padding: "0 18px" }}
          >
            {currentId === `${EPISODE_PREFIX}${episodes[latestIdx]?.id}` && isPlaying ? "Pause" : "Play latest"}
          </motion.button>
          <Tooltip label={saved ? "Unfollow podcast" : "Follow podcast"} side="top">
            <motion.button
              type="button"
              onClick={toggleSave}
              aria-pressed={saved}
              className="ghost-pill focus-ring"
              data-on={saved}
              whileTap={PRESS}
              transition={PRESS_TRANSITION}
              style={{ height: 36, padding: "0 16px", borderRadius: 99, color: "#ffffff", fontSize: 13, fontWeight: 600, display: "flex", alignItems: "center", gap: 6, cursor: "pointer" }}
            >
              <Heart size={14} strokeWidth={2.2} active={saved} />
              <span>{saved ? "Following" : "Follow"}</span>
            </motion.button>
          </Tooltip>
        </div>
      </PageHeader>

      <section aria-labelledby="show-episodes">
        <SectionTitle id="show-episodes">All episodes</SectionTitle>
        {episodes.length === 0 ? (
          <p className="t-caption" style={{ color: "var(--color-text-dim)" }}>No episodes available in your region.</p>
        ) : (
          episodes.map((ep, i) => {
            const active = currentId === `${EPISODE_PREFIX}${ep.id}`;
            return (
              <EpisodeRow key={ep.id} episode={ep} active={active} playing={active && isPlaying} onPlay={() => play(i)} />
            );
          })
        )}
        {nextOffset != null && (
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
