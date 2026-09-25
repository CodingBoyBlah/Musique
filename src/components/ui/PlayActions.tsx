import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { Shuffle, Pin, Link2, Globe, Plus, MoreHorizontal } from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { usePinsStore, type PinnedItem } from "../../store/pins.store";
import { useSpeedDialStore } from "../../store/speedDial.store";
import { playTrack, pausePlayback, resumeOrPlay } from "../../api/playback";
import type { TrackItem } from "../../types/spotify";
import { gpuLayer, zTransform } from "../../lib/motion";
import { useContextMenu } from "./ContextMenu";
import { shareSpotifyLink, shareUniversalLink, type ShareKind } from "../../lib/share";
import { Tooltip } from "./Tooltip";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";

interface Props {
  tracks:    TrackItem[];
  contextId: string;
  pinItem:   PinnedItem;
}

export function PlayActions({ tracks, contextId, pinItem }: Props) {
  const setCurrentTrack     = usePlayerStore((s) => s.setCurrentTrack);
  const currentTrack        = usePlayerStore((s) => s.currentTrack);
  const isPlaying           = usePlayerStore((s) => s.isPlaying);
  const sessionReady        = usePlayerStore((s) => s.sessionReady);
  const lyricsOpen          = usePlayerStore((s) => s.lyricsOpen);
  const queueOpen           = usePlayerStore((s) => s.queueOpen);
  const playContext         = useQueueStore((s) => s.playContext);
  const playContextShuffled = useQueueStore((s) => s.playContextShuffled);
  const activeContext       = useQueueStore((s) => s.contextId);
  const shuffle             = useQueueStore((s) => s.shuffle);
  const pins                = usePinsStore((s) => s.pins);
  const togglePin           = usePinsStore((s) => s.togglePin);
  const { open: openMenu, element: menuEl } = useContextMenu();

  // when the button row gets narrow or when lyrics/queue rail opens,
  // condense Play and Shuffle to circular icon buttons with smooth blur morph
  const rootRef = useRef<HTMLDivElement>(null);
  const [compact, setCompact] = useState(false);
  useEffect(() => {
    const el = rootRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width ?? 0;
      setCompact(w > 0 && w < 360);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const isCondensed = lyricsOpen || queueOpen || compact;

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
      const pos = sessionReady ? usePlayerStore.getState().positionMs : 0;
      resumeOrPlay(currentTrack.id, pos).catch(() => {});
      return;
    }
    const start = playContext(tracks, 0, contextId);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
      if (pinItem.type === "playlist") {
        useSpeedDialStore.getState().recordPlaylist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      } else if (pinItem.type === "album") {
        useSpeedDialStore.getState().recordAlbum({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      } else if (pinItem.type === "artist") {
        useSpeedDialStore.getState().recordArtist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      }
    }
  }

  function onShuffle() {
    const start = playContextShuffled(tracks, contextId);
    if (start) {
      setCurrentTrack(start);
      playTrack(start.id).catch(() => {});
      if (pinItem.type === "playlist") {
        useSpeedDialStore.getState().recordPlaylist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      } else if (pinItem.type === "album") {
        useSpeedDialStore.getState().recordAlbum({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      } else if (pinItem.type === "artist") {
        useSpeedDialStore.getState().recordArtist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url });
      }
    }
  }

  const shuffleActive = isActive && shuffle;

  const allEntries = [
    { label: pinned ? "Remove from sidebar" : "Pin to sidebar", icon: <Pin size={14} active={pinned} />, onSelect: () => togglePin(pinItem) },
    ...shareEntries,
  ];

  return (
    <div ref={rootRef} className="flex items-center mt-2" style={{ gap: 10, width: "100%" }}>
      <Tooltip label={playing ? "Pause" : "Play"} side="top">
        <motion.button
          initial={false}
          onClick={onPlay}
          disabled={empty}
          whileHover={empty ? {} : { scale: 1.04 }}
          whileTap={empty ? {} : { scale: 0.96 }}
          animate={{
            width: isCondensed ? 36 : 102,
            paddingLeft: isCondensed ? 0 : 18,
            paddingRight: isCondensed ? 0 : 18,
          }}
          transition={{ type: "spring", stiffness: 320, damping: 30 }}
          transformTemplate={zTransform}
          style={{
            ...gpuLayer,
            width: isCondensed ? 36 : 102,
            paddingLeft: isCondensed ? 0 : 18,
            paddingRight: isCondensed ? 0 : 18,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: 36,
            minWidth: 36,
            borderRadius: 99,
            border: "none",
            background: "#ffffff",
            color: "#000000",
            fontSize: 13.5,
            fontWeight: 700,
            letterSpacing: "-0.01em",
            cursor: empty ? "default" : "pointer",
            opacity: empty ? 0.5 : 1,
            boxShadow: playing
              ? "0 0 0 4px rgba(255, 255, 255, 0.25), 0 4px 16px rgba(0, 0, 0, 0.35)"
              : "0 2px 10px rgba(255, 255, 255, 0.20)",
            transition: "box-shadow 0.2s, background 0.15s",
            overflow: "hidden",
            whiteSpace: "nowrap",
          }}
        >
          <AnimatedPlayPause
            isPlaying={playing}
            size={15}
            strokeWidth={0}
            fill="currentColor"
          />
          <motion.span
            initial={false}
            animate={{
              maxWidth: isCondensed ? 0 : 50,
              opacity: isCondensed ? 0 : 1,
              filter: isCondensed ? "blur(6px)" : "blur(0px)",
              scale: isCondensed ? 0.75 : 1,
              marginLeft: isCondensed ? 0 : 7,
            }}
            transition={{
              maxWidth: { type: "spring", stiffness: 320, damping: 30 },
              marginLeft: { type: "spring", stiffness: 320, damping: 30 },
              opacity: { duration: 0.18, ease: [0.23, 1, 0.32, 1] },
            }}
            style={{
              maxWidth: isCondensed ? 0 : 50,
              opacity: isCondensed ? 0 : 1,
              marginLeft: isCondensed ? 0 : 7,
              display: "inline-block",
              overflow: "hidden",
              whiteSpace: "nowrap",
            }}
          >
            {playing ? "Pause" : "Play"}
          </motion.span>
        </motion.button>
      </Tooltip>

      <Tooltip label={shuffleActive ? "Shuffle active" : "Shuffle play"} side="top">
        <motion.button
          initial={false}
          onClick={onShuffle}
          disabled={empty}
          whileHover={empty ? {} : { scale: 1.04 }}
          whileTap={empty ? {} : { scale: 0.96 }}
          animate={{
            width: isCondensed ? 36 : 108,
            paddingLeft: isCondensed ? 0 : 16,
            paddingRight: isCondensed ? 0 : 16,
          }}
          transition={{ type: "spring", stiffness: 320, damping: 30 }}
          transformTemplate={zTransform}
          style={{
            ...gpuLayer,
            width: isCondensed ? 36 : 108,
            paddingLeft: isCondensed ? 0 : 16,
            paddingRight: isCondensed ? 0 : 16,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: 36,
            minWidth: 36,
            borderRadius: 99,
            border: shuffleActive
              ? "1.5px solid var(--color-accent)"
              : "1px solid rgba(255, 255, 255, 0.14)",
            background: shuffleActive
              ? "var(--color-accent-dim)"
              : "rgba(255, 255, 255, 0.08)",
            color: "#ffffff",
            fontSize: 13.5,
            fontWeight: 600,
            letterSpacing: "-0.01em",
            cursor: empty ? "default" : "pointer",
            opacity: empty ? 0.5 : 1,
            overflow: "hidden",
            whiteSpace: "nowrap",
            transition: "border 0.2s, background 0.2s",
          }}
        >
          <motion.span
            animate={{
              rotate: shuffleActive ? [0, -15, 15, 0] : 0,
            }}
            transition={{ duration: 0.35, ease: "easeOut" }}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              flexShrink: 0,
              color: shuffleActive ? "var(--color-accent-hover, #819af1)" : "#ffffff",
            }}
          >
            <Shuffle size={15} strokeWidth={2.2} />
          </motion.span>
          <motion.span
            initial={false}
            animate={{
              maxWidth: isCondensed ? 0 : 54,
              opacity: isCondensed ? 0 : 1,
              filter: isCondensed ? "blur(6px)" : "blur(0px)",
              scale: isCondensed ? 0.75 : 1,
              marginLeft: isCondensed ? 0 : 7,
            }}
            transition={{
              maxWidth: { type: "spring", stiffness: 320, damping: 30 },
              marginLeft: { type: "spring", stiffness: 320, damping: 30 },
              opacity: { duration: 0.18, ease: [0.23, 1, 0.32, 1] },
            }}
            style={{
              maxWidth: isCondensed ? 0 : 54,
              opacity: isCondensed ? 0 : 1,
              marginLeft: isCondensed ? 0 : 7,
              display: "inline-block",
              overflow: "hidden",
              whiteSpace: "nowrap",
            }}
          >
            Shuffle
          </motion.span>
        </motion.button>
      </Tooltip>

      <div style={{ flex: 1 }} />

      {/* Right actions: + Add and ... menu */}
      <Tooltip label={pinned ? "Pinned to sidebar" : "Pin to sidebar"} side="top">
        <motion.button
          onClick={() => togglePin(pinItem)}
          whileHover={{ scale: 1.04 }}
          whileTap={{ scale: 0.96 }}
          style={{
            height: 36,
            padding: "0 16px",
            borderRadius: 99,
            background: pinned ? "rgba(255, 255, 255, 0.16)" : "rgba(255, 255, 255, 0.08)",
            border: pinned ? "1px solid rgba(255, 255, 255, 0.28)" : "1px solid rgba(255, 255, 255, 0.14)",
            color: "#ffffff",
            fontSize: 13,
            fontWeight: 600,
            display: "flex",
            alignItems: "center",
            gap: 6,
            cursor: "pointer",
            flexShrink: 0,
            transition: "background 0.15s, border-color 0.15s",
          }}
        >
          <Plus size={14} strokeWidth={2.4} />
          <span>{pinned ? "Added" : "Add"}</span>
        </motion.button>
      </Tooltip>

      <Tooltip label="More options" side="top">
        <motion.button
          onClick={(e) => openMenu(allEntries)(e)}
          onContextMenu={openMenu(allEntries)}
          whileHover={{ scale: 1.06 }}
          whileTap={{ scale: 0.94 }}
          transformTemplate={zTransform}
          style={{
            ...gpuLayer,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: 36,
            height: 36,
            borderRadius: "50%",
            border: "1px solid rgba(255, 255, 255, 0.14)",
            background: "rgba(255, 255, 255, 0.08)",
            color: "var(--color-text-hi)",
            cursor: "pointer",
            transition: "background 0.15s, border-color 0.15s",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.14)";
            (e.currentTarget as HTMLElement).style.borderColor = "rgba(255, 255, 255, 0.22)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLElement).style.background = "rgba(255, 255, 255, 0.08)";
            (e.currentTarget as HTMLElement).style.borderColor = "rgba(255, 255, 255, 0.14)";
          }}
        >
          <MoreHorizontal size={16} strokeWidth={2} />
        </motion.button>
      </Tooltip>

      {menuEl}
    </div>
  );
}
