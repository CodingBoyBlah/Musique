import { useRef, useState } from "react";
import { useEvenColumns } from "../hooks/useEvenColumns";
import { Link, Navigate } from "react-router-dom";
import { usePrefsStore } from "../store/prefs.store";
import { useQuery } from "@tanstack/react-query";
import { Stats } from "@/lib/icons";
import { getListeningStats, type ListeningStats, type Ranked, type StatsRange } from "../api/stats";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { SectionTitle } from "../components/ui/SectionTitle";
import { Shelf } from "../components/ui/Shelf";
import { ArtistCard } from "../components/ui/ArtistCard";
import { AlbumCard } from "../components/ui/AlbumCard";
import { CoverArt } from "../components/ui/CoverArt";
import { Loader } from "../components/ui/Loader";
import { EmptyState } from "../components/ui/EmptyState";
import { usePlayerStore } from "../store/player.store";
import { useQueueStore } from "../store/queue.store";
import { playTrack } from "../api/playback";
import { errMsg } from "../lib/err";
import { heatLevel, heatMax, hourLabel, listenTime, peakSlot } from "../utils/stats";
import { useReflowPulse } from "../hooks/useReflowPulse";
import type { TrackItem } from "../types/spotify";

const RANGES: { value: StatsRange; label: string }[] = [
  { value: "week", label: "7 days" },
  { value: "month", label: "30 days" },
  { value: "year", label: "This year" },
  { value: "all", label: "All time" },
];
const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const FULL_DAYS = ["Sundays", "Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays"];
// monday-first rows read more naturally for a week
const ROW_ORDER = [1, 2, 3, 4, 5, 6, 0];

function Tile({ label, value, hint }: { label: string; value: string; hint?: string }) {
  return (
    <div style={{ padding: "14px 16px", borderRadius: 14, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", minWidth: 0 }}>
      <div className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)", fontWeight: 600 }}>{label}</div>
      <div className="tnum" style={{ fontSize: "clamp(20px, 2.2vw, 28px)", fontWeight: 800, color: "var(--color-text-hi)", letterSpacing: "-0.02em", marginTop: 4 }}>{value}</div>
      {hint && <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)", marginTop: 2 }}>{hint}</div>}
    </div>
  );
}

/* when you listen: weekday rows x hour columns, one accent hue light -> dark.
each cell names its own value on hover (and to screen readers). */
function Heatmap({ grid }: { grid: number[][] }) {
  const max = heatMax(grid);
  const [hover, setHover] = useState<{ day: number; hour: number } | null>(null);
  const peak = peakSlot(grid);
  return (
    <section aria-labelledby="stats-when">
      <SectionTitle id="stats-when">When you listen</SectionTitle>
      <div style={{ overflowX: "auto", paddingBottom: 4 }}>
        <div role="grid" aria-label="Plays by day and hour" style={{ display: "grid", gridTemplateColumns: "36px repeat(24, minmax(14px, 1fr))", gap: 3, minWidth: 420 }}>
          <span />
          {Array.from({ length: 24 }, (_, h) => (
            <span key={h} className="t-caption tnum" style={{ fontSize: 10, color: "var(--color-text-dim)", textAlign: "center" }}>
              {h % 6 === 0 ? hourLabel(h) : ""}
            </span>
          ))}
          {ROW_ORDER.map((day) => (
            <div key={day} role="row" style={{ display: "contents" }}>
              <span className="t-caption" style={{ fontSize: 11, color: "var(--color-text-dim)", alignSelf: "center" }}>{DAYS[day]}</span>
              {grid[day].map((plays, hour) => {
                const level = heatLevel(plays, max);
                const on = hover?.day === day && hover?.hour === hour;
                return (
                  <div
                    key={hour}
                    role="gridcell"
                    aria-label={`${DAYS[day]} ${hourLabel(hour)}: ${plays} ${plays === 1 ? "play" : "plays"}`}
                    title={`${DAYS[day]} ${hourLabel(hour)} · ${plays} ${plays === 1 ? "play" : "plays"}`}
                    onMouseEnter={() => setHover({ day, hour })}
                    onMouseLeave={() => setHover(null)}
                    style={{
                      aspectRatio: "1 / 1",
                      borderRadius: 4,
                      background: level > 0 ? `color-mix(in srgb, var(--color-accent) ${Math.round(level * 100)}%, transparent)` : "rgba(255,255,255,0.05)",
                      outline: on ? "2px solid var(--color-text-hi)" : "none",
                      outlineOffset: 1,
                    }}
                  />
                );
              })}
            </div>
          ))}
        </div>
      </div>
      <p className="t-caption" style={{ fontSize: 12.5, color: "var(--color-text-dim)", marginTop: 8, minHeight: 18 }}>
        {hover
          ? `${DAYS[hover.day]} ${hourLabel(hover.hour)}: ${grid[hover.day][hover.hour]} plays`
          : peak
          ? `You listen most on ${FULL_DAYS[peak.day]} around ${hourLabel(peak.hour)}.`
          : ""}
      </p>
    </section>
  );
}

// share of listening per genre, single-series bars with direct labels
function Genres({ genres }: { genres: ListeningStats["top_genres"] }) {
  if (genres.length === 0) return null;
  const max = genres[0].share;
  return (
    <section aria-labelledby="stats-genres">
      <SectionTitle id="stats-genres">Top genres</SectionTitle>
      <div style={{ display: "flex", flexDirection: "column", gap: 8, maxWidth: 640 }}>
        {genres.map((g) => (
          <div key={g.genre} style={{ display: "grid", gridTemplateColumns: "minmax(90px, 160px) 1fr 44px", alignItems: "center", gap: 10 }} title={`${g.genre}: ${Math.round(g.share * 100)}%`}>
            <span style={{ fontSize: 13, color: "var(--color-text)", textTransform: "capitalize", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{g.genre}</span>
            <div style={{ height: 10, borderRadius: 4, background: "rgba(255,255,255,0.05)", overflow: "hidden" }}>
              <div style={{ width: `${(g.share / max) * 100}%`, height: "100%", borderRadius: 4, background: "var(--color-accent)" }} />
            </div>
            <span className="tnum" style={{ fontSize: 12.5, color: "var(--color-text-dim)", textAlign: "right" }}>{Math.round(g.share * 100)}%</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function RankedTracks({ id, title, rows, contextId, caption }: { id: string; title: string; rows: Ranked<TrackItem>[]; contextId: string; caption?: string }) {
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const playContext = useQueueStore((s) => s.playContext);
  if (rows.length === 0) return null;
  const tracks = rows.map((r) => r.item);
  function play(i: number) {
    const start = playContext(tracks, i, contextId);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
    }
  }
  return (
    <section aria-labelledby={id}>
      <SectionTitle id={id}>{title}</SectionTitle>
      {caption && <p className="t-caption" style={{ fontSize: 12.5, color: "var(--color-text-dim)", margin: "-4px 0 8px" }}>{caption}</p>}
      <ol style={{ listStyle: "none", margin: 0, padding: 0 }}>
        {rows.slice(0, 10).map((r, i) => (
          <li key={r.item.id}>
            <button
              type="button"
              className="q-row focus-ring"
              data-clickable
              onClick={() => play(i)}
              style={{
                all: "unset", boxSizing: "border-box", width: "100%", display: "flex", alignItems: "center", gap: 12,
                // the same geometry as TrackRow, so every track list in the app lines up
                padding: "8px 12px", margin: "1.5px 0", borderRadius: 8, cursor: "pointer",
                background: i % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent",
              }}
            >
              <span className="tnum" style={{ width: 28, flexShrink: 0, textAlign: "center", fontSize: 13, color: "var(--color-text-muted)" }}>{i + 1}</span>
              <CoverArt url={r.item.album?.image_url} alt="" size={38} style={{ width: 38, height: 38, flexShrink: 0 }} />
              <span style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
                <span style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{r.item.name}</span>
                <span className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {r.item.artists.map((a) => a.name).join(", ")}
                </span>
              </span>
              <span className="tnum t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)", flexShrink: 0 }}>
                {r.plays} {r.plays === 1 ? "play" : "plays"}
              </span>
            </button>
          </li>
        ))}
      </ol>
    </section>
  );
}

const TILE_MIN = 150;
const TILE_GAP = 10;

/* always six tiles, laid out 6 / 3 / 2 / 1 across so the rows stay even at
every width */
function StatTiles({ data }: { data: ListeningStats }) {
  const ref = useRef<HTMLDivElement>(null);
  const cols = useEvenColumns(ref, 6, TILE_MIN, TILE_GAP);
  const days = (n: number) => `${n} ${n === 1 ? "day" : "days"}`;
  const biggest = data.biggest_day
    ? `Most in a day: ${listenTime(data.biggest_day[1])}`
    : undefined;
  return (
    <div ref={ref} data-rail-lock="flip" style={{ display: "grid", gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`, gap: TILE_GAP }}>
      <Tile label="Time listened" value={listenTime(data.total_ms)} hint={biggest} />
      <Tile label="Plays" value={data.plays.toLocaleString()} />
      <Tile label="Songs" value={data.distinct_tracks.toLocaleString()} />
      <Tile label="Artists" value={data.distinct_artists.toLocaleString()} />
      <Tile label="Current streak" value={days(data.current_streak)} />
      <Tile label="Longest streak" value={days(data.longest_streak)} />
    </div>
  );
}

export default function StatsPage() {
  useReflowPulse();
  const [range, setRange] = useState<StatsRange>("month");
  const enabled = usePrefsStore((s) => s.showStats);
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ["stats", range],
    queryFn: () => getListeningStats(range),
    staleTime: 60_000,
    enabled,
  });

  // turned off in settings: the page doesn't exist
  if (!enabled) return <Navigate to="/" replace />;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(26px, 3.4vw, 38px)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 12, flexWrap: "wrap" }}>
        <h1 className="t-title" style={{ margin: 0, color: "var(--color-text-hi)" }}>Your listening</h1>
        <SegmentedControl
          options={RANGES.map((r) => r.label)}
          value={RANGES.find((r) => r.value === range)!.label}
          onChange={(label) => setRange(RANGES.find((r) => r.label === label)?.value ?? "month")}
          layoutId="stats-range"
        />
      </div>

      {isLoading ? (
        <Loader label="Crunching your listening" />
      ) : error || !data ? (
        <EmptyState
          title="Couldn't load your stats"
          description={error ? errMsg(error) : undefined}
          action={<button type="button" className="btn-pill" onClick={() => refetch()}>Try again</button>}
        />
      ) : data.plays === 0 ? (
        <EmptyState
          icon={<Stats size={22} />}
          title="Nothing here yet"
          description="Stats build up from what you play in Musique. Play a few songs and check back."
        />
      ) : (
        <>
          <StatTiles data={data} />

          <Heatmap grid={data.heatmap} />
          <RankedTracks id="stats-top-tracks" title="Top songs" rows={data.top_tracks} contextId={`stats-top-${range}`} />
          <Shelf
            id="stats-top-artists"
            title="Top artists"
            items={data.top_artists}
            getKey={(r) => r.item.id}
            renderItem={(r, i) => <ArtistCard artist={r.item} index={i} />}
          />
          <Shelf
            id="stats-top-albums"
            title="Top albums"
            items={data.top_albums}
            getKey={(r) => r.item.id}
            renderItem={(r, i) => <AlbumCard album={r.item} index={i} />}
          />
          <Genres genres={data.top_genres} />
          <RankedTracks id="stats-repeat" title="On repeat" caption="What you can't stop playing this month." rows={data.on_repeat} contextId="stats-repeat" />
          <RankedTracks id="stats-discoveries" title="New discoveries" caption="Songs you played for the first time in this period." rows={data.discoveries} contextId={`stats-new-${range}`} />
          <RankedTracks id="stats-forgotten" title="Forgotten favourites" caption="Songs you used to finish every time, not played in two months." rows={data.forgotten} contextId="stats-forgotten" />
          <p className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
            Worked out on this device from what you play in Musique. See also your <Link to="/profile" style={{ color: "var(--color-text)" }}>profile</Link>.
          </p>
        </>
      )}
    </div>
  );
}
