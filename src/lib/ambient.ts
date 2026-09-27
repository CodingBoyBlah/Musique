/* Ambient for the immersive view.
 *
 * The look we want is the one Cider has: the record's own artwork, blurred into
 * the room it is playing in. The mistake was never the blur - it was doing the
 * blur live. The old view stacked two copies of the 640px cover, each sized to
 * 150% of the viewport, each carrying `filter: blur(60px)`, both on an infinite
 * animation, with a `backdrop-filter: blur(48px)` panel over the top. A
 * backdrop-filter's input is whatever sits behind it, and what sat behind it was
 * moving, so it could never be cached: WebView2 re-ran a 48px gaussian across
 * half the window, over two live image blurs, sixty times a second.
 *
 * The correction is to blur the artwork exactly once, into a small bitmap, and
 * treat the result as an ordinary image from then on. A 160px canvas blurred
 * and then scaled up to fill the window is a far heavier blur than the 60px one
 * it replaces - the upscale is itself a smooth interpolation - and it keeps the
 * artwork's structure and colour where four flat gradients threw both away.
 * After that first frame nothing here costs anything: drift is a transform on a
 * plain image, which is the one thing a compositor does for free.
 */

import { hslToRgb, rgbToHsl, type RGB } from "./color";

export interface Ambient {
  /** the cover, blurred once into a data URL. "" if it could not be read */
  art: string;
  /** colour accents for depth over the blurred art, as "r, g, b" */
  blobs: [string, string, string];
  /** the floor under everything, tinted with the cover's dominant hue */
  base: string;
  /** the light the active lyric is lit by, as "r, g, b" */
  glow: string;
  /** lyric ink: near-white, carrying a trace of the cover's hue */
  ink: string;
}

export const AMBIENT_FALLBACK: Ambient = {
  art: "",
  blobs: ["86, 74, 148", "128, 76, 126", "62, 92, 152"],
  base: "#0a0910",
  glow: "198, 188, 255",
  ink: "246, 245, 252",
};

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const trip = ({ r, g, b }: RGB) => `${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}`;

/** move a colour into a usable lightness/saturation band without shifting hue */
function grade(rgb: RGB, s: [number, number], l: number): RGB {
  const hsl = rgbToHsl(rgb);
  return hslToRgb({ h: hsl.h, s: clamp(hsl.s, s[0], s[1]), l });
}

const BLUR_PX = 160;

/* Blur the cover into a bitmap.
 *
 * The image is drawn 70% larger than the canvas and centred, so that at the
 * canvas edge the gaussian still has real pixels to sample - draw it at exactly
 * the canvas size and it samples transparent black just outside the frame,
 * which rims the whole wash in a dark border. */
function preblur(img: HTMLImageElement): string {
  const cv = document.createElement("canvas");
  cv.width = BLUR_PX;
  cv.height = BLUR_PX;
  const ctx = cv.getContext("2d");
  if (!ctx) return "";

  const over = BLUR_PX * 0.7;
  /* Saturation lifted hard. Blurring is an average, and averaging pixels always
     pulls them toward grey - the version of this that shipped at 1.35 turned a
     hot pink sleeve into dead maroon on screen. The live filter this replaced
     ran at 1.8 for the same reason. */
  ctx.filter = "blur(7px) saturate(1.85)";
  ctx.drawImage(img, -over / 2, -over / 2, BLUR_PX + over, BLUR_PX + over);

  try {
    return cv.toDataURL("image/jpeg", 0.88);
  } catch {
    return ""; // tainted canvas
  }
}

const SAMPLE = 24;
const BINS = 16;

function read(img: HTMLImageElement): Ambient | null {
  const cv = document.createElement("canvas");
  cv.width = SAMPLE;
  cv.height = SAMPLE;
  const ctx = cv.getContext("2d", { willReadFrequently: true });
  if (!ctx) return null;

  let data: Uint8ClampedArray;
  try {
    ctx.drawImage(img, 0, 0, SAMPLE, SAMPLE);
    data = ctx.getImageData(0, 0, SAMPLE, SAMPLE).data;
  } catch {
    return null; // tainted canvas
  }

  const bin = Array.from({ length: BINS }, () => ({ r: 0, g: 0, b: 0, n: 0, s: 0 }));
  let allR = 0, allG = 0, allB = 0, allN = 0;

  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] < 200) continue;
    const r = data[i], g = data[i + 1], b = data[i + 2];
    allR += r; allG += g; allB += b; allN++;
    const { h, s, l } = rgbToHsl({ r, g, b });
    if (l < 0.09 || l > 0.93 || s < 0.13) continue;
    const k = Math.min(BINS - 1, Math.floor(h * BINS));
    const w = 0.35 + s; // area first, saturation as a nudge - as lib/color weights it
    bin[k].r += r * w; bin[k].g += g * w; bin[k].b += b * w; bin[k].n += w; bin[k].s += s * w;
  }
  if (!allN) return null;

  const ranked = bin
    .filter((c) => c.n > 0)
    .map((c) => ({ rgb: { r: c.r / c.n, g: c.g / c.n, b: c.b / c.n }, score: c.n, sat: c.s / c.n }))
    .sort((a, b) => b.score - a.score);

  const mean: RGB = { r: allR / allN, g: allG / allN, b: allB / allN };

  /* A duotone sleeve or a black-and-white photo would otherwise leave empty
     accents. Rotating the one hue it does have keeps the room reading as a
     single family rather than importing a stranger. */
  const picks: RGB[] = [];
  for (let i = 0; i < 3; i++) {
    if (ranked[i]) { picks.push(ranked[i].rgb); continue; }
    const seed = ranked[0]?.rgb ?? mean;
    const { h, s, l } = rgbToHsl(seed);
    picks.push(hslToRgb({ h: (h + (i - ranked.length) * 0.06 + 1) % 1, s, l }));
  }

  const vivid = ranked.slice(0, 3).sort((a, b) => b.sat - a.sat)[0]?.rgb ?? mean;
  // a monochrome sleeve has no hue to lend, and forcing saturation onto one
  // would invent a colour out of whatever the grey rounds to
  const hasColour = ranked.length > 0;
  const hex = (v: number) => Math.round(v).toString(16).padStart(2, "0");
  const floor = grade(mean, [0.18, 0.5], 0.05);

  return {
    art: preblur(img),
    blobs: picks.map((p, i) => trip(grade(p, [0.45, 0.85], 0.58 - i * 0.05))) as Ambient["blobs"],
    base: `#${hex(floor.r)}${hex(floor.g)}${hex(floor.b)}`,
    glow: trip(grade(vivid, [0.45, 0.78], 0.72)),
    /* Lyrics are not pure white - a trace of the cover's hue is what makes the
       type sit in the picture rather than on top of it.

       Deliberately faint. The reference swatches are stronger than this: a warm
       sleeve reads about #f2e3c4, which is HSL 40deg 64% 86%, and a green one
       #edf2dc at 46% 91%. Matching them was tried and pulled back twice, because
       a swatch in isolation is the wrong thing to match - the ambient behind the
       lyrics is sampled from the same artwork, so on a strongly single-hued
       sleeve the type and the room share a hue and the tint carries much further
       in place than the colour on its own suggests.

       At 93% lightness the gap between the lightest and darkest channel is about
       11/255 whatever saturation does, which is the point: a warm record lands on
       #f3eee8 and a cold one on #e8ebf3. Read as swatches those are all but white.
       In place they are not, because the whole panel behind them is the same hue -
       the eye reads the type against that, not against paper. If this is ever
       raised again, judge it in the view on a monochrome-ish blue sleeve, where
       type and room share a hue, and not against a swatch. */
    ink: hasColour ? trip(grade(vivid, [0.16, 0.30], 0.93)) : "246, 245, 250",
  };
}

/* Memory only. The blurred bitmap runs to several kilobytes of base64 and a
   listening session would evict everything else in localStorage inside an hour;
   re-reading it is one cached image decode and a canvas draw. */
const mem = new Map<string, Ambient>();

/* Both the backdrop and the lyrics ask for the same cover on the same render -
   one wants the blurred bitmap, the other the ink and the glow. Without this
   every track change decoded the image and ran the blur twice. */
const inflight = new Map<string, Promise<Ambient>>();

/** what is already in memory for this cover, synchronously - so a view that
    opens on a cover it has seen before paints the right room on its first
    frame instead of one frame of the fallback */
export function peekAmbient(url: string | null | undefined): Ambient | undefined {
  if (!url) return AMBIENT_FALLBACK;
  return mem.get(url);
}

export function loadAmbient(url: string | null | undefined): Promise<Ambient> {
  if (!url) return Promise.resolve(AMBIENT_FALLBACK);

  const hit = mem.get(url);
  if (hit) return Promise.resolve(hit);

  const pending = inflight.get(url);
  if (pending) return pending;

  const job = new Promise<Ambient>((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous"; // i.scdn.co sends ACAO:*, as lib/color already relies on
    img.referrerPolicy = "no-referrer";
    img.onload = () => {
      const out = read(img) ?? AMBIENT_FALLBACK;
      mem.set(url, out);
      resolve(out);
    };
    img.onerror = () => resolve(AMBIENT_FALLBACK);
    img.src = url;
  }).finally(() => inflight.delete(url));

  inflight.set(url, job);
  return job;
}
