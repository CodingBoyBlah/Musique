import { motion } from "framer-motion";
import { Play, Pause, Shuffle, Pin, Share2, Link2, Globe } from "lucide-react";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { usePinsStore, type PinnedItem } from "../../store/pins.store";
import { playTrack, pausePlayback, resumeOrPlay } from "../../api/playback";
import type { TrackItem } from "../../types/spotify";
import { gpuLayer, zTransform } from "../../lib/motion";
import { useContextMenu } from "./ContextMenu";
import { shareSpotifyLink, shareUniversalLink, type ShareKind } from "../../lib/share";

interface Props {
  tracks:    TrackItem[];
  contextId: string;
  pinItem:   PinnedItem;
}

export function PlayActions({ tracks, contextId, pinItem }: Props) {
  const setCurrentTrack     = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack        = usePlayerStore((s) => s.currentTrack);
  const isPlaying           = usePlayerStore((s) => s.isPlaying);
  const positionMs          = usePlayerStore((s) => s.positionMs);
  const sessionReady        = usePlayerStore((s) => s.sessionReady);
  const playContext         = useQueueStore((s) => s.playContext);
  const playContextShuffled = useQueueStore((s) => s.playContextShuffled);
  const activeContext       = useQueueStore((s) => s.contextId);
  const shuffle             = useQueueStore((s) => s.shuffle);
  const pins                = usePinsStore((s) => s.pins);
  const togglePin           = usePinsStore((s) => s.togglePin);
  const { open: openMenu, element: menuEl } = useContextMenu();

  const shareKind = pinItem.type as ShareKind;
  const shareEntries = [
    { label: "Copy Spotify link",   icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink(shareKind, pinItem.id) },
    { label: "Copy universal link", icon: <Globe size={14} />, onSelect: () => shareUniversalLink(shareKind, pinItem.id) },
  ];

  const empty   = tracks.length === 0;
  const isActive = activeContext === contextId;          // this context is the loaded one
  const playing  = isActive && isPlaying;
  const pinned   = pins.some((p) => p.id === pinItem.id);

  function onPlay() {
    if (playing) { pausePlayback().catch(() => {}); return; }
    if (isActive && currentTrack) {
      resumeOrPlay(currentTrack.id, sessionReady ? positionMs : 0).catch(() => {});
      return;
    }
    const start = playContext(tracks, 0, contextId);
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  function onShuffle() {
    const start = playContextShuffled(tracks, contextId);
    if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); }
  }

  const shuffleActive = isActive && shuffle;

  return (
    <div className="flex items-center mt-3" style={{ gap: 12, flexWrap: "wrap", rowGap: 12 }}>
      <motion.button
        layout="position"
        onClick={onPlay}
        disabled={empty}
        whileHover={empty ? {} : { scale: 1.04 }}
        whileTap={empty ? {} : { scale: 0.96 }}
        transformTemplate={zTransform}
        style={{
          ...gpuLayer,
          flexShrink: 0,
          display: "flex", alignItems: "center", gap: 8, height: 44, padding: "0 24px",
          borderRadius: 99, border: "none", background: "var(--color-accent)",
          color: "#fff", fontSize: 14, fontWeight: 700,
          cursor: empty ? "default" : "pointer", opacity: empty ? 0.5 : 1,
          boxShadow: playing ? "0 0 0 4px var(--color-accent-dim)" : "none",
        }}
      >
        {playing
          ? <><Pause size={17} fill="currentColor" strokeWidth={0} /> Pause</>
          : <><Play  size={17} fill="currentColor" strokeWidth={0} /> Play</>}
      </motion.button>

      <motion.button
        layout="position"
        onClick={onShuffle}
        disabled={empty}
        title="Shuffle play"
        whileHover={empty ? {} : { scale: 1.04 }}
        whileTap={empty ? {} : { scale: 0.96 }}
        transformTemplate={zTransform}
        style={{
          ...gpuLayer,
          flexShrink: 0,
          display: "flex", alignItems: "center", gap: 8, height: 44, padding: "0 18px",
          borderRadius: 99,
          border: shuffleActive ? "1px solid transparent" : "1px solid var(--color-border)",
          background: shuffleActive ? "var(--color-accent-dim)" : "var(--color-surface)",
          color: shuffleActive ? "var(--color-accent)" : "var(--color-text-hi)",
          fontSize: 14, fontWeight: 600,
          cursor: empty ? "default" : "pointer", opacity: empty ? 0.5 : 1,
        }}
      >
        <Shuffle size={16} /> Shuffle
      </motion.button>

      <motion.button
        layout="position"
        onClick={() => togglePin(pinItem)}
        title={pinned ? "Unpin" : "Pin to sidebar"}
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.94 }}
        transformTemplate={zTransform}
        style={{
          ...gpuLayer,
          flexShrink: 0,
          display: "flex", alignItems: "center", justifyContent: "center", width: 44, height: 44,
          borderRadius: "50%",
          border: pinned ? "none" : "1px solid var(--color-border)",
          background: pinned ? "var(--color-accent)" : "transparent",
          color: pinned ? "#fff" : "var(--color-text)", cursor: "pointer",
        }}
      >
        <Pin size={17} fill={pinned ? "currentColor" : "none"} />
      </motion.button>

      <motion.button
        layout="position"
        onClick={(e) => openMenu(shareEntries)(e)}
        onContextMenu={openMenu(shareEntries)}
        title="Share"
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.94 }}
        transformTemplate={zTransform}
        style={{
          ...gpuLayer,
          flexShrink: 0,
          display: "flex", alignItems: "center", justifyContent: "center", width: 44, height: 44,
          borderRadius: "50%", border: "1px solid var(--color-border)",
          background: "transparent", color: "var(--color-text)", cursor: "pointer",
        }}
      >
        <Share2 size={17} />
      </motion.button>

      {menuEl}
    </div>
  );
}
