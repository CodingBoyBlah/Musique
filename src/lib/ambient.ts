/* Ambient for the immersive view.
 *
 * The room the record plays in is a moving mesh of the cover's colours, not the
 * cover. It was a blurred copy of the artwork for a long time, and however
 * heavy the blur, a sleeve with one big shape on it - a figure against a plain
 * wall - kept that shape behind the lyrics. A flat field painted from the
 * cover's hues fixed that but went the other way, averaging everything into
 * one tone. So the cover now lends a small palette - its main colours,
 * neutrals included, each with its share of the sleeve - and
 * components/layout/MeshRoom moves a soft pool of each around the window.
 *
 * Reading a cover is one tiny canvas draw and a few hundred pixels of
 * clustering, done once per cover and cached.
 */

import { hslToRgb, rgbToHsl, type RGB } from "./color";

/** one colour of the room's mesh, and how much of the sleeve it covers (0-1) */
export interface MeshColor {
  rgb: [number, number, number];
  share: number;
}

export interface Ambient {
  /** the room's palette, largest share first */
  mesh: MeshColor[];
  /** the floor under everything, tinted with the cover's dominant hue */
  base: string;
  /** the light the active lyric is lit by, as "r, g, b" */
  glow: string;
  /** lyric ink: near-white, carrying a trace of the cover's hue */
  ink: string;
  /** how bright the room gets where the lyrics sit, as relative luminance 0-1 */
  light: number;
}

export const AMBIENT_FALLBACK: Ambient = {
  mesh: [
    { rgb: [52, 44, 96], share: 0.4 },
    { rgb: [128, 76, 126], share: 0.25 },
    { rgb: [62, 92, 152], share: 0.2 },
    { rgb: [24, 20, 40], share: 0.15 },
  ],
  base: "#0a0910",
  glow: "198, 188, 255",
  ink: "246, 245, 252",
  light: 0,
};

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const trip = ({ r, g, b }: RGB) => `${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}`;

/** move a colour into a usable lightness/saturation band without shifting hue */
function grade(rgb: RGB, s: [number, number], l: number): RGB {
  const hsl = rgbToHsl(rgb);
  return hslToRgb({ h: hsl.h, s: clamp(hsl.s, s[0], s[1]), l });
}

const toLinear = (c: number) => {
  const v = c / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
};
const relLum = ([r, g, b]: [number, number, number]) =>
  0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);

type Px = [number, number, number];
const d2 = (a: readonly number[], b: readonly number[]) =>
  (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2;
const toTriple = ({ r, g, b }: RGB): Px => [Math.round(r), Math.round(g), Math.round(b)];

const MESH_K = 6;
const MESH_MAX = 5;
// two clusters closer than this (RGB distance) are one colour to the eye
const MESH_MERGE = 34;

/* The cover's palette, by k-means over the sampled pixels.
 *
 * Neutrals count. Hue-binning skips anything pale, dark or grey, which is why a
 * figure on a white wall gave an all-red room: the wall, most of the sleeve,
 * was simply not a colour. Seeding is farthest-point from the pixel nearest the
 * average - deterministic, so a cover always gives the same room, and it
 * reaches the small vivid accents a random seed would miss.
 *
 * Each colour is then graded the way the room needs it: a touch richer than
 * measured (pools blending into each other grey out), and held between
 * near-black and a soft light, so a white wall becomes pale light rather than a
 * glare the lyrics would have to fight. Tiny accents keep a floor share so they
 * still show as a pool of their own. */
function meshPalette(px: Px[]): MeshColor[] {
  const n = px.length;
  const mean = [0, 1, 2].map((c) => px.reduce((s, p) => s + p[c], 0) / n);

  let first = 0;
  for (let i = 1; i < n; i++) if (d2(px[i], mean) < d2(px[first], mean)) first = i;
  const centres: number[][] = [[...px[first]]];
  const near = px.map((p) => d2(p, centres[0]));
  while (centres.length < Math.min(MESH_K, n)) {
    let far = 0;
    for (let i = 1; i < n; i++) if (near[i] > near[far]) far = i;
    if (near[far] === 0) break;
    centres.push([...px[far]]);
    for (let i = 0; i < n; i++) near[i] = Math.min(near[i], d2(px[i], px[far]));
  }

  const owner = new Array<number>(n).fill(0);
  for (let iter = 0; iter < 10; iter++) {
    for (let i = 0; i < n; i++) {
      let best = 0;
      for (let k = 1; k < centres.length; k++) if (d2(px[i], centres[k]) < d2(px[i], centres[best])) best = k;
      owner[i] = best;
    }
    const sum = centres.map(() => [0, 0, 0, 0]);
    for (let i = 0; i < n; i++) {
      const s = sum[owner[i]];
      s[0] += px[i][0]; s[1] += px[i][1]; s[2] += px[i][2]; s[3]++;
    }
    sum.forEach((s, k) => { if (s[3]) centres[k] = [s[0] / s[3], s[1] / s[3], s[2] / s[3]]; });
  }

  const counts = centres.map(() => 0);
  owner.forEach((k) => counts[k]++);
  const ranked = centres
    .map((c, k) => ({ c, w: counts[k] / n }))
    .filter((x) => x.w > 0)
    .sort((a, b) => b.w - a.w);

  const merged: { c: number[]; w: number }[] = [];
  for (const x of ranked) {
    const twin = merged.find((m) => d2(m.c, x.c) < MESH_MERGE ** 2);
    if (twin) {
      const w = twin.w + x.w;
      twin.c = twin.c.map((v, i) => (v * twin.w + x.c[i] * x.w) / w);
      twin.w = w;
    } else merged.push({ c: [...x.c], w: x.w });
  }
  const clusters = merged.sort((a, b) => b.w - a.w).slice(0, MESH_MAX);

  const out: MeshColor[] = clusters.map(({ c, w }) => {
    const hsl = rgbToHsl({ r: c[0], g: c[1], b: c[2] });
    const g = hslToRgb({ h: hsl.h, s: clamp(hsl.s * 1.25, 0, 0.88), l: clamp(hsl.l, 0.07, 0.76) });
    return { rgb: toTriple(g), share: Math.max(0.05, w) };
  });

  /* A one- or two-colour sleeve still needs a mesh, not a flat sheet: lend it
     lighter and darker shades of its own main colour, a little apart in hue,
     rather than importing a stranger. */
  const [r0, g0, b0] = out[0].rgb;
  const seed = rgbToHsl({ r: r0, g: g0, b: b0 });
  const shades = [[0.14, 0.03], [-0.12, -0.03], [0.24, -0.05]];
  for (let i = 0; out.length < 4 && i < shades.length; i++) {
    const [dl, dh] = shades[i];
    const g = hslToRgb({ h: (seed.h + dh + 1) % 1, s: seed.s, l: clamp(seed.l + dl, 0.05, 0.76) });
    out.push({ rgb: toTriple(g), share: 0.14 });
  }
  return out;
}

/* How bright the room gets behind the lyrics, for the shade over that side.
 *
 * The pools wander, so every colour passes behind the lyrics sooner or later;
 * this blends the average the field settles at with its brightest colour, so
 * the shade is deep enough for the moment the palest pool drifts under the
 * type, not just for the average. */
function meshLight(mesh: MeshColor[]): number {
  const total = mesh.reduce((s, m) => s + m.share, 0);
  const avg = mesh.reduce((s, m) => s + relLum(m.rgb) * m.share, 0) / total;
  const peak = Math.max(...mesh.map((m) => relLum(m.rgb)));
  return avg * 0.6 + peak * 0.4;
}

const SAMPLE = 24;
const BINS = 16;

function read(img: CanvasImageSource): Ambient | null {
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

  // hue bins pick the lyric glow and ink: those want the cover's vivid colour
  const bin = Array.from({ length: BINS }, () => ({ r: 0, g: 0, b: 0, n: 0, s: 0 }));
  let allR = 0, allG = 0, allB = 0, allN = 0;
  const pixels: Px[] = [];

  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] < 200) continue;
    const r = data[i], g = data[i + 1], b = data[i + 2];
    allR += r; allG += g; allB += b; allN++;
    pixels.push([r, g, b]);
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
  const vivid = ranked.slice(0, 3).sort((a, b) => b.sat - a.sat)[0]?.rgb ?? mean;
  // a monochrome sleeve has no hue to lend, and forcing saturation onto one
  // would invent a colour out of whatever the grey rounds to
  const hasColour = ranked.length > 0;
  const hex = (v: number) => Math.round(v).toString(16).padStart(2, "0");
  const floor = grade(mean, [0.18, 0.5], 0.05);
  const mesh = meshPalette(pixels);

  return {
    mesh,
    light: meshLight(mesh),
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

/* Memory only. Re-reading a cover is one cached image decode and a tiny canvas
   draw, not worth a slot in localStorage. */
const mem = new Map<string, Ambient>();

/* Both the backdrop and the lyrics ask for the same cover on the same render -
   one wants the palette, the other the ink and the glow. Without this every
   track change decoded the image twice. */
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

/** the same room, read off whatever frame a canvas video is showing right now.
    null when the frame can't be read (not decoded yet, or a cross-origin video
    without CORS) - the caller keeps the cover's room then. */
export function ambientFromFrame(video: HTMLVideoElement): Ambient | null {
  if (video.readyState < 2 || !video.videoWidth) return null;
  return read(video);
}
