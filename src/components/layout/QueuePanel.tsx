import { memo } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { motion, AnimatePresence, Reorder } from "framer-motion";
import { X, GripVertical, Queue } from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { playTrack } from "../../api/playback";
import { fmtMs } from "../../utils/fmt";
import { meshGradient } from "../../lib/mesh";
import type { TrackItem } from "../../types/spotify";
import { Tooltip } from "../ui/Tooltip";
import { EASE_DRAWER, EASE_OUT, SPRING_PANEL } from "../../lib/motion";
import { useQuery } from "@tanstack/react-query";
import { getRemoteQueue } from "../../api/connect";

const WIDTH = 272;

/* Stable identity for queue entries. The store gives every queued entry its
   own object (see queue.store enqueue/playNext), so the object itself is the
   identity: removing row 3 animates row 3 out, and a reorder moves rows rather
   than re-mounting them. The index was the key before, which renamed every row
   after a removal and animated the *last* one out instead. */
const uidOf = new WeakMap<object, number>();
let uidSeq = 0;
function entryUid(t: TrackItem): number {
  let id = uidOf.get(t);
  if (id === undefined) {
    id = ++uidSeq;
    uidOf.set(t, id);
  }
  return id;
}

// small square cover, falls back to a seeded mesh gradient (no more grey box)
function Cover({ track, size }: { track: TrackItem; size: number }) {
  const art = track.album?.image_url;
  return art ? (
    <img
      src={coverUrl(art, size) ?? art}
      alt=""
      loading="lazy"
      decoding="async"
      style={{ width: size, height: size, borderRadius: 6, objectFit: "cover", flexShrink: 0, outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1 }}
    />
  ) : (
    <div style={{ width: size, height: size, borderRadius: 6, flexShrink: 0, outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1, overflow: "hidden", ...meshGradient(track.id) }} />
  );
}

const QueueTrackRow = memo(function QueueTrackRow({
  track, onRemove, onPlay, reorderable, dim,
}: {
  track:        TrackItem;
  onRemove?:    () => void;
  onPlay?:      () => void;
  reorderable?: boolean;
  dim?:         boolean;
}) {
  return (
    <div
      className="group q-row"
      data-drag={reorderable || undefined}
      data-clickable={onPlay ? true : undefined}
      role={onPlay ? "button" : undefined}
      tabIndex={onPlay ? 0 : undefined}
      onClick={onPlay}
      onKeyDown={onPlay ? (e) => {
        if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onPlay(); }
      } : undefined}
      style={{
        display: "flex", alignItems: "center", gap: 9,
        padding: "6px 8px",
        opacity: dim ? 0.5 : 1,
      }}
    >
      {reorderable && (
        <span
          className="q-grip"
          aria-hidden
          style={{ color: "rgba(255,255,255,0.3)", flexShrink: 0, display: "flex", alignItems: "center", marginLeft: -2 }}
        >
          <GripVertical size={13} strokeWidth={2} />
        </span>
      )}
      <Cover track={track} size={36} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <p className="t-caption" style={{ margin: 0, fontSize: 12.5, fontWeight: 500, color: "rgba(255,255,255,0.9)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.name}
        </p>
        <p className="t-caption" style={{ margin: 0, fontSize: 11, color: "rgba(255,255,255,0.5)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.artists.map((a) => a.name).join(", ")}
        </p>
      </div>
      {onRemove ? (
        <Tooltip label="Remove from queue" side="top" align="end">
          <button
            aria-label={`Remove ${track.name} from queue`}
            // a press on the x must not start a row drag
            onPointerDown={(e) => e.stopPropagation()}
            onClick={(e) => { e.stopPropagation(); onRemove(); }}
            className="queue-btn btn-icon q-remove"
          >
            <X size={12} strokeWidth={2.5} />
          </button>
        </Tooltip>
      ) : (
        <span className="tnum t-caption" style={{ fontSize: 11, color: "rgba(255,255,255,0.36)", flexShrink: 0 }}>
          {fmtMs(track.duration_ms)}
        </span>
      )}
    </div>
  );
});

// animated 3-bar equaliser for the now-playing card
function Equaliser() {
  return (
    <span style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 13 }}>
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="eq-bar-q"
          style={{
            width: 3,
            borderRadius: 2,
            background: "var(--color-accent)",
            height: "100%",
            ["--eq-delay" as string]: `${i * 0.18}s`,
          }}
        />
      ))}
    </span>
  );
}

const NowPlayingCard = memo(function NowPlayingCard({ track, isPlaying }: { track: TrackItem; isPlaying: boolean }) {
  return (
    <div
      style={{
        display: "flex", alignItems: "center", gap: 11,
        margin: "0 8px", padding: 10, borderRadius: 12,
        background: "var(--color-surface)",
        outline: "1px solid var(--color-border)", outlineOffset: -1,
      }}
    >
      <Cover track={track} size={46} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ margin: 0, fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.name}
        </p>
        <p className="t-caption" style={{ margin: "2px 0 0", fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.artists.map((a) => a.name).join(", ")}
        </p>
      </div>
      {isPlaying && <Equaliser />}
    </div>
  );
});

function SectionHead({ label, onClear }: { label: string; onClear?: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 12px 6px" }}>
      <p style={{ margin: 0, fontSize: 10.5, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "rgba(255,255,255,0.34)" }}>
        {label}
      </p>
      {onClear && (
        <Tooltip label={`Clear ${label.toLowerCase()}`} side="top" align="end">
          <button
            onClick={onClear}
            className="btn-text t-caption"
            style={{ fontSize: 11, fontWeight: 600, padding: "0 2px" }}
          >
            Clear
          </button>
        </Tooltip>
      )}
    </div>
  );
}

function EmptyRow({ children }: { children: React.ReactNode }) {
  return (
    <p className="t-caption" style={{ margin: 0, padding: "4px 12px 8px", fontSize: 12, color: "rgba(255,255,255,0.45)" }}>{children}</p>
  );
}

/* spotify's own queue on the device that's playing. read-only: spotify's api
can add to it but not reorder or remove, so there's nothing to drag. */
function RemoteQueue({ deviceName }: { deviceName: string }) {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["remote-queue"],
    queryFn: getRemoteQueue,
    refetchInterval: 5000,
    staleTime: 2000,
  });
  const items = data?.queue ?? [];
  return (
    <>
      <SectionHead label={`Next up on ${deviceName}`} />
      {isLoading ? (
        <EmptyRow>Loading queue...</EmptyRow>
      ) : isError ? (
        <EmptyRow>Couldn't read the queue on {deviceName}</EmptyRow>
      ) : items.length === 0 ? (
        <EmptyRow>Nothing queued on {deviceName}</EmptyRow>
      ) : (
        <div style={{ padding: "0 4px" }}>
          {items.slice(0, 30).map((track, i) => (
            <QueueTrackRow key={track.id + i} track={track} />
          ))}
        </div>
      )}
    </>
  );
}

export function QueuePanel() {
  const toggleQueue     = usePlayerStore((s) => s.toggleQueue);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);
  const isPlaying       = usePlayerStore((s) => s.isPlaying);
  const isRemote        = usePlayerStore((s) => s.isRemotePlayback);
  const remoteName      = usePlayerStore((s) => s.activeDevice?.name ?? "device");

  const queue        = useQueueStore((s) => s.queue);
  const history      = useQueueStore((s) => s.history);
  const removeAt     = useQueueStore((s) => s.removeAt);
  const setQueue     = useQueueStore((s) => s.setQueue);
  const clearQueue   = useQueueStore((s) => s.clearQueue);
  const clearHistory = useQueueStore((s) => s.clearHistory);

  // the same object can legitimately appear twice (older persisted queues,
  // previous() pushing the current track back). give the repeat its own key
  // rather than letting two rows collide.
  const seen = new Map<number, number>();
  const keyed = queue.map((track) => {
    const uid = entryUid(track);
    const n = seen.get(uid) ?? 0;
    seen.set(uid, n + 1);
    return { track, key: n === 0 ? `q${uid}` : `q${uid}-${n}` };
  });
  // Reorder finds the dragged row by value identity, so it gets the per-row
  // keys (unique even when a track repeats), not the track objects.
  const keys = keyed.map((k) => k.key);
  const onReorder = (order: string[]) => {
    const byKey = new Map(keyed.map((k) => [k.key, k.track]));
    setQueue(order.map((k) => byKey.get(k)!).filter(Boolean));
  };

  function playItem(track: TrackItem) {
    setCurrentTrack(track);
    usePlayerStore.getState().setPlaying(true);
    usePlayerStore.getState().setTargetState("playing");
    playTrack(track.id).catch(() => {});
  }

  return (
    <motion.div
      // in-flow rail below the title bar; slides via transform. width is reserved
      // by the spacer in Layout.tsx so the grid reflows once, both ways.
      // in from the right, out to the right, on the sheet curve both ways.
      // (the exit used to be an ease-in, which starts slow at the exact moment
      // the user is watching for the panel to respond.)
      initial={{ x: WIDTH }}
      animate={{ x: 0 }}
      exit={{ x: WIDTH, transition: { duration: 0.22, ease: EASE_DRAWER } }}
      transition={SPRING_PANEL}
      style={{
        position: "absolute", top: 0, right: 0, bottom: 0, zIndex: 5,
        width: WIDTH, maxWidth: "100vw", display: "flex", flexDirection: "column", overflow: "hidden",
        background: "transparent", borderLeft: "none",
        boxShadow: "none",
        willChange: "transform",
      }}
    >
      <div style={{ width: WIDTH, flexShrink: 0, display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
        {/* header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "4px 12px 0 14px", height: 40, flexShrink: 0, borderBottom: "none" }}>
          <span style={{ fontSize: 14, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Queue</span>
          <Tooltip label="Close queue" side="bottom" align="end">
            <button
              onClick={toggleQueue}
              aria-label="Close queue"
              className="btn-icon"
              style={{ width: 26, height: 26, borderRadius: 7 }}
            >
              <X size={14} strokeWidth={2.2} />
            </button>
          </Tooltip>
        </div>

        <div className="scroll-y" style={{ flex: 1, overflowY: "auto", overflowX: "hidden", paddingBottom: 12 }}>
          {/* now playing */}
          <SectionHead label="Now playing" />
          {currentTrack
            ? <NowPlayingCard track={currentTrack} isPlaying={isPlaying} />
            : <EmptyRow>Nothing playing</EmptyRow>}

          {/* next up */}
          {isRemote ? <RemoteQueue deviceName={remoteName} /> : <>
          <SectionHead label="Next up" onClear={queue.length > 0 ? clearQueue : undefined} />
          {queue.length === 0 ? (
            <div className="t-caption" style={{ margin: "0 12px", padding: "16px 14px", borderRadius: 10, border: "1.5px dashed var(--color-glass-border)", background: "var(--color-glass)", display: "flex", alignItems: "center", gap: 9, fontSize: 12, color: "var(--color-text-dim)" }}>
              <Queue size={14} strokeWidth={2} style={{ flexShrink: 0 }} />
              <span>Nothing queued. Add a song with the ＋ on any track.</span>
            </div>
          ) : (
            /* Drag to reorder: the row stays glued to the pointer (from where
               it was grabbed) and its neighbours part live as it passes them,
               instead of the old HTML5 drag ghost that showed nothing until the
               drop. Reorder.Item captures the pointer, so the drag keeps
               tracking when it leaves the row. */
            <Reorder.Group
              as="div"
              axis="y"
              values={keys}
              onReorder={onReorder}
              style={{ padding: "0 4px" }}
            >
              <AnimatePresence initial={false}>
                {keyed.map(({ track, key }, i) => (
                  <Reorder.Item
                    as="div"
                    key={key}
                    value={key}
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: "auto" }}
                    exit={{ opacity: 0, height: 0, transition: { duration: 0.18, ease: EASE_DRAWER } }}
                    transition={{ duration: 0.2, ease: EASE_OUT }}
                    whileDrag={{
                      scale: 1.02,
                      boxShadow: "0 10px 26px rgba(0,0,0,0.45)",
                      backgroundColor: "rgba(40,40,46,0.96)",
                      zIndex: 30,
                    }}
                    style={{ position: "relative", borderRadius: 9 }}
                  >
                    <QueueTrackRow
                      track={track}
                      reorderable
                      onRemove={() => removeAt(i)}
                    />
                  </Reorder.Item>
                ))}
              </AnimatePresence>
            </Reorder.Group>
          )}
          </>}

          {/* history */}
          {history.length > 0 && (
            <>
              <SectionHead label="Recently played" onClear={clearHistory} />
              <div style={{ padding: "0 4px" }}>
                {[...history].reverse().slice(0, 10).map((track, i) => (
                  <QueueTrackRow key={track.id + i} track={track} dim onPlay={() => playItem(track)} />
                ))}
              </div>
            </>
          )}
        </div>
      </div>
    </motion.div>
  );
}
