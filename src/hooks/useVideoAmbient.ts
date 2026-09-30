import { useEffect, useState } from "react";
import { ambientFromFrame, type Ambient } from "../lib/ambient";

// how often the room re-reads the video. the backdrop crossfades between reads,
// so this is the pace the colours drift at, not a frame rate
const EVERY_MS = 1500;

/* the ambient room lit by a playing canvas video instead of the static cover:
colours follow what's on screen. returns null until a frame has been read (or
if it can't be), so the caller falls back to the cover. */
export function useVideoAmbient(video: HTMLVideoElement | null): Ambient | null {
  const [ambient, setAmbient] = useState<Ambient | null>(null);

  useEffect(() => {
    setAmbient(null);
    if (!video) return;
    let dead = false;
    const sample = () => {
      if (dead || video.paused || document.hidden) return;
      const a = ambientFromFrame(video);
      if (a && a.art) setAmbient(a);
    };
    const onReady = () => sample();
    video.addEventListener("loadeddata", onReady);
    video.addEventListener("playing", onReady);
    const iv = window.setInterval(sample, EVERY_MS);
    sample();
    return () => {
      dead = true;
      window.clearInterval(iv);
      video.removeEventListener("loadeddata", onReady);
      video.removeEventListener("playing", onReady);
    };
  }, [video]);

  return ambient;
}
