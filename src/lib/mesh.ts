// Deterministic "mesh gradient" covers, Apple-Music-editorial style.
//
// Anywhere a track / section / shortcut has no real artwork we render soft,
// overlapping colour blobs seeded from a string instead of a flat surface with
// a grey icon (the old "red box + heart" look). Same seed -> same cover every
// render, so a given song always gets the same art.

const PALETTES: [string, string, string, string][] = [
  ["#7C5CFF", "#4E8CFF", "#22D3EE", "#0B1020"], // indigo  cyan
  ["#FF5E7E", "#FF9E6D", "#FFC978", "#1A0B12"], // sunset
  ["#12D0A2", "#3AE0C0", "#8CF5D2", "#04140F"], // mint 
  ["#FF6FA5", "#FF4D6D", "#C81E5B", "#1A0910"], // pink  red
  ["#5873D8", "#7C8BF0", "#B0A7F5", "#0B0C1A"], // periwinkle
  ["#F5A623", "#FF7A45", "#FF5E62", "#180B08"], // amber
  ["#3AA0FF", "#5ED0FF", "#9BE7FF", "#03101A"], // sky
  ["#9D5CFF", "#C86DFF", "#FF8AD6", "#120820"], // violet  magenta
];

function hash(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}


export function meshPaletteIndex(seed: string): number {
  return hash(seed || "x") % PALETTES.length;
}


export function meshGradient(seed: string): React.CSSProperties {
  const h = hash(seed || "x");
  const paletteIdx = h % PALETTES.length;
  const [a, b, c, base] = PALETTES[paletteIdx];

  
  const imgIdx = (h % 6) + 1;
  const rotation = (h % 4) * 90; // 0, 90, 180, 270 degrees
  const flip = (h % 2) === 0 ? "scaleX(-1)" : "scaleX(1)";

  
  return {
    backgroundImage: `url(/basegradient${imgIdx}.png), radial-gradient(circle at 0% 0%, ${a} 0%, transparent 70%), radial-gradient(circle at 100% 100%, ${b} 0%, transparent 70%), radial-gradient(circle at 50% 50%, ${c} 0%, transparent 70%), linear-gradient(140deg, ${base}, ${base})`,
    backgroundSize: "cover, 100% 100%, 100% 100%, 100% 100%, 100% 100%",
    backgroundPosition: "center",
    backgroundBlendMode: "overlay, normal, normal, normal, normal",
    backgroundColor: base,
    transform: `rotate(${rotation}deg) ${flip}`,
  };
}
