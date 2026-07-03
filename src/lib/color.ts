

export interface RGB { r: number; g: number; b: number; }

function clamp(v: number, lo: number, hi: number) { return Math.max(lo, Math.min(hi, v)); }



export function hexToRgb(hex: string): RGB {
  const h = hex.replace("#", "");
  const n = h.length === 3 ? h.split("").map((c) => c + c).join("") : h;
  const int = parseInt(n, 16);
  return { r: (int >> 16) & 255, g: (int >> 8) & 255, b: int & 255 };
}

export function rgbToHex({ r, g, b }: RGB): string {
  const h = (v: number) => Math.round(clamp(v, 0, 255)).toString(16).padStart(2, "0");
  return `#${h(r)}${h(g)}${h(b)}`;
}

export function rgbToHsl({ r, g, b }: RGB): { h: number; s: number; l: number } {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0, s = 0;
  const d = max - min;
  if (d !== 0) {
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h /= 6;
  }
  return { h, s, l };
}

export function hslToRgb({ h, s, l }: { h: number; s: number; l: number }): RGB {
  if (s === 0) { const v = l * 255; return { r: v, g: v, b: v }; }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const hue = (t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };
  return { r: hue(h + 1 / 3) * 255, g: hue(h) * 255, b: hue(h - 1 / 3) * 255 };
}


export function luminance({ r, g, b }: RGB): number {
  const f = (c: number) => { c /= 255; return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

const TARGET_LUM = 0.2; // avg relative luminance of #f22a43 / #5873d8
const MIN_S = 0.5;
const MAX_S = 0.9;

export function normalizeBrightness(rgb: RGB): RGB {
  const { h } = rgbToHsl(rgb);
  const s = clamp(rgbToHsl(rgb).s, MIN_S, MAX_S);
  let lo = 0.18, hi = 0.72, l = 0.45;
  for (let i = 0; i < 18; i++) {
    l = (lo + hi) / 2;
    if (luminance(hslToRgb({ h, s, l })) > TARGET_LUM) hi = l;
    else lo = l;
  }
  return hslToRgb({ h, s, l });
}

export function extractVibrant(img: HTMLImageElement): RGB | null {
  const S = 40;
  const canvas = document.createElement("canvas");
  canvas.width = S; canvas.height = S;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) return null;
  try {
    ctx.drawImage(img, 0, 0, S, S);
    const { data } = ctx.getImageData(0, 0, S, S);
    const N = 24; // finer hue buckets
    const w = new Array(N).fill(0);
    const sr = new Array(N).fill(0);
    const sg = new Array(N).fill(0);
    const sb = new Array(N).fill(0);
    let considered = 0, colored = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i + 3] < 200) continue;
      const r = data[i], g = data[i + 1], b = data[i + 2];
      const { h, s, l } = rgbToHsl({ r, g, b });
      if (l < 0.08 || l > 0.94) continue; // ignore only near-black / near-white
      considered++;
      if (s < 0.12) continue; // grayscale: counts as "considered" but casts no hue vote
      colored++;
      const weight = 1 + s * 1.5; // AREA-first, mild saturation nudge
      const bi = Math.min(N - 1, Math.floor(h * N));
      w[bi] += weight; sr[bi] += r * weight; sg[bi] += g * weight; sb[bi] += b * weight;
    }
    // truly monochrome cover -> don't fabricate an accent, keep the base
    if (considered === 0 || colored < considered * 0.03) return null;
    let best = -1, bw = 0;
    for (let i = 0; i < N; i++) if (w[i] > bw) { bw = w[i]; best = i; }
    if (best < 0 || bw === 0) return null;
    return { r: sr[best] / w[best], g: sg[best] / w[best], b: sb[best] / w[best] };
  } catch {
    return null;
  }
}


const memCache = new Map<string, string | null>();

export function loadCoverAccent(url: string): Promise<string | null> {
  if (memCache.has(url)) return Promise.resolve(memCache.get(url)!);
  const ls = localStorage.getItem("cover-accent-v2:" + url);
  if (ls !== null) { const v = ls || null; memCache.set(url, v); return Promise.resolve(v); }
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";   // spotifys i.scdn.co sends ACAO:* so canvas can read it
    img.referrerPolicy = "no-referrer";
    img.onload = () => {
      const raw = extractVibrant(img);
      const hex = raw ? rgbToHex(normalizeBrightness(raw)) : null;
      memCache.set(url, hex);
      try { localStorage.setItem("cover-accent:" + url, hex ?? ""); } catch { /* quota */ }
      resolve(hex);
    };
    img.onerror = () => { memCache.set(url, null); resolve(null); };
    img.src = url;
  });
}


export function dataUrlAccent(dataUrl: string): Promise<string | null> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      const raw = extractVibrant(img);
      resolve(raw ? rgbToHex(normalizeBrightness(raw)) : null);
    };
    img.onerror = () => resolve(null);
    img.src = dataUrl;
  });
}

export function applyAccent(baseHex: string | null): void {
  const root = document.documentElement;
  const vars = ["--color-accent", "--color-accent-hover", "--color-accent-dim", "--color-accent-text"];
  if (!baseHex) { vars.forEach((v) => root.style.removeProperty(v)); return; }
  const rgb = hexToRgb(baseHex);
  const { h, s, l } = rgbToHsl(rgb);
  const hover = rgbToHex(hslToRgb({ h, s, l: clamp(l + 0.1, 0, 0.85) }));
  const dim = `rgba(${Math.round(rgb.r)}, ${Math.round(rgb.g)}, ${Math.round(rgb.b)}, 0.16)`;
  const text = luminance(rgb) > 0.55 ? "#0a0a0f" : "#ffffff";
  root.style.setProperty("--color-accent", baseHex);
  root.style.setProperty("--color-accent-hover", hover);
  root.style.setProperty("--color-accent-dim", dim);
  root.style.setProperty("--color-accent-text", text);
}
