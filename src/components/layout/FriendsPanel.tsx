import { memo } from "react";
import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { X, Users, RefreshCw, ListMusic, Disc3, User, Play } from "@/lib/icons";
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
// spotify treats anything in the last few minutes as "listening now"
const LIVE_MS = 6 * 60_000;

function ago(ts: number, now: number): string {
  const d = Math.max(0, now - ts);
  const m = Math.round(d / 60_000);
  if (m < 60) return `${Math.max(1, m)}m`;
  const h = Math.round(m / 60);
  if (h < 24) return `${h}h`;
  const days = Math.round(h / 24);
  return days < 7 ? `${days}d` : `${Math.round(days / 7)}w`;
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

function contextOf(f: FriendActivity): { to: string; Icon: typeof ListMusic } | null {
  const [, kind, id] = (f.context_uri ?? "").split(":");
  if (!id) return null;
  if (kind === "playlist") return { to: `/playlist/${id}`, Icon: ListMusic };
  if (kind === "album") return { to: `/album/${id}`, Icon: Disc3 };
  if (kind === "artist") return { to: `/artist/${id}`, Icon: User };
  return null;
}

function Bars() {
  return (
    <span aria-label="Listening now" style={{ display: "inline-flex", alignItems: "flex-end", gap: 1.5, height: 10 }}>
      {[0, 1, 2].map((i) => (
        <span key={i} className="eq-bar-q" style={{ width: 2, height: "100%", borderRadius: 1, background: "var(--color-accent)", ["--eq-delay" as string]: `${i * 0.18}s` }} />
      ))}
    </span>
  );
}

const Label = ({ children }: { children: React.ReactNode }) => (
  <p style={{ margin: 0, padding: "14px 14px 6px", fontSize: 10.5, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "rgba(255,255,255,0.34)" }}>
    {children}
  </p>
);

const FriendRow = memo(function FriendRow({ f, now, live }: { f: FriendActivity; now: number; live: boolean }) {
  const track = asTrack(f);
  const ctx = contextOf(f);

  function play() {
    if (!track) return;
    const start = useQueueStore.getState().playContext([track], 0, `friend-${f.user_id}`);
    if (start) {
      usePlayerStore.getState().setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
    }
  }

  return (
    <div className="q-row fr-row" style={{ display: "flex", gap: 10, padding: "8px 10px", margin: "0 4px", alignItems: "center", borderRadius: 10 }}>
      <Link to={`/user/${f.user_id}`} aria-label={`${f.name}'s profile`} style={{ position: "relative", flexShrink: 0, lineHeight: 0 }}>
        {f.image_url ? (
          <CoverArt url={f.image_url} alt="" size={40} rounded style={{ width: 40, height: 40 }} />
        ) : (
          <span style={{ width: 40, height: 40, borderRadius: "50%", display: "flex", alignItems: "center", justifyContent: "center", background: "var(--color-surface-2)", fontSize: 15, fontWeight: 700, color: "var(--color-text-dim)" }}>
            {f.name.slice(0, 1).toUpperCase()}
          </span>
        )}
        {live && (
          <span style={{ position: "absolute", right: -2, bottom: -2, width: 16, height: 16, borderRadius: "50%", background: "var(--color-popover, #1c1c22)", display: "flex", alignItems: "center", justifyContent: "center" }}>
            <span style={{ width: 9, height: 9, borderRadius: "50%", background: "var(--color-accent)" }} />
          </span>
        )}
      </Link>

      <div style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", gap: 2 }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 6 }}>
          <Link to={`/user/${f.user_id}`} style={{ fontSize: 13, fontWeight: 650, color: "var(--color-text-hi)", textDecoration: "none", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {f.name}
          </Link>
          {live ? <Bars /> : (
            <span className="t-caption tnum" style={{ fontSize: 11, color: "var(--color-text-dim)", flexShrink: 0 }}>{ago(f.timestamp, now)}</span>
          )}
        </div>
        {track && (
          <span style={{ fontSize: 12, color: "var(--color-text)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {f.track_name}
            {f.artist_name && <span style={{ color: "var(--color-text-dim)" }}> · {f.artist_name}</span>}
          </span>
        )}
        {f.context_name && (
          ctx ? (
            <Link to={ctx.to} className="fr-ctx" style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 11.5, color: "var(--color-text-dim)", textDecoration: "none", minWidth: 0 }}>
              <ctx.Icon size={11} style={{ flexShrink: 0 }} />
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{f.context_name}</span>
            </Link>
          ) : (
            <span className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{f.context_name}</span>
          )
        )}
      </div>

      {track && (
        <Tooltip label={`Play ${f.track_name}`} side="top" align="end">
          <button
            type="button"
            onClick={play}
            aria-label={`Play ${f.track_name}`}
            className="fr-art focus-ring"
            style={{ position: "relative", width: 38, height: 38, padding: 0, border: "none", borderRadius: 6, overflow: "hidden", flexShrink: 0, cursor: "pointer", background: "transparent" }}
          >
            <CoverArt url={f.track_image} alt="" size={38} style={{ width: 38, height: 38 }} />
            <span className="fr-art-play" style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.45)", color: "#fff" }}>
              <Play size={14} />
            </span>
          </button>
        </Tooltip>
      )}
    </div>
  );
});

// the right-rail view of what the people you follow are playing, with jam on top
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
  const live = data.filter((f) => now - f.timestamp < LIVE_MS);
  const earlier = data.filter((f) => now - f.timestamp >= LIVE_MS);

  return (
    <motion.div
      initial={{ x: WIDTH }}
      animate={{ x: 0 }}
      exit={{ x: WIDTH, transition: { duration: 0.22, ease: EASE_DRAWER } }}
      transition={SPRING_PANEL}
      style={{ position: "absolute", top: 0, right: 0, bottom: 0, zIndex: 5, width: WIDTH, maxWidth: "100vw", display: "flex", flexDirection: "column", overflow: "hidden", willChange: "transform" }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "4px 12px 0 14px", height: 40, flexShrink: 0 }}>
        <span style={{ fontSize: 14, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Friends</span>
        <div style={{ display: "flex", gap: 2 }}>
          <Tooltip label="Refresh" side="bottom">
            <button onClick={() => refetch()} aria-label="Refresh friend activity" className="btn-icon" style={{ width: 26, height: 26, borderRadius: 7 }} disabled={isFetching}>
              <motion.span animate={{ rotate: isFetching ? 360 : 0 }} transition={isFetching ? { repeat: Infinity, duration: 0.9, ease: "linear" } : { duration: 0 }} style={{ display: "flex" }}>
                <RefreshCw size={13} strokeWidth={2.2} />
              </motion.span>
            </button>
          </Tooltip>
          <Tooltip label="Close" side="bottom" align="end">
            <button onClick={toggleFriends} aria-label="Close friend activity" className="btn-icon" style={{ width: 26, height: 26, borderRadius: 7 }}>
              <X size={14} strokeWidth={2.2} />
            </button>
          </Tooltip>
        </div>
      </div>

      <div className="scroll-y" style={{ flex: 1, overflowY: "auto", paddingBottom: 16 }}>
        <JamCard />
        {isLoading ? (
          <p className="t-caption" style={{ padding: "12px 14px", color: "var(--color-text-dim)" }}>Loading...</p>
        ) : error ? (
          <p className="t-caption" style={{ padding: "12px 14px", color: "var(--color-text-dim)" }}>Couldn't load friend activity: {errMsg(error)}</p>
        ) : data.length === 0 ? (
          <div style={{ padding: "28px 22px", textAlign: "center", display: "flex", flexDirection: "column", alignItems: "center", gap: 8 }}>
            <span style={{ width: 44, height: 44, borderRadius: "50%", background: "var(--color-glass)", display: "flex", alignItems: "center", justifyContent: "center", color: "var(--color-text-dim)" }}>
              <Users size={20} />
            </span>
            <span style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)" }}>No friend activity</span>
            <span className="t-caption" style={{ fontSize: 12, lineHeight: 1.45, color: "var(--color-text-dim)" }}>
              Follow friends on Spotify. What they play shows up here if they share their listening.
            </span>
          </div>
        ) : (
          <>
            {live.length > 0 && (
              <>
                <Label>Listening now</Label>
                {live.map((f) => <FriendRow key={f.user_id} f={f} now={now} live />)}
              </>
            )}
            {earlier.length > 0 && (
              <>
                <Label>{live.length > 0 ? "Earlier" : "Recently played"}</Label>
                {earlier.map((f) => <FriendRow key={f.user_id} f={f} now={now} live={false} />)}
              </>
            )}
          </>
        )}
      </div>
    </motion.div>
  );
}
