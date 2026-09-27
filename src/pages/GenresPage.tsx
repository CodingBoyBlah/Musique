import { useQuery } from "@tanstack/react-query";
import { Sparkles } from "@/lib/icons";
import { GenreCrate } from "../components/ui/GenreCrate";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { search } from "../api/spotify";
import { playTrack, pausePlayback } from "../api/playback";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { getMyGenres } from "../api/stats";

// the genres you actually listen to, each as a crate of records to dig through
export default function GenresPage() {
  useReflowPulse();
  const { data = [], isLoading, error } = useQuery({
    queryKey: ["library", "my-genres"],
    queryFn: getMyGenres,
    staleTime: 10 * 60_000,
  });
  const contextId = useQueueStore((s) => s.contextId);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  async function playGenre(genre: string) {
    const ctx = `genre-${genre}`;
    if (useQueueStore.getState().contextId === ctx && usePlayerStore.getState().isPlaying) {
      pausePlayback().catch(() => {});
      return;
    }
    try {
      const res = await search(`genre:"${genre}"`, "track");
      if (res.tracks.length === 0) {
        toast.info(`Nothing to play for ${genre}`);
        return;
      }
      const start = useQueueStore.getState().playContextShuffled(res.tracks, ctx);
      if (start) {
        usePlayerStore.getState().setCurrentTrack(start);
        playTrack(start.id).catch(() => {});
      }
    } catch (e) {
      toast.error(`Couldn't play ${genre}: ${errMsg(e)}`);
    }
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(20px, 2.5vw, 28px)" }}>
      <div>
        <h1 className="t-title" style={{ margin: 0, color: "var(--color-text-hi)" }}>Your genres</h1>
        <p className="t-caption" style={{ margin: "6px 0 0", fontSize: 13, color: "var(--color-text-dim)" }}>
          From what you play, your top artists and who you follow.
        </p>
      </div>
      {isLoading ? (
        <Loader label="Sorting your records" />
      ) : error ? (
        <EmptyState title="Couldn't load your genres" description={errMsg(error)} />
      ) : data.length === 0 ? (
        <EmptyState
          icon={<Sparkles size={22} />}
          title="No genres yet"
          description="Sync your library or play a few songs and your genres will show up here."
        />
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(clamp(170px, 18vw, 220px), 1fr))", gap: "clamp(14px, 2vw, 24px)" }}>
          {data.map((g) => (
            <GenreCrate
              key={g.genre}
              genre={g.genre}
              to={`/search?q=${encodeURIComponent(`genre:"${g.genre}"`)}`}
              onPlay={() => playGenre(g.genre)}
              isPlaying={contextId === `genre-${g.genre}` && isPlaying}
            />
          ))}
        </div>
      )}
    </div>
  );
}
