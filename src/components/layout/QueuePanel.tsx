import { useRef } from "react";
import { motion } from "framer-motion";
import { X, GripVertical } from "lucide-react";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { playTrack } from "../../api/playback";
import { fmtMs } from "../../utils/fmt";
import type { TrackItem } from "../../types/spotify";

function QueueTrackRow({
  track,
  onRemove,
  draggable,
  onDragStart,
  onDragOver,
  onDrop,
  dim,
}: {
  track:        TrackItem;
  onRemove?:    () => void;
  draggable?:   boolean;
  onDragStart?: () => void;
  onDragOver?:  (e: React.DragEvent) => void;
  onDrop?:      () => void;
  dim?:         boolean;
}) {
  return (
    <div
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={(e) => { e.preventDefault(); onDragOver?.(e); }}
      onDrop={onDrop}
      className="group"
      style={{
        display:      "flex",
        alignItems:   "center",
        gap:          6,
        padding:      "5px 8px",
        borderRadius: 6,
        opacity:      dim ? 0.4 : 1,
        cursor:       draggable ? "grab" : "default",
        transition:   "background 0.1s",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLDivElement).style.background = "var(--color-surface-hover)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLDivElement).style.background = "transparent";
      }}
    >
      {draggable && (
        <span
          className="group-hover-visible"
          style={{ color: "rgba(255,255,255,0.20)", flexShrink: 0, display: "flex", alignItems: "center" }}
        >
          <GripVertical size={12} strokeWidth={2} />
        </span>
      )}
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ margin: 0, fontSize: 12, fontWeight: 500, color: "rgba(255,255,255,0.88)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.name}
        </p>
        <p style={{ margin: 0, fontSize: 11, color: "rgba(255,255,255,0.42)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.artists.map((a) => a.name).join(", ")}
        </p>
      </div>
      <span style={{ fontSize: 11, color: "rgba(255,255,255,0.30)", flexShrink: 0, fontVariantNumeric: "tabular-nums" }}>
        {fmtMs(track.duration_ms)}
      </span>
      {onRemove && (
        <button
          onClick={onRemove}
          style={{
            display:        "flex",
            alignItems:     "center",
            justifyContent: "center",
            width:          20,
            height:         20,
            borderRadius:   4,
            border:         "none",
            background:     "transparent",
            color:          "rgba(255,255,255,0.30)",
            cursor:         "pointer",
            flexShrink:     0,
          }}
          title="Remove"
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "#fff"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.30)"; }}
        >
          <X size={11} strokeWidth={2.5} />
        </button>
      )}
    </div>
  );
}

// section header

function SectionHead({ label, onClear }: { label: string; onClear?: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 12px 4px" }}>
      <p style={{ margin: 0, fontSize: 10, fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "rgba(255,255,255,0.28)" }}>
        {label}
      </p>
      {onClear && (
        <button
          onClick={onClear}
          style={{ background: "none", border: "none", fontSize: 11, color: "rgba(255,255,255,0.35)", cursor: "pointer", padding: "0 2px" }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.65)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.35)"; }}
        >
          Clear
        </button>
      )}
    </div>
  );
}

// queuepanel

export function QueuePanel() {
  const toggleQueue     = usePlayerStore((s) => s.toggleQueue);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);

  const { queue, history, removeAt, reorder, clearQueue, clearHistory } = useQueueStore();

  const dragIdx = useRef<number | null>(null);

  function playItem(track: TrackItem) {
    setCurrentTrack(track);
    playTrack(track.id).catch(() => {});
  }

  return (
    <motion.div
      initial={{ width: 0, opacity: 0 }}
      animate={{ width: 272, opacity: 1 }}
      exit={{ width: 0, opacity: 0 }}
      transition={{ type: "spring", stiffness: 360, damping: 38, opacity: { duration: 0.18 } }}
      style={{
        flexShrink:    0,
        display:       "flex",
        flexDirection: "column",
        overflow:      "hidden",
        background:    "var(--color-sidebar)",
        borderLeft:    "1px solid var(--color-border)",
      }}
    >
      {/* inner column is fixed-width so content doesn't squish during the
          width animation */}
      <div style={{ width: 272, flexShrink: 0, display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      {/* header */}
      <div
        style={{
          display:        "flex",
          alignItems:     "center",
          justifyContent: "space-between",
          padding:        "0 12px",
          height:         40,
          borderBottom:   "1px solid var(--color-border)",
          flexShrink:     0,
        }}
      >
        <span style={{ fontSize: 13, fontWeight: 600, color: "rgba(255,255,255,0.88)" }}>Queue</span>
        <button
          onClick={toggleQueue}
          style={{
            display:        "flex",
            alignItems:     "center",
            justifyContent: "center",
            width:          24,
            height:         24,
            borderRadius:   6,
            border:         "none",
            background:     "transparent",
            color:          "rgba(255,255,255,0.40)",
            cursor:         "pointer",
          }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "#fff"; (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-hover)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.40)"; (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
        >
          <X size={13} strokeWidth={2} />
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto" }}>
        {/* now playing */}
        <SectionHead label="Now playing" />
        <div style={{ padding: "0 4px" }}>
          {currentTrack
            ? <QueueTrackRow track={currentTrack} />
            : <p style={{ margin: 0, padding: "4px 12px", fontSize: 12, color: "rgba(255,255,255,0.25)" }}>Nothing playing</p>
          }
        </div>

        {/* next up */}
        <SectionHead label="Next up" onClear={queue.length > 0 ? clearQueue : undefined} />
        <div style={{ padding: "0 4px" }}>
          {queue.length === 0 ? (
            <p style={{ margin: 0, padding: "4px 12px", fontSize: 12, color: "rgba(255,255,255,0.25)" }}>Queue empty</p>
          ) : (
            queue.map((track, i) => (
              <QueueTrackRow
                key={track.id + i}
                track={track}
                draggable
                onDragStart={() => { dragIdx.current = i; }}
                onDragOver={(e) => e.preventDefault()}
                onDrop={() => {
                  if (dragIdx.current !== null && dragIdx.current !== i) {
                    reorder(dragIdx.current, i);
                  }
                  dragIdx.current = null;
                }}
                onRemove={() => removeAt(i)}
              />
            ))
          )}
        </div>

        {/* history */}
        {history.length > 0 && (
          <>
            <div style={{ height: 1, background: "var(--color-divider)", margin: "8px 12px" }} />
            <SectionHead label="History" onClear={clearHistory} />
            <div style={{ padding: "0 4px" }}>
              {[...history].reverse().slice(0, 10).map((track, i) => (
                <div key={track.id + i} onClick={() => playItem(track)} style={{ cursor: "pointer" }}>
                  <QueueTrackRow track={track} dim />
                </div>
              ))}
            </div>
          </>
        )}
      </div>
      </div>
    </motion.div>
  );
}
