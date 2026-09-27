import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Shuffle, Pin, Link2, Globe, Share2 } from "@/lib/icons";
import { usePlayerStore } from "../../store/player.store";
import { useQueueStore } from "../../store/queue.store";
import { usePinsStore, type PinnedItem } from "../../store/pins.store";
import { useSpeedDialStore } from "../../store/speedDial.store";
import { usePrefsStore } from "../../store/prefs.store";
import { playTrack, pausePlayback, resumeOrPlay } from "../../api/playback";
import { remotePlayContext, remoteSetShuffle } from "../../api/connect";
import { toast } from "../../store/toast.store";
import type { TrackItem } from "../../types/spotify";
import { EASE_OUT, PRESS, PRESS_TRANSITION, REFLOW_SPRING, zTransform } from "../../lib/motion";
import "../../styles/ui.css";
import { useContextMenu } from "./ContextMenu";
import { shareSpotifyLink, shareUniversalLink, type ShareKind } from "../../lib/share";
import { Tooltip } from "./Tooltip";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";

// the page this row plays. Artists can't be pinned to the sidebar, so an
// artist page passes its own right-hand control (Follow) as `accessory`.
type ContextItem = PinnedItem | (Omit<PinnedItem, "type"> & { type: "artist" | "show" });

interface Props {
  tracks:     TrackItem[];
  contextId:  string;
  pinItem:    ContextItem;
  accessory?: ReactNode;
  /* how to start playback once the context is queued. defaults to playing
     the first item from the top; podcasts pass one that resumes the episode
     where you left off */
  onStart?:   (start: TrackItem) => void;
  // a podcast in shuffled order makes no sense
  hideShuffle?: boolean;
}

export function PlayActions({ tracks, contextId, pinItem, accessory, onStart, hideShuffle }: Props) {
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
  const sidebarMode         = usePrefsStore((s) => s.sidebarMode);
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

  const friendsOpen = usePlayerStore((s) => s.friendsOpen);
  const isCondensed = lyricsOpen || queueOpen || friendsOpen || compact;

  const shareKind = pinItem.type as ShareKind;
  const shareEntries = [
    { label: "Copy Spotify link",   icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink(shareKind, pinItem.id) },
    { label: "Copy universal link", icon: <Globe size={14} />, onSelect: () => shareUniversalLink(shareKind, pinItem.id) },
  ];

  const empty   = tracks.length === 0;
  const isActive = activeContext === contextId;          // this context is the loaded one
  const playing  = isActive && isPlaying;
  const pinned   = pins.some((p) => p.id === pinItem.id);
  // with the sidebar listing every playlist instead of pins, a pin has
  // nowhere to show up, so the button goes and Share takes its slot
  const pinnable = (pinItem.type === "playlist" || pinItem.type === "album") && sidebarMode !== "playlists";

  // another device is playing: start the real context over there so its own
  // next/prev/shuffle walk the album/playlist, not a one-track queue
  function playRemote(shuffled: boolean): boolean {
    const p = usePlayerStore.getState();
    if (!p.isRemotePlayback || !/^[A-Za-z0-9]{22}$/.test(pinItem.id)) return false;
    const deviceId = p.activeDevice?.id ?? null;
    const go = async () => {
      if (shuffled) {
        await remoteSetShuffle(true).catch(() => {});
        p.setRemoteShuffle(true);
      }
      await remotePlayContext({ contextUri: `spotify:${pinItem.type}:${pinItem.id}`, deviceId });
    };
    go().catch(() => toast.error(`Couldn't play on ${p.activeDevice?.name ?? "the remote device"}`));
    return true;
  }

  function onPlay() {
    if (playing) { pausePlayback().catch(() => {}); return; }
    if (!isActive && playRemote(false)) return;
    if (isActive && currentTrack) {
      const pos = sessionReady ? usePlayerStore.getState().positionMs : 0;
      resumeOrPlay(currentTrack.id, pos).catch(() => {});
      return;
    }
    const start = playContext(tracks, 0, contextId);
    if (start) {
      setCurrentTrack(start);
      if (onStart) onStart(start);
      else playTrack(start.id).catch(() => {});
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
    if (playRemote(true)) return;
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

  // no pin button to sit beside: Share becomes a labelled pill of the same
  // size. an artist page has Follow there instead, so it keeps the icon
  const shareLabelled = !pinnable && pinItem.type !== "artist" && pinItem.type !== "show";

  return (
    <div ref={rootRef} className="flex items-center mt-2" style={{ gap: 10, width: "100%" }}>
      <Tooltip label={playing ? "Pause" : "Play"} side="top">
        {/* The condense morph runs as a layout (transform) animation: the width
            flips once and framer tweens the difference as a scale, with the
            icon and label counter-scaled so they never stretch. Nothing
            relayouts per frame, and the label simply fades - no blur or scale
            on text, which read as a smear. */}
        <motion.button
          layout
          initial={false}
          onClick={onPlay}
          disabled={empty}
          whileTap={empty ? undefined : PRESS}
          transition={{ layout: REFLOW_SPRING, default: PRESS_TRANSITION }}
          className="focus-ring"
          style={{
            width: isCondensed ? 36 : 102,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: 36,
            minWidth: 36,
            padding: 0,
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
            overflow: "hidden",
            whiteSpace: "nowrap",
          }}
        >
          <motion.span layout="position" transition={{ layout: REFLOW_SPRING }} style={{ display: "flex" }}>
            <AnimatedPlayPause
              isPlaying={playing}
              size={15}
              strokeWidth={0}
              fill="currentColor"
            />
          </motion.span>
          <AnimatePresence initial={false} mode="popLayout">
            {!isCondensed && (
              <motion.span
                key="label"
                layout="position"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ opacity: { duration: 0.15, ease: EASE_OUT }, layout: REFLOW_SPRING }}
                style={{ marginLeft: 7, display: "inline-block", whiteSpace: "nowrap" }}
              >
                {playing ? "Pause" : "Play"}
              </motion.span>
            )}
          </AnimatePresence>
        </motion.button>
      </Tooltip>

      {!hideShuffle && <Tooltip label={shuffleActive ? "Shuffle active" : "Shuffle play"} side="top">
        <motion.button
          layout
          initial={false}
          onClick={onShuffle}
          disabled={empty}
          whileTap={empty ? undefined : PRESS}
          transition={{ layout: REFLOW_SPRING, default: PRESS_TRANSITION }}
          className="focus-ring"
          style={{
            width: isCondensed ? 36 : 108,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: 36,
            minWidth: 36,
            padding: 0,
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
            transition: "border-color 0.2s, background-color 0.2s",
          }}
        >
          <motion.span
            layout="position"
            animate={{
              rotate: shuffleActive ? [0, -15, 15, 0] : 0,
            }}
            transition={{ rotate: { duration: 0.35, ease: EASE_OUT }, layout: REFLOW_SPRING }}
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
          <AnimatePresence initial={false} mode="popLayout">
            {!isCondensed && (
              <motion.span
                key="label"
                layout="position"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ opacity: { duration: 0.15, ease: EASE_OUT }, layout: REFLOW_SPRING }}
                style={{ marginLeft: 7, display: "inline-block", whiteSpace: "nowrap" }}
              >
                Shuffle
              </motion.span>
            )}
          </AnimatePresence>
        </motion.button>
      </Tooltip>}

      <div style={{ flex: 1 }} />

      {/* Right actions: Pin and Share. The button pins to the sidebar, so it
          says Pin - it used to say Add with a plus, which read as adding to
          the library. */}
      {accessory}

      {pinnable && <Tooltip label={pinned ? "Unpin from sidebar" : "Pin to sidebar"} side="top">
        <motion.button
          onClick={() => togglePin(pinItem as PinnedItem)}
          aria-pressed={pinned}
          className="ghost-pill focus-ring"
          data-on={pinned}
          whileTap={PRESS}
          transition={PRESS_TRANSITION}
          style={{
            height: 36,
            padding: "0 16px",
            borderRadius: 99,
            color: "#ffffff",
            fontSize: 13,
            fontWeight: 600,
            display: "flex",
            alignItems: "center",
            gap: 6,
            cursor: "pointer",
            flexShrink: 0,
          }}
        >
          <Pin size={14} strokeWidth={2.2} active={pinned} />
          <span>{pinned ? "Pinned" : "Pin"}</span>
        </motion.button>
      </Tooltip>}

      <Tooltip label="Share" side="top">
        <motion.button
          // a click anchors the menu under this button (see useContextMenu)
          onClick={(e) => openMenu(shareEntries)(e)}
          onContextMenu={openMenu(shareEntries)}
          aria-label="Share"
          aria-haspopup="menu"
          className="ghost-pill focus-ring"
          whileTap={PRESS}
          transition={PRESS_TRANSITION}
          transformTemplate={zTransform}
          style={shareLabelled ? {
            height: 36,
            padding: "0 16px",
            borderRadius: 99,
            color: "#ffffff",
            fontSize: 13,
            fontWeight: 600,
            display: "flex",
            alignItems: "center",
            gap: 6,
            cursor: "pointer",
            flexShrink: 0,
          } : {
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: 36,
            height: 36,
            borderRadius: "50%",
            color: "var(--color-text-hi)",
            cursor: "pointer",
          }}
        >
          <Share2 size={shareLabelled ? 14 : 15} strokeWidth={2.2} />
          {shareLabelled && <span>Share</span>}
        </motion.button>
      </Tooltip>

      {menuEl}
    </div>
  );
}
