import { useEffect, useState } from "react";
import { coverUrl } from "../lib/coverUrl";
import { AMBIENT_FALLBACK, loadAmbient, type Ambient } from "../lib/ambient";

/* Read the cover once per track.
 *
 * The 300px CDN variant, not the 64px one: this image is the source of the
 * blurred backdrop as well as the palette, and 64px upscaled to fill a window
 * loses the artwork's structure entirely - which is the whole reason the
 * backdrop is the artwork rather than a gradient. 300px is also what the album
 * page already loads, so it is usually warm in the HTTP cache. */
export function useAmbient(url: string | null | undefined): Ambient {
  const [ambient, setAmbient] = useState<Ambient>(AMBIENT_FALLBACK);
  const src = coverUrl(url, 300) ?? url ?? null;

  useEffect(() => {
    let live = true;
    loadAmbient(src).then((a) => { if (live) setAmbient(a); });
    return () => { live = false; };
  }, [src]);

  return ambient;
}
