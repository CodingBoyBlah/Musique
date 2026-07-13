const PALETTES: [string, string, string, string][] = [
  ["#7C5CFF", "#4E8CFF", "#22D3EE", "#0B1020"], // indigo to cyan
  ["#FF5E7E", "#FF9E6D", "#FFC978", "#1A0B12"], // sunset
  ["#12D0A2", "#3AE0C0", "#8CF5D2", "#04140F"], // mint
  ["#FF6FA5", "#FF4D6D", "#C81E5B", "#1A0910"], // pink and red
  ["#5873D8", "#7C8BF0", "#B0A7F5", "#0B0C1A"], // periwinkle
  ["#F5A623", "#FF7A45", "#FF5E62", "#180B08"], // amber
  ["#3AA0FF", "#5ED0FF", "#9BE7FF", "#03101A"], // sky
  ["#9D5CFF", "#C86DFF", "#FF8AD6", "#120820"], // violet and magenta
];

function has(s: string): number {
  let h =  0;
  for (let i=0; i<s.length; i++) h = (h * 31+s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

export function meshPaletteIndex(seed: string): number {
  return hash(seed || "x") % PALETTES.length;
}

export function meshGradient(seed: string): string {
  const h = hash(seed || "x");
  const [a, b, c, base] = PALETTES[h % PALETTES.length];
  const p = (n: number) => 12 + ((h>>n)%74);

  return [
    `radial-gradient(65% 65% at ${p(1)}% ${p(3)}%, ${a} 0%, transparent 60%)`,
    `radial-gradient(55% 55% at ${p(5)}% ${p(2)}%, ${b} 0%, transparent 56%)`,
    `radial-gradient(72% 72% at ${p(4)}% ${p(6)}%, ${c} 0%, transparent 62%)`,
    `linear-gradient(140deg, ${base}, ${base})`,
    ].join(",");
}
