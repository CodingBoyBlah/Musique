import { memo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { X, Users, RefreshCw } from "@/lib/icons";
import { getFriendActivity, type FriendActivity } from "../../api/social";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { playTrack } from "../../api/playback";
import { CoverArt } from "../ui/CoverArt";
import { JamCard } from "./JamCard";
import { Tooltip } from "../ui/Tooltip";
import { errMsg } from "../../lib/err";
import { EASE_DRAWER, SPRING_PANEL } from "../../lib/motion";
import type { TrackItem } from "../../types/spotify";

const WIDTH = 272;
// spotify shows "now" for anything in the last few minutes
const LIVE_MS = 6 * 60_000;

function ago(ts: number, now: number): string {
  const d = Math.max(0, now - ts);
  if (d < LIVE_MS) return "now";
  const m = Math.round(d / 60_000);
  if (m < 60) return `${m} m`;
  const h = Math.round(m / 60);
  if (h < 24) return `${h} hr`;
  return `${Math.round(h / 24)} d`;
}

function asTrack(f: FriendActivity): TrackItem | null {
  if (!f.track_id || !f.track_name) return null;
  return {
    id: f.track_id,
    name: f.track_name,
    duration_ms: 0,
    explicit: false,
    artists: f.artist_id ? [{ id: f.artist_id, name: f.artist_name ?? "", image_url: null }] : [],
    album: f.album_id
      ? { id: f.album_id, name: f.album_name ?? "", album_type: "album", image_url: f.track_image, release_date: null, artists: [] }
      : null,
  };
}

const FriendRow = memo(function FriendRow({ f, now }: { f: FriendActivity; now: number }) {
  const live = now - f.timestamp < LIVE_MS;
  const track = asTrack(f);
  const contextLink = f.context_uri?.startsWith("spotify:playlist:")
    ? `/playlist/${f.context_uri.split(":")[2]}`
    : f.context_uri?.startsWith("spotify:album:")
    ? `/album/${f.context_uri.split(":")[2]}`
    : f.context_uri?.startsWith("spotify:artist:")
    ? `/artist/${f.context_uri.split(":")[2]}`
    : null;

  function play() {
    if (!track) return;
    const start = useQueueStore.getState().playContext([track], 0, `friend-${f.user_id}`);
    if (start) {
      usePlayerStore.getState().setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
    }
  }

  return (
    <div style={{ display: "flex", gap: 10, padding: "8px 10px", alignItems: "flex-start" }}>
      <Link to={`/user/${f.user_id}`} style={{ position: "relative", flexShrink: 0 }} aria-label={f.name}>
        {f.image_url ? (
          <CoverArt url={f.image_url} alt="" size={36} rounded style={{ width: 36, height: 36 }} />
        ) : (
          <span style={{ width: 36, height: 36, borderRadius: "50%", display: "flex", alignItems: "center", justifyContent: "center", background: "var(--color-surface-2)", fontSize: 14, fontWeight: 700, color: "var(--color-text-dim)" }}>
            {f.name.slice(0, 1).toUpperCase()}
          </span>
        )}
        {live && (
          <span aria-label="Listening now" style={{ position: "absolute", right: -1, bottom: -1, width: 11, height: 11, borderRadius: "50%", background: "var(--color-accent)", border: "2px solid var(--color-bg, #111)" }} />
        )}
      </Link>
      <div style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", gap: 1 }}>
        <div style={{ display: "flex", justifyContent: "space-between", gap: 6 }}>
          <Link to={`/user/${f.user_id}`} style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)", textDecoration: "none", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {f.name}
          </Link>
          <span className="t-caption tnum" style={{ fontSize: 11.5, color: live ? "var(--color-accent)" : "var(--color-text-dim)", flexShrink: 0 }}>
            {ago(f.timestamp, now)}
          </span>
        </div>
        {track && (
          <button
            type="button"
            onClick={play}
            className="focus-ring"
            title={`Play ${f.track_name}`}
            style={{ all: "unset", cursor: "pointer", fontSize: 12.5, color: "var(--color-text)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
          >
            {f.track_name}
            {f.artist_name && <span style={{ color: "var(--color-text-dim)" }}> · {f.artist_name}</span>}
          </button>
        )}
        {f.context_name && (
          contextLink ? (
            <Link to={contextLink} className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)", textDecoration: "none", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {f.context_name}
            </Link>
          ) : (
            <span className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>{f.context_name}</span>
          )
        )}
      </div>
    </div>
  );
});

// the right-rail list of what the people you follow are playing
export function FriendsPanel() {
  const toggleFriends = usePlayerStore((s) => s.toggleFriends);
  const { data = [], isLoading, error, refetch, isFetching, dataUpdatedAt } = useQuery({
    queryKey: ["friend-activity"],
    queryFn: getFriendActivity,
    refetchInterval: 60_000,
    staleTime: 30_000,
    retry: false,
  });
  const now = dataUpdatedAt || Date.now();

  return (
    <motion.div
      initial={{ x: WIDTH }}
      animate={{ x: 0 }}
      exit={{ x: WIDTH, transition: { duration: 0.22, ease: EASE_DRAWER } }}
      transition={SPRING_PANEL}
      style={{ position: "absolute", top: 0, right: 0, bottom: 0, zIndex: 5, width: WIDTH, maxWidth: "100vw", display: "flex", flexDirection: "column", overflow: "hidden", willChange: "transform" }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "4px 12px 0 14px", height: 40, flexShrink: 0 }}>
        <span style={{ fontSize: 14, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Friend activity</span>
        <div style={{ display: "flex", gap: 2 }}>
          <Tooltip label="Refresh" side="bottom">
            <button onClick={() => refetch()} aria-label="Refresh friend activity" className="btn-icon" style={{ width: 26, height: 26, borderRadius: 7 }} disabled={isFetching}>
              <RefreshCw size={13} strokeWidth={2.2} />
            </button>
          </Tooltip>
          <Tooltip label="Close" side="bottom" align="end">
            <button onClick={toggleFriends} aria-label="Close friend activity" className="btn-icon" style={{ width: 26, height: 26, borderRadius: 7 }}>
              <X size={14} strokeWidth={2.2} />
            </button>
          </Tooltip>
        </div>
      </div>
      <div className="scroll-y" style={{ flex: 1, overflowY: "auto", paddingBottom: 12 }}>
        <JamCard />
        {isLoading ? (
          <p className="t-caption" style={{ padding: "12px 14px", color: "var(--color-text-dim)" }}>Loading...</p>
        ) : error ? (
          <p className="t-caption" style={{ padding: "12px 14px", color: "var(--color-text-dim)" }}>Couldn't load friend activity: {errMsg(error)}</p>
        ) : data.length === 0 ? (
          <div className="t-caption" style={{ margin: "8px 12px", padding: "16px 14px", borderRadius: 10, border: "1.5px dashed var(--color-glass-border)", display: "flex", gap: 9, fontSize: 12, color: "var(--color-text-dim)" }}>
            <Users size={14} strokeWidth={2} style={{ flexShrink: 0 }} />
            <span>Follow friends on Spotify and what they play shows up here (if they share their listening).</span>
          </div>
        ) : (
          data.map((f) => <FriendRow key={f.user_id} f={f} now={now} />)
        )}
      </div>
    </motion.div>
  );
}
