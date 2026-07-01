

const FALLBACK = ["#3b3a52", "#5a4a6a", "#6a4a55", "#3f5068", "#4a4a4a"];

function rgbToHsl(r: number, g: number, b: number) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0, s = 0;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = ((g - b) / d + (g < b ? 6 : 0));
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
  }
  return { h, s, l };
}

export function extractPalette(url: string | null | undefined): Promise<string[]> {
  return new Promise((resolve) => {
    if (!url) return resolve(FALLBACK);
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      try {
        const S = 32;
        const cv = document.createElement("canvas");
        cv.width = S; cv.height = S;
        const ctx = cv.getContext("2d", { willReadFrequently: true });
        if (!ctx) return resolve(FALLBACK);
        ctx.drawImage(img, 0, 0, S, S);
        const { data } = ctx.getImageData(0, 0, S, S);

        // bucket by hue, keeping the vivid mid-tone pixels
        const buckets = new Map<number, { r: number; g: number; b: number; n: number; sat: number }>();
        for (let i = 0; i < data.length; i += 4) {
          const r = data[i], g = data[i + 1], b = data[i + 2], a = data[i + 3];
          if (a < 200) continue;
          const { h, s, l } = rgbToHsl(r, g, b);
          if (l < 0.12 || l > 0.92 || s < 0.16) continue;
          const key = Math.round(h / 30); // 12 hue bins, good enough
          const cur = buckets.get(key) ?? { r: 0, g: 0, b: 0, n: 0, sat: 0 };
          cur.r += r; cur.g += g; cur.b += b; cur.n++; cur.sat += s;
          buckets.set(key, cur);
        }

        let colors = [...buckets.values()]
          .map((c) => ({
            r: c.r / c.n, g: c.g / c.n, b: c.b / c.n,
            score: c.n * (0.4 + c.sat / c.n), // frequency x saturation
          }))
          .sort((a, b) => b.score - a.score)
          .slice(0, 5)
          .map((c) => `rgb(${Math.round(c.r)}, ${Math.round(c.g)}, ${Math.round(c.b)})`);

        if (colors.length < 3) colors = colors.concat(FALLBACK).slice(0, 4);
        resolve(colors);
      } catch {
        resolve(FALLBACK); // tainted canvas -> just use the fallback
      }
    };
    img.onerror = () => resolve(FALLBACK);
    img.src = url;
  });
}

export const FALLBACK_PALETTE = FALLBACK;
