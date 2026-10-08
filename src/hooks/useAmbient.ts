import { useEffect, useMemo, useState } from "react";
import { coverUrl } from "../lib/coverUrl";
import { AMBIENT_FALLBACK, loadAmbient, peekAmbient, type Ambient } from "../lib/ambient";

export interface AmbientState extends Ambient {
  /** false only before the very first cover has been read. the room should
      not be lit off the fallback palette while it is simply still loading */
  ready: boolean;
}

/* Read the cover once per track.
 *
 * The 300px CDN variant, not the 64px one: this image is the source of the
 * blurred backdrop as well as the palette, and 64px upscaled to fill a window
 * loses the artwork's structure entirely - which is the whole reason the
 * backdrop is the artwork rather than a gradient. 300px is also what the album
 * page already loads, so it is usually warm in the HTTP cache.
 *
 * Seeded synchronously from the in-memory cache, so a cover that has been read
 * before is right on the first frame. On a track change the previous room is
 * held until the next one is ready, never swapped for the fallback in between;
 * the backdrop crossfades between the two. */
export function useAmbient(url: string | null | undefined): AmbientState {
  const src = coverUrl(url, 300) ?? url ?? null;
  const [state, setState] = useState(() => {
    const hit = peekAmbient(src);
    return { src, ambient: hit ?? AMBIENT_FALLBACK, ready: !!hit };
  });

  // a cover we already hold: adopt it during render, no in-between frame
  if (state.src !== src) {
    const hit = peekAmbient(src);
    if (hit) setState({ src, ambient: hit, ready: true });
  }

  useEffect(() => {
    let live = true;
    loadAmbient(src).then((a) => {
      if (live) setState((s) => (s.src === src && s.ambient === a && s.ready ? s : { src, ambient: a, ready: true }));
    });
    return () => { live = false; };
  }, [src]);

  return useMemo(() => ({ ...state.ambient, ready: state.ready }), [state.ambient, state.ready]);
}
