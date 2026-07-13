import { useRef } from "react";
import { motion } from "framer-motion";
import { X, GripVertical, ListMusic } from "lucide-react";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { playTrack } from "../../api/playback";
import { fmtMs } from "../../utils/fmt";
import { meshGradient } from "../../lib/mesh";
import type { TrackItem } from "../../types/spotify";

const WIDTH = 272;

// small square cover, falls back to a seeded mesh gradient (no more grey box)
function Cover({ track, size }: { track: TrackItem; size: number }) {
  const art = track.album?.image_url;
  return art ? (
    <img
      src={art}
      alt=""
      loading="lazy"
      decoding="async"
      style={{ width: size, height: size, borderRadius: 6, objectFit: "cover", flexShrink: 0, outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1 }}
    />
  ) : (
    <div style={{ width: size, height: size, borderRadius: 6, flexShrink: 0, outline: "1px solid rgba(255,255,255,0.1)", outlineOffset: -1, overflow: "hidden", ...meshGradient(track.id) }} />
  );
}

function QueueTrackRow({
  track, onRemove, draggable, onDragStart, onDragOver, onDrop, dim,
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
        display: "flex", alignItems: "center", gap: 9,
        padding: "6px 8px", borderRadius: 9,
        opacity: dim ? 0.5 : 1,
        cursor: draggable ? "grab" : "default",
        transition: "background 0.12s, opacity 0.12s",
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLDivElement).style.background = "var(--color-surface-hover)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = "transparent"; }}
    >
      {draggable && (
        <span
          className="group-hover-visible"
          style={{ color: "rgba(255,255,255,0.22)", flexShrink: 0, display: "flex", alignItems: "center", marginLeft: -2 }}
        >
          <GripVertical size={13} strokeWidth={2} />
        </span>
      )}
      <Cover track={track} size={36} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ margin: 0, fontSize: 12.5, fontWeight: 500, color: "rgba(255,255,255,0.9)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.name}
        </p>
        <p style={{ margin: 0, fontSize: 11, color: "rgba(255,255,255,0.45)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.artists.map((a) => a.name).join(", ")}
        </p>
      </div>
      {onRemove ? (
        <button
          onClick={(e) => { e.stopPropagation(); onRemove(); }}
          className="queue-btn"
          style={{
            display: "flex", alignItems: "center", justifyContent: "center",
            width: 22, height: 22, borderRadius: 6, border: "none",
            background: "transparent", color: "rgba(255,255,255,0.35)",
            cursor: "pointer", flexShrink: 0, transition: "color 0.12s, background 0.12s",
          }}
          title="Remove"
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "#fff"; (e.currentTarget as HTMLButtonElement).style.background = "rgba(255,255,255,0.1)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.35)"; (e.currentTarget as HTMLButtonElement).style.background = "transparent"; }}
        >
          <X size={12} strokeWidth={2.5} />
        </button>
      ) : (
        <span style={{ fontSize: 11, color: "rgba(255,255,255,0.3)", flexShrink: 0, fontVariantNumeric: "tabular-nums" }}>
          {fmtMs(track.duration_ms)}
        </span>
      )}
    </div>
  );
}

// animated 3-bar equaliser for the now-playing card
function Equaliser() {
  return (
    <span style={{ display: "flex", alignItems: "flex-end", gap: 2, height: 13 }}>
      {[0, 1, 2].map((i) => (
        <motion.span
          key={i}
          animate={{ height: ["30%", "100%", "45%", "80%", "30%"] }}
          transition={{ duration: 0.9, repeat: Infinity, ease: "easeInOut", delay: i * 0.18 }}
          style={{ width: 3, borderRadius: 2, background: "var(--color-accent)", height: "30%" }}
        />
      ))}
    </span>
  );
}

function NowPlayingCard({ track, isPlaying }: { track: TrackItem; isPlaying: boolean }) {
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
        <p style={{ margin: "2px 0 0", fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {track.artists.map((a) => a.name).join(", ")}
        </p>
      </div>
      {isPlaying && <Equaliser />}
    </div>
  );
}

function SectionHead({ label, onClear }: { label: string; onClear?: () => void }) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "14px 12px 6px" }}>
      <p style={{ margin: 0, fontSize: 10.5, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "rgba(255,255,255,0.34)" }}>
        {label}
      </p>
      {onClear && (
        <button
          onClick={onClear}
          style={{ background: "none", border: "none", fontSize: 11, fontWeight: 600, color: "rgba(255,255,255,0.4)", cursor: "pointer", padding: "0 2px" }}
          onMouseEnter={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)"; }}
          onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.color = "rgba(255,255,255,0.4)"; }}
        >
          Clear
        </button>
      )}
    </div>
  );
}

function EmptyRow({ children }: { children: React.ReactNode }) {
  return (
    <p style={{ margin: 0, padding: "4px 12px 8px", fontSize: 12, color: "rgba(255,255,255,0.32)" }}>{children}</p>
  );
}

export function QueuePanel() {
  const toggleQueue     = usePlayerStore((s) => s.toggleQueue);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack    = usePlayerStore((s) => s.currentTrack);
  const isPlaying       = usePlayerStore((s) => s.isPlaying);

  const { queue, history, removeAt, reorder, clearQueue, clearHistory } = useQueueStore();
  const dragIdx = useRef<number | null>(null);

  function playItem(track: TrackItem) {
    setCurrentTrack(track);
    playTrack(track.id).catch(() => {});
  }

  return (
    <motion.div
      
      initial={{ x: WIDTH, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: WIDTH, opacity: 0 }}
      transition={{ x: { duration: 0.34, ease: [0.32, 0.72, 0, 1] }, opacity: { duration: 0.2 } }}
      style={{
        position: "absolute", top: 0, right: 0, bottom: 0, zIndex: 5,
        width: WIDTH, display: "flex", flexDirection: "column", overflow: "hidden",
        background: "var(--color-sidebar)", borderLeft: "1px solid var(--color-border)",
      }}
    >
      <div style={{ width: WIDTH, flexShrink: 0, display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
        {/* header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 12px", height: 48, flexShrink: 0, borderBottom: "1px solid var(--color-border)" }}>
          <span style={{ fontSize: 14, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>Queue</span>
        </div>

        <div className="scroll-y" style={{ flex: 1, overflowY: "auto", paddingBottom: 12 }}>
          {/* now playing */}
          <SectionHead label="Now playing" />
          {currentTrack
            ? <NowPlayingCard track={currentTrack} isPlaying={isPlaying} />
            : <EmptyRow>Nothing playing</EmptyRow>}

          {/* next up */}
          <SectionHead label="Next up" onClear={queue.length > 0 ? clearQueue : undefined} />
          {queue.length === 0 ? (
            <div style={{ margin: "0 12px", padding: "16px 14px", borderRadius: 10, border: "1.5px dashed var(--color-glass-border)", background: "var(--color-glass)", display: "flex", alignItems: "center", gap: 9, fontSize: 12, color: "var(--color-text-dim)" }}>
              <ListMusic size={14} strokeWidth={2} style={{ flexShrink: 0 }} />
              <span>Nothing queued. Add a song with the ＋ on any track.</span>
            </div>
          ) : (
            <div style={{ padding: "0 4px" }}>
              {queue.map((track, i) => (
                <QueueTrackRow
                  key={track.id + i}
                  track={track}
                  draggable
                  onDragStart={() => { dragIdx.current = i; }}
                  onDragOver={(e) => e.preventDefault()}
                  onDrop={() => {
                    if (dragIdx.current !== null && dragIdx.current !== i) reorder(dragIdx.current, i);
                    dragIdx.current = null;
                  }}
                  onRemove={() => removeAt(i)}
                />
              ))}
            </div>
          )}

          {/* history */}
          {history.length > 0 && (
            <>
              <SectionHead label="Recently played" onClear={clearHistory} />
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
