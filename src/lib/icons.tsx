import React, { useId } from "react";

/* the app's icon set. hand drawn on a 24 grid in one style: round caps and
 * joins, soft corners, circles as true arcs. same names and props as the
 * lucide set it replaced, so call sites never changed.
 *
 * every icon is ONE <path>. crossing strokes inside a single path paint once,
 * so an icon drawn in an rgba colour never goes lighter/darker where two of
 * its lines overlap (separate <path>/<circle> elements each apply the alpha).
 *
 * `active` swaps an icon to its filled glyph (see SOLID) with a short fade:
 * outline when idle, filled when the thing it stands for is on / selected.
 * the filled glyph is built as a mask and painted once, so it has no alpha
 * seams either.
 */

export interface IconProps extends Omit<React.SVGProps<SVGSVGElement>, "stroke" | "fill"> {
  size?: number | string;
  strokeWidth?: number | string;
  color?: string;
  fill?: string;
  /* show the filled glyph. leave undefined for icons that never toggle, so
     they skip the mask entirely */
  active?: boolean;
}
export type LucideProps = IconProps;
export type LucideIcon = React.ComponentType<IconProps>;

// geometry helpers

const n = (v: number) => +v.toFixed(3);

const circle = (cx: number, cy: number, r: number) =>
  `M${n(cx - r)} ${n(cy)}a${r} ${r} 0 1 0 ${n(2 * r)} 0a${r} ${r} 0 1 0 ${n(-2 * r)} 0`;

const rrect = (x: number, y: number, w: number, h: number, r: number) =>
  `M${n(x + r)} ${n(y)}h${n(w - 2 * r)}a${r} ${r} 0 0 1 ${r} ${r}v${n(h - 2 * r)}` +
  `a${r} ${r} 0 0 1 ${-r} ${r}h${n(-(w - 2 * r))}a${r} ${r} 0 0 1 ${-r} ${-r}` +
  `v${n(-(h - 2 * r))}a${r} ${r} 0 0 1 ${r} ${-r}z`;

// a dot that reads as a solid disc once stroked
const dot = (x: number, y: number) => circle(x, y, 0.7);

function gear(teeth: number, inner: number, outer: number) {
  const pt = (r: number, deg: number) => {
    const a = (deg * Math.PI) / 180;
    return `${n(12 + r * Math.cos(a))} ${n(12 + r * Math.sin(a))}`;
  };
  const step = 360 / teeth;
  let d = "";
  for (let k = 0; k < teeth; k++) {
    const t = k * step - 90;
    d += `${k === 0 ? "M" : "L"}${pt(inner, t - 17)}L${pt(outer, t - 9)}L${pt(outer, t + 9)}L${pt(inner, t + 17)}`;
    d += `A${inner} ${inner} 0 0 1 ${pt(inner, t + step - 17)}`;
  }
  return d + "z";
}

// shared outlines (the sidebar's NavGlyph uses these too)

const PERSON_BODY = "M4.5 19.6c0-3.4 3.3-5.9 7.5-5.9s7.5 2.5 7.5 5.9a.9.9 0 0 1-.9.9H5.4a.9.9 0 0 1-.9-.9z";
const VOLUME_HORN =
  "M4.5 9h2.6l4.1-3.6a.9.9 0 0 1 1.5.7v11.8a.9.9 0 0 1-1.5.7L7.1 15H4.5A1.5 1.5 0 0 1 3 13.5v-3A1.5 1.5 0 0 1 4.5 9z";
const REPEAT = "M17 3l3 3-3 3M20 6H8a4 4 0 0 0-4 4v1M7 21l-3-3 3-3M4 18h12a4 4 0 0 0 4-4v-1";
const PANEL = rrect(3, 4, 18, 16, 3) + "M9.5 4v16";
// monitor with a phone in front. the monitor's edges stop short of the phone
const DEVICES_MONITOR = "M12.5 15H4.5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h11a2 2 0 0 1 2 2v1";
const DEVICES_PHONE = rrect(14.5, 9.5, 7, 11, 1.8);
const QUEUE_PLAY = "M4 5.2v4.6a.6.6 0 0 0 .9.5l3.7-2.3a.6.6 0 0 0 0-1L4.9 4.7a.6.6 0 0 0-.9.5z";
const EYE = "M2.5 12C4.5 7.8 8 5.5 12 5.5s7.5 2.3 9.5 6.5c-2 4.2-5.5 6.5-9.5 6.5S4.5 16.2 2.5 12z";

export const iconPaths = {
  AlertTriangle: "M10.27 4.5a2 2 0 0 1 3.46 0l7.5 13a2 2 0 0 1-1.73 3H4.5a2 2 0 0 1-1.73-3zM12 9.5v4M12 17h.01",
  ArrowRight: "M5 12h14M13 6l6 6-6 6",
  ArrowUpRight: "M7 17 17 7M8.5 7H17v8.5",
  Captions: rrect(3, 5, 18, 14, 3) + "M7 12h3.5M13.5 12h3.5M7 15.5h6M16 15.5h1",
  Cast:
    "M2.5 8V6.5A2.5 2.5 0 0 1 5 4h14a2.5 2.5 0 0 1 2.5 2.5v11A2.5 2.5 0 0 1 19 20h-5" +
    "M2.5 12.5a7.5 7.5 0 0 1 7.5 7.5M2.5 16.5A3.5 3.5 0 0 1 6 20M2.5 20h.01",
  Check: "M5 12.5l4.5 4.5L19 7.5",
  ChevronDown: "M6 9.5l6 6 6-6",
  ChevronLeft: "M14.5 6l-6 6 6 6",
  ChevronRight: "M9.5 6l6 6-6 6",
  Clock: circle(12, 12, 9) + "M12 7.5V12l3 2",
  Devices: DEVICES_MONITOR + "M8.5 15v3.5M6 18.5h5" + DEVICES_PHONE + "M17.5 17.5h1",
  Disc3: circle(12, 12, 9) + circle(12, 12, 2.5) + "M6.5 12A5.5 5.5 0 0 1 12 6.5",
  Eye: EYE + circle(12, 12, 3),
  EyeOff: EYE + "M3.5 3.5l17 17",
  Globe:
    circle(12, 12, 9) + "M3 12h18" +
    "M12 3c-2.5 2.5-3.8 5.6-3.8 9s1.3 6.5 3.8 9c2.5-2.5 3.8-5.6 3.8-9S14.5 5.5 12 3z",
  GripVertical: dot(9, 6) + dot(15, 6) + dot(9, 12) + dot(15, 12) + dot(9, 18) + dot(15, 18),
  Heart:
    "M12 20.25C6.8 17.2 3.5 13.6 3.5 9.6A4.6 4.6 0 0 1 12 7.2a4.6 4.6 0 0 1 8.5 2.4c0 4-3.3 7.6-8.5 10.65z",
  Home:
    "M3 10.5 11.35 3.55a1 1 0 0 1 1.3 0L21 10.5" +
    "M5 8.8v9.7A1.5 1.5 0 0 0 6.5 20h3v-5a1.5 1.5 0 0 1 1.5-1.5h2a1.5 1.5 0 0 1 1.5 1.5v5h3a1.5 1.5 0 0 0 1.5-1.5V8.8",
  Info: circle(12, 12, 9) + "M12 11v5.5M12 7.75h.01",
  Languages:
    "M3 5.5h9.5M7.5 3.5v2M5 8.5l5 5M4 13.5l5-5 1.5-3" +
    "M12.5 20.5l4-9 4 9M14 17.5h5",
  Laptop: rrect(4.5, 5, 15, 10.5, 1.5) + "M2.5 19h19",
  Link:
    "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" +
    "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71",
  Link2: "M9 17H7A5 5 0 0 1 7 7h2M15 7h2a5 5 0 0 1 0 10h-2M8 12h8",
  ListMusic: "M3.5 6h11M3.5 11h11M3.5 16h6.5M20 17.5V4.5" + circle(17, 17.5, 3),
  ListPlus: "M3.5 6h11M3.5 11h11M3.5 16h7M18 13v7M14.5 16.5h7",
  Loader2: "M20.5 12A8.5 8.5 0 1 1 12 3.5",
  LogIn: "M14 4h3.5A2.5 2.5 0 0 1 20 6.5v11a2.5 2.5 0 0 1-2.5 2.5H14M10 8l4 4-4 4M14 12H3.5",
  LogOut: "M10 4H6.5A2.5 2.5 0 0 0 4 6.5v11A2.5 2.5 0 0 0 6.5 20H10M16 8l4 4-4 4M20 12H9.5",
  Maximize2: "M14.5 3.5h6v6M20.5 3.5 14 10M9.5 20.5h-6v-6M3.5 20.5 10 14",
  Minimize2: "M14 4v6h6M14 10l6.5-6.5M10 20v-6H4M10 14l-6.5 6.5",
  Minus: "M5 12h14",
  MonitorSpeaker:
    rrect(2.5, 4.5, 11, 9, 2) + "M8 13.5v3M5.5 17h5" + rrect(16, 3.5, 5.5, 17, 2) +
    circle(18.75, 14, 1.5) + "M18.75 7h.01",
  MoreHorizontal: dot(5, 12) + dot(12, 12) + dot(19, 12),
  Music: "M9 18V6l11-2v12M9 9.5l11-2" + circle(6.5, 18, 2.5) + circle(17.5, 16, 2.5),
  Music2: "M11 17.5V4l6.5 2.8" + circle(8, 17.5, 3),
  PanelLeft: PANEL,
  PanelLeftClose: PANEL + "M16 9.5 13.5 12l2.5 2.5",
  Pause: rrect(6, 4.5, 4, 15, 1.2) + rrect(14, 4.5, 4, 15, 1.2),
  Pin: "M12 16v5M8.5 3.5h7M9.5 3.5v5.2l-3 3.3V14.5h11V12l-3-3.3V3.5",
  PinOff: "M12 16v5M8.5 3.5h7M14.5 3.5v5.2l3 3.3v2.5h-3M9.5 6.5v2.2l-3 3.3v2.5h8M3.5 3.5l17 17",
  Play: "M7 5.4a1.2 1.2 0 0 1 1.8-1.04l10.4 6.6a1.2 1.2 0 0 1 0 2.08l-10.4 6.6A1.2 1.2 0 0 1 7 18.6z",
  Plus: "M12 5v14M5 12h14",
  Queue: QUEUE_PLAY + "M12 7.5h8.5M3.5 13.5h17M3.5 19h17",
  RefreshCw:
    "M20 11A8 8 0 0 0 6.1 6.6L4 8.5M4 4v4.5h4.5M4 13a8 8 0 0 0 13.9 4.4L20 15.5M20 20v-4.5h-4.5",
  Repeat: REPEAT,
  Repeat1: REPEAT + "M11 10.5l1.5-1v5",
  RotateCcw: "M3.5 12a8.5 8.5 0 1 0 2.5-6L3.5 8.5M3.5 3.5v5h5",
  RotateCw: "M20.5 12a8.5 8.5 0 1 1-2.5-6l2.5 2.5M20.5 3.5v5h-5",
  Search: circle(10.75, 10.75, 6.75) + "M15.75 15.75l4.75 4.75",
  Settings: gear(8, 7, 9.4) + circle(12, 12, 3),
  // three nodes, the links stopping at each ring's edge
  Share2:
    circle(18, 5, 3) + circle(6, 12, 3) + circle(18, 19, 3) +
    "M8.59 10.49l6.82-3.98M8.59 13.51l6.82 3.98",
  Shuffle:
    "M16.5 3.5 20 7l-3.5 3.5M16.5 13.5 20 17l-3.5 3.5" +
    "M3.5 7h3.2a4 4 0 0 1 3.3 1.7l4 6.6a4 4 0 0 0 3.3 1.7H20" +
    "M3.5 17h3.2a4 4 0 0 0 3.3-1.7M14 8.7A4 4 0 0 1 17.3 7H20",
  SkipBack: "M19.5 6.2a1.2 1.2 0 0 0-1.85-1l-9 5.8a1.2 1.2 0 0 0 0 2l9 5.8a1.2 1.2 0 0 0 1.85-1zM4.5 5v14",
  SkipForward: "M4.5 6.2a1.2 1.2 0 0 1 1.85-1l9 5.8a1.2 1.2 0 0 1 0 2l-9 5.8a1.2 1.2 0 0 1-1.85-1zM19.5 5v14",
  Smartphone: rrect(6.5, 2.5, 11, 19, 2.5) + "M11 18h2",
  Sparkles:
    "M10 4.5c.4 3.9 2.6 6.1 6.5 6.5-3.9.4-6.1 2.6-6.5 6.5-.4-3.9-2.6-6.1-6.5-6.5 3.9-.4 6.1-2.6 6.5-6.5z" +
    "M18 2.5c.15 1.4.9 2.35 2.5 2.5-1.6.15-2.35.9-2.5 2.5-.15-1.6-.9-2.35-2.5-2.5 1.6-.15 2.35-1.1 2.5-2.5z",
  Speaker: rrect(5, 2.5, 14, 19, 2.5) + circle(12, 14.5, 3.5) + "M12 6.5h.01",
  Trash2:
    "M4 6.5h16M9.5 6.5V5a1.5 1.5 0 0 1 1.5-1.5h2A1.5 1.5 0 0 1 14.5 5v1.5" +
    "M6 6.5l.8 12.1a2 2 0 0 0 2 1.9h6.4a2 2 0 0 0 2-1.9L18 6.5M10 11v5M14 11v5",
  Tv: rrect(3, 5, 18, 12.5, 2.5) + "M8.5 20.5h7",
  User: circle(12, 7.5, 4) + PERSON_BODY,
  UserPlus:
    circle(9.5, 8, 3.5) +
    "M3 19.6c0-3 2.9-5.3 6.5-5.3s6.5 2.3 6.5 5.3a.9.9 0 0 1-.9.9H3.9a.9.9 0 0 1-.9-.9z" +
    "M19 8v6M16 11h6",
  Users:
    circle(9, 8, 3.5) +
    "M2.5 19.6c0-3 2.9-5.3 6.5-5.3s6.5 2.3 6.5 5.3a.9.9 0 0 1-.9.9H3.4a.9.9 0 0 1-.9-.9z" +
    "M15.5 4.8a3.5 3.5 0 0 1 0 6.4M18 14.6c2.1.7 3.5 2.4 3.5 4.6v.4",
  X: "M6.5 6.5l11 11M17.5 6.5l-11 11",
} satisfies Record<string, string>;

export type IconName = keyof typeof iconPaths;

/* filled glyphs. `fill` is painted solid, then `cut` is stroked back out of
   it, then `edge` is stroked on top (defaults to the outline; null = none).
   so a cut can never eat into the border. */
interface Solid {
  fill: string;
  cut?: string;
  edge?: string | null;
}

const TRIANGLE = "M10.27 4.5a2 2 0 0 1 3.46 0l7.5 13a2 2 0 0 1-1.73 3H4.5a2 2 0 0 1-1.73-3z";
const PANEL_SIDE = "M9.5 4H6a3 3 0 0 0-3 3v10a3 3 0 0 0 3 3h3.5z";

const SOLID: Partial<Record<IconName, Solid>> = {
  AlertTriangle: { fill: TRIANGLE, cut: "M12 9.5v4M12 17h.01", edge: TRIANGLE },
  Captions: {
    fill: rrect(3, 5, 18, 14, 3),
    cut: "M7 12h3.5M13.5 12h3.5M7 15.5h6M16 15.5h1",
    edge: rrect(3, 5, 18, 14, 3),
  },
  Clock: { fill: circle(12, 12, 9), cut: "M12 7.5V12l3 2", edge: circle(12, 12, 9) },
  Devices: {
    fill: "M12.5 15H4.5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h11a2 2 0 0 1 2 2v1h-5z" + DEVICES_PHONE,
    cut: "M17.5 17.5h1",
    edge: DEVICES_MONITOR + "M8.5 15v3.5M6 18.5h5" + DEVICES_PHONE,
  },
  Disc3: {
    fill: circle(12, 12, 9) + circle(12, 12, 2.3),
    cut: "M6.5 12A5.5 5.5 0 0 1 12 6.5",
    edge: circle(12, 12, 9),
  },
  Eye: { fill: EYE, cut: circle(12, 12, 3), edge: EYE },
  Globe: {
    fill: circle(12, 12, 9),
    cut: "M3 12h18M12 3c-2.5 2.5-3.8 5.6-3.8 9s1.3 6.5 3.8 9c2.5-2.5 3.8-5.6 3.8-9S14.5 5.5 12 3z",
    edge: circle(12, 12, 9),
  },
  Heart: { fill: iconPaths.Heart },
  Home: {
    fill: "M5 9.1 12 3.3l7 5.8v9.4a1.5 1.5 0 0 1-1.5 1.5h-3v-5a1.5 1.5 0 0 0-1.5-1.5h-2a1.5 1.5 0 0 0-1.5 1.5v5h-3A1.5 1.5 0 0 1 5 18.5z",
  },
  Info: { fill: circle(12, 12, 9), cut: "M12 11v5.5M12 7.75h.01", edge: circle(12, 12, 9) },
  Laptop: { fill: rrect(4.5, 5, 15, 10.5, 1.5) },
  ListMusic: { fill: circle(17, 17.5, 3) },
  MonitorSpeaker: {
    fill: rrect(2.5, 4.5, 11, 9, 2) + rrect(16, 3.5, 5.5, 17, 2),
    cut: circle(18.75, 14, 1.5) + "M18.75 7h.01",
    edge: rrect(2.5, 4.5, 11, 9, 2) + "M8 13.5v3M5.5 17h5" + rrect(16, 3.5, 5.5, 17, 2),
  },
  Music: { fill: "M9 6l11-2v3.5l-11 2z" + circle(6.5, 18, 2.5) + circle(17.5, 16, 2.5) },
  Music2: { fill: circle(8, 17.5, 3) },
  PanelLeft: { fill: PANEL_SIDE },
  PanelLeftClose: { fill: PANEL_SIDE },
  Pause: { fill: iconPaths.Pause },
  Pin: { fill: "M9.5 3.5v5.2l-3 3.3v2.5h11V12l-3-3.3V3.5z" },
  Play: { fill: iconPaths.Play },
  Queue: { fill: QUEUE_PLAY },
  Settings: { fill: gear(8, 7, 9.4) + circle(12, 12, 2.6), edge: gear(8, 7, 9.4) },
  SkipBack: { fill: iconPaths.SkipBack },
  SkipForward: { fill: iconPaths.SkipForward },
  Smartphone: { fill: rrect(6.5, 2.5, 11, 19, 2.5), cut: "M11 18h2", edge: rrect(6.5, 2.5, 11, 19, 2.5) },
  Sparkles: { fill: iconPaths.Sparkles },
  Speaker: {
    fill: rrect(5, 2.5, 14, 19, 2.5),
    cut: circle(12, 14.5, 3.5) + "M12 6.5h.01",
    edge: rrect(5, 2.5, 14, 19, 2.5),
  },
  Tv: { fill: rrect(3, 5, 18, 12.5, 2.5) },
  User: { fill: circle(12, 7.5, 4) + PERSON_BODY },
  UserPlus: {
    fill: circle(9.5, 8, 3.5) + "M3 19.6c0-3 2.9-5.3 6.5-5.3s6.5 2.3 6.5 5.3a.9.9 0 0 1-.9.9H3.9a.9.9 0 0 1-.9-.9z",
  },
  Users: {
    fill: circle(9, 8, 3.5) + "M2.5 19.6c0-3 2.9-5.3 6.5-5.3s6.5 2.3 6.5 5.3a.9.9 0 0 1-.9.9H3.4a.9.9 0 0 1-.9-.9z",
  },
};

const FADE = "opacity 0.15s ease";

function svgProps(size: number | string, strokeWidth: number | string, color: string, style?: React.CSSProperties) {
  return {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    stroke: color,
    strokeWidth,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    style: { display: "inline-block", verticalAlign: "middle", flexShrink: 0, ...style },
  };
}

function createIcon(name: IconName) {
  const d = iconPaths[name];
  const solid = SOLID[name];
  const Comp = React.forwardRef<SVGSVGElement, IconProps>(function Icon(
    { size = 24, strokeWidth = 2, color = "currentColor", fill = "none", active, style, ...rest },
    ref,
  ) {
    const maskId = "ic" + useId().replace(/[^a-zA-Z0-9_-]/g, "");
    const toggles = solid && active !== undefined;
    return (
      <svg ref={ref} {...svgProps(size, strokeWidth, color, style)} fill="none" {...rest}>
        <path d={d} fill={fill} style={toggles ? { transition: FADE, opacity: active ? 0 : 1 } : undefined} />
        {toggles && (
          <>
            <mask id={maskId} maskUnits="userSpaceOnUse" x="0" y="0" width="24" height="24">
              <path d={solid.fill} fill="#fff" fillRule="evenodd" stroke="none" />
              {solid.cut && <path d={solid.cut} stroke="#000" />}
              {solid.edge !== null && <path d={solid.edge ?? d} stroke="#fff" />}
            </mask>
            <rect
              width="24"
              height="24"
              fill={color}
              stroke="none"
              mask={`url(#${maskId})`}
              style={{ transition: FADE, opacity: active ? 1 : 0 }}
            />
          </>
        )}
      </svg>
    );
  });
  Comp.displayName = name;
  return Comp;
}

/* volume: `fill` only fills the speaker horn so the sound waves / mute cross
   stay outlines */
function createVolumeIcon(displayName: string, waves: string) {
  const Comp = React.forwardRef<SVGSVGElement, IconProps>(function VolumeIcon(
    { size = 24, strokeWidth = 2, color = "currentColor", fill = "none", active: _active, style, ...rest },
    ref,
  ) {
    return (
      <svg ref={ref} {...svgProps(size, strokeWidth, color, style)} fill="none" {...rest}>
        <path d={VOLUME_HORN} fill={fill} />
        <path d={waves} />
      </svg>
    );
  });
  Comp.displayName = displayName;
  return Comp;
}

export const AlertTriangle = createIcon("AlertTriangle");
export const ArrowRight = createIcon("ArrowRight");
export const ArrowUpRight = createIcon("ArrowUpRight");
export const Captions = createIcon("Captions");
export const Cast = createIcon("Cast");
export const Check = createIcon("Check");
export const ChevronDown = createIcon("ChevronDown");
export const ChevronLeft = createIcon("ChevronLeft");
export const ChevronRight = createIcon("ChevronRight");
export const Clock = createIcon("Clock");
export const Devices = createIcon("Devices");
export const Disc3 = createIcon("Disc3");
export const Eye = createIcon("Eye");
export const EyeOff = createIcon("EyeOff");
export const Globe = createIcon("Globe");
export const GripVertical = createIcon("GripVertical");
export const Heart = createIcon("Heart");
export const Home = createIcon("Home");
export const Info = createIcon("Info");
export const Languages = createIcon("Languages");
export const Laptop = createIcon("Laptop");
export const Link = createIcon("Link");
export const Link2 = createIcon("Link2");
export const ListMusic = createIcon("ListMusic");
export const ListPlus = createIcon("ListPlus");
export const Loader2 = createIcon("Loader2");
export const LogIn = createIcon("LogIn");
export const LogOut = createIcon("LogOut");
export const Maximize2 = createIcon("Maximize2");
// the counterpart to Maximize2, which is what opens the immersive view
export const Minimize2 = createIcon("Minimize2");
export const Minus = createIcon("Minus");
export const MonitorSpeaker = createIcon("MonitorSpeaker");
export const MoreHorizontal = createIcon("MoreHorizontal");
export const Music = createIcon("Music");
export const Music2 = createIcon("Music2");
export const PanelLeft = createIcon("PanelLeft");
export const PanelLeftClose = createIcon("PanelLeftClose");
export const Pause = createIcon("Pause");
export const Pin = createIcon("Pin");
export const PinOff = createIcon("PinOff");
export const Play = createIcon("Play");
export const Plus = createIcon("Plus");
export const Queue = createIcon("Queue");
export const RefreshCw = createIcon("RefreshCw");
export const Repeat = createIcon("Repeat");
export const Repeat1 = createIcon("Repeat1");
export const RotateCcw = createIcon("RotateCcw");
export const RotateCw = createIcon("RotateCw");
export const Search = createIcon("Search");
export const Settings = createIcon("Settings");
export const Share2 = createIcon("Share2");
export const Shuffle = createIcon("Shuffle");
export const SkipBack = createIcon("SkipBack");
export const SkipForward = createIcon("SkipForward");
export const Smartphone = createIcon("Smartphone");
export const Sparkles = createIcon("Sparkles");
export const Speaker = createIcon("Speaker");
export const Trash2 = createIcon("Trash2");
export const Tv = createIcon("Tv");
export const User = createIcon("User");
export const UserPlus = createIcon("UserPlus");
export const Users = createIcon("Users");
export const X = createIcon("X");

export const Volume1 = createVolumeIcon("Volume1", "M16 9.5a3.5 3.5 0 0 1 0 5");
export const Volume2 = createVolumeIcon("Volume2", "M16 9.5a3.5 3.5 0 0 1 0 5M18.8 6.5a7.5 7.5 0 0 1 0 11");
export const VolumeX = createVolumeIcon("VolumeX", "M16.5 9.5l5 5M21.5 9.5l-5 5");

const ICONS: Record<string, LucideIcon> = {
  AlertTriangle, ArrowRight, ArrowUpRight, Captions, Cast, Check,
  ChevronDown, ChevronLeft, ChevronRight, Clock, Devices, Disc3, Eye, EyeOff,
  Globe, GripVertical, Heart, Home, Info, Languages, Laptop, Link, Link2,
  ListMusic, ListPlus, Loader2, LogIn, LogOut, Maximize2, Minimize2, Minus,
  MonitorSpeaker, MoreHorizontal, Music, Music2, PanelLeft, PanelLeftClose,
  Pause, Pin, PinOff, Play, Plus, Queue, RefreshCw, Repeat, Repeat1, RotateCcw,
  RotateCw, Search, Settings, Share2, Shuffle, SkipBack, SkipForward,
  Smartphone, Sparkles, Speaker, Trash2, Tv, User, UserPlus, Users,
  Volume1, Volume2, VolumeX, X,
};

export function RuneIcon({ name, ...props }: IconProps & { name: string }) {
  const Comp = ICONS[name] || ICONS["Music"];
  return <Comp {...props} />;
}

export { ICONS as iconRegistry };
