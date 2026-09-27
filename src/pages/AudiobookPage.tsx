import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Heart, Link2 } from "@/lib/icons";
import {
  getAudiobook,
  getAudiobookChapters,
  isAudiobookSaved,
  saveAudiobook,
  unsaveAudiobook,
} from "../api/audiobooks";
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
import { EPISODE_PREFIX, episodeLength } from "../utils/episode";
import { PRESS, PRESS_TRANSITION } from "../lib/motion";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { EpisodeItem } from "../types/podcast";

export default function AudiobookPage() {
  useReflowPulse();
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();
  const { open: openMenu, element: menuEl } = useContextMenu();
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["audiobook", id],
    queryFn: () => getAudiobook(id!),
    enabled: !!id,
    staleTime: 120_000,
  });
  const { data: saved = false } = useQuery({
    queryKey: ["library", "audiobook-saved", id],
    queryFn: () => isAudiobookSaved(id!),
    enabled: !!id,
  });

  const [more, setMore] = useState<EpisodeItem[]>([]);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);
  useEffect(() => {
    setMore([]);
    setNextOffset(data?.next_offset ?? null);
  }, [data]);

  const chapters = useMemo(() => [...(data?.chapters ?? []), ...more], [data?.chapters, more]);
  const play = usePlayEpisodes(chapters, `audiobook-${id}`);
  const currentId = usePlayerStore((s) => s.currentId);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  if (isLoading) return <Loader label="Loading audiobook" />;
  if (error || !data) {
    return (
      <EmptyState
        title="Couldn't load this audiobook"
        description={error ? errMsg(error) : "Audiobooks aren't available in every country."}
        action={<button type="button" className="btn-pill" onClick={() => refetch()}>Try again</button>}
      />
    );
  }

  async function toggleSave() {
    const key = ["library", "audiobook-saved", id];
    qc.setQueryData(key, !saved);
    try {
      await (saved ? unsaveAudiobook(id!) : saveAudiobook(id!));
      qc.invalidateQueries({ queryKey: ["library", "audiobooks"] });
    } catch (e) {
      qc.setQueryData(key, saved);
      toast.error(`Couldn't update your library: ${errMsg(e)}`);
    }
  }

  async function loadMore() {
    if (nextOffset == null || loadingMore) return;
    setLoadingMore(true);
    try {
      const page = await getAudiobookChapters(id!, nextOffset);
      setMore((m) => [...m, ...page.chapters]);
      setNextOffset(page.next_offset);
    } catch (e) {
      toast.error(`Couldn't load more chapters: ${errMsg(e)}`);
    } finally {
      setLoadingMore(false);
    }
  }

  // resume the first chapter you haven't finished
  const resumeIdx = Math.max(0, chapters.findIndex((c) => !c.fully_played && c.is_playable));
  const started = chapters.some((c) => c.fully_played || (c.resume_position_ms ?? 0) > 0);
  const totalMs = chapters.reduce((n, c) => n + c.duration_ms, 0);
  const anyPlayable = chapters.some((c) => c.is_playable);

  return (
    <div className="flex flex-col" onContextMenu={openMenu([
      { label: "Copy Spotify link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("audiobook", data.id) },
    ])}>
      <PageHeader imageUrl={data.image_url} eyebrow="Audiobook" title={data.name}>
        {data.authors.length > 0 && (
          <p className="text-sm" style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>{data.authors.join(", ")}</p>
        )}
        {data.narrators.length > 0 && (
          <p className="text-sm" style={{ color: "var(--color-text-dim)" }}>Narrated by {data.narrators.join(", ")}</p>
        )}
        {data.description && <ExpandableDescription text={data.description} />}
        <p className="text-sm tnum" style={{ color: "var(--color-text-dim)" }}>
          {data.total_chapters != null && <>{data.total_chapters} chapters</>}
          {totalMs > 0 && data.next_offset == null && <> · {episodeLength(totalMs)}</>}
          {data.edition && <> · {data.edition}</>}
          {data.publisher && <> · {data.publisher}</>}
        </p>
        <div style={{ display: "flex", gap: 10, marginTop: 8 }}>
          <motion.button
            type="button"
            className="btn-primary focus-ring"
            onClick={() => play(resumeIdx)}
            disabled={!anyPlayable}
            whileTap={PRESS}
            transition={PRESS_TRANSITION}
            style={{ height: 36, padding: "0 18px" }}
          >
            {currentId === `${EPISODE_PREFIX}${chapters[resumeIdx]?.id}` && isPlaying ? "Pause" : started ? "Resume" : "Start listening"}
          </motion.button>
          <Tooltip label={saved ? "Remove from your library" : "Save to your library"} side="top">
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
              <span>{saved ? "Saved" : "Save"}</span>
            </motion.button>
          </Tooltip>
        </div>
        {!anyPlayable && chapters.length > 0 && (
          <p className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)", marginTop: 6 }}>
            This book isn't playable on your account (it may need to be bought, or your listening hours are used up).
          </p>
        )}
      </PageHeader>

      <section aria-labelledby="book-chapters">
        <SectionTitle id="book-chapters">Chapters</SectionTitle>
        {chapters.map((ch, i) => {
          const active = currentId === `${EPISODE_PREFIX}${ch.id}`;
          return <EpisodeRow key={ch.id} episode={ch} active={active} playing={active && isPlaying} onPlay={() => play(i)} />;
        })}
        {nextOffset != null && (
          <div style={{ display: "flex", justifyContent: "center", padding: "16px 0" }}>
            <button type="button" className="btn-pill" onClick={loadMore} disabled={loadingMore}>
              {loadingMore ? "Loading..." : "Load more chapters"}
            </button>
          </div>
        )}
        {data.copyrights.length > 0 && (
          <p className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)", marginTop: 18 }}>
            {data.copyrights.join(" · ")}
          </p>
        )}
      </section>
      {menuEl}
    </div>
  );
}
