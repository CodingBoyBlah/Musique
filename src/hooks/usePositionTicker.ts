import { useEffect } from "react";
import { usePlayerStore } from "../store/player.store";

/* The playhead's local clock, owned in exactly one place.
 *
 * The backend reports position about once a second, so between reports the app
 * advances it itself. That used to live inside PlayerBar, which was fine only
 * for as long as exactly one PlayerBar existed - the moment the immersive view
 * mounted a second one, two intervals ran and the position advanced at twice
 * real time. The scrubber ran ahead, the lyric clock got dragged forward with
 * it, and every real position report snapped both back: the song appeared to
 * skip around.
 *
 * Mounting a component twice should not corrupt playback state, so the ticker
 * no longer lives in a component that can be mounted twice. Layout owns it. */
export function usePositionTicker() {
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const incrementPos = usePlayerStore((s) => s.incrementPos);

  useEffect(() => {
    if (!isPlaying) return;
    const timer = setInterval(incrementPos, 1000);
    return () => clearInterval(timer);
  }, [isPlaying, incrementPos]);
}
