import { useSyncExternalStore } from "react";
import { usePlayerStore } from "../store/player.store";

/* reflow pulse

framer-motion layout animations only fire when the motion component itself
re-renders (framer snapshots the old box on render, measures the new one
after commit, animates the delta). grid cards are memoised and their pages
dont subscribe to window size or the side panels - so when the grid
re-columns (resize, lyrics/queue panel opens) the card boxes move via CSS but
no react render happens, framer never re-measures, cards teleport instead of
gliding. annoying as hell to track down.

this hook hands every card a cheap subscription to exactly those two events
so it re-renders the instant the grid can reflow. memo stays intact (it only
blocks parent-prop re-renders, an internal hook update always renders).

resize listener is shared across all cards + rAF-coalesced, so N cards cost
one listener + one render pass per frame, not N listeners. */

let listeners = new Set<() => void>();
let tick = 0;
let raf = 0;

function onResize() {
  if (raf) return;
  raf = requestAnimationFrame(() => {
    raf = 0;
    tick++;
    listeners.forEach((l) => l());
  });
}

function subscribe(cb: () => void) {
  if (listeners.size === 0) window.addEventListener("resize", onResize);
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
    if (listeners.size === 0) {
      window.removeEventListener("resize", onResize);
      if (raf) { cancelAnimationFrame(raf); raf = 0; }
    }
  };
}

function getSnapshot() {
  return tick;
}


export function useReflowPulse(): void {
  // resize -> grid re-columns
  useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  // side panels eat horizontal space -> grid re-columns. selecting these  subscribes the caller to the toggles.
  usePlayerStore((s) => s.lyricsOpen);
  usePlayerStore((s) => s.queueOpen);
}
