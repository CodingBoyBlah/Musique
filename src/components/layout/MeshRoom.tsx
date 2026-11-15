import { useEffect, useRef } from "react";
import type { MeshColor } from "../../lib/ambient";

/* The room as a moving mesh gradient.
 *
 * One soft pool of light per colour of the cover's palette (lib/ambient),
 * each on its own looping path, over a floor of the main colour. Drawn every
 * frame into a 128x72 canvas and stretched to fill the window: at that size a
 * frame is a handful of radial gradients and one small blur, next to nothing,
 * and the bilinear upscale turns it into a seamless wash with no edges and no
 * structure from the artwork to make out.
 *
 * Paths are Lissajous loops on periods that never line up, so the field never
 * repeats a frame and never stops - pools cross, merge and pull apart. Pausing
 * (paused music, an unfocused window, Settings > Animated background, reduced
 * motion) freezes the clock where it is, so it picks up from there.
 *
 * A new palette is eased into, not cut to: each pool's colour and size glide
 * to the new record's over about a second while the motion carries on, which
 * is the track-change crossfade without a second canvas. */

const W = 128;
const H = 72;
const POOLS = 5;

// where each pool loops around, and how: anchor, reach, and its two periods (s)
const PATHS = [
  { x: 0.28, y: 0.32, ax: 0.3, ay: 0.3, px: 9.3, py: 10.9, ph: 0.0 },
  { x: 0.74, y: 0.28, ax: 0.28, ay: 0.32, px: 11.7, py: 8.7, ph: 1.7 },
  { x: 0.7, y: 0.74, ax: 0.32, ay: 0.28, px: 8.1, py: 12.1, ph: 3.1 },
  { x: 0.26, y: 0.76, ax: 0.3, ay: 0.3, px: 13.3, py: 9.6, ph: 4.4 },
  { x: 0.5, y: 0.5, ax: 0.36, ay: 0.34, px: 10.4, py: 7.9, ph: 5.6 },
];

/* How fast the field runs against the periods above. 1.3x puts each loop at
   roughly 6-10s: unhurried, but always visibly on the move. One knob for the
   whole tempo: the breathing rides the same clock. */
const SPEED = 1.3;

// how fast a new palette is taken up: time constant, seconds
const EASE_TAU = 0.35;

interface Pool { r: number; g: number; b: number; share: number }

function poolsOf(mesh: MeshColor[]): Pool[] {
  // a short palette repeats around, so there are always five pools moving
  return Array.from({ length: POOLS }, (_, i) => {
    const m = mesh[i % mesh.length];
    return { r: m.rgb[0], g: m.rgb[1], b: m.rgb[2], share: i < mesh.length ? m.share : m.share * 0.7 };
  });
}

export function MeshRoom({ mesh, ready, moving }: { mesh: MeshColor[]; ready: boolean; moving: boolean }) {
  const ref = useRef<HTMLCanvasElement>(null);
  const target = useRef<Pool[]>(poolsOf(mesh));
  const current = useRef<Pool[] | null>(null);
  const clock = useRef(Math.random() * 60); // a fresh view doesn't open on the same frame
  const movingRef = useRef(moving);
  const kick = useRef<() => void>(() => {});

  movingRef.current = moving;

  useEffect(() => {
    target.current = poolsOf(mesh);
    if (ready && !current.current) current.current = target.current.map((p) => ({ ...p }));
    kick.current();
  }, [mesh, ready]);

  useEffect(() => { kick.current(); }, [moving]);

  useEffect(() => {
    const cv = ref.current;
    const out = cv?.getContext("2d");
    if (!cv || !out) return;
    const scratch = document.createElement("canvas");
    scratch.width = W;
    scratch.height = H;
    const ctx = scratch.getContext("2d");
    if (!ctx) return;

    let raf = 0;
    let last = 0;
    const span = Math.hypot(W, H);
    const rgba = (p: Pool, a: number) => `rgba(${p.r | 0}, ${p.g | 0}, ${p.b | 0}, ${a})`;

    const draw = (pools: Pool[], t: number) => {
      ctx.fillStyle = rgba(pools[0], 1);
      ctx.fillRect(0, 0, W, H);
      pools.forEach((p, i) => {
        const path = PATHS[i];
        const cx = (path.x + path.ax * Math.sin((t * 2 * Math.PI) / path.px + path.ph)) * W;
        const cy = (path.y + path.ay * Math.cos((t * 2 * Math.PI) / path.py + path.ph * 1.3)) * H;
        // pools breathe a little as they travel
        const breathe = 1 + 0.14 * Math.sin((t * 2 * Math.PI) / (path.px * 0.7) + path.ph);
        /* size and strength follow the colour's share of the sleeve, so a
           black cover with a thin rainbow on it stays a dark room with glows
           moving through it rather than turning into the rainbow */
        const radius = (0.2 + 0.5 * Math.sqrt(p.share)) * span * breathe;
        const peak = 0.6 + 0.35 * Math.min(1, p.share / 0.2);
        const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
        // eased falloff: a two-stop radial reads as a disc with a rim
        g.addColorStop(0, rgba(p, peak));
        g.addColorStop(0.35, rgba(p, peak * 0.82));
        g.addColorStop(0.68, rgba(p, peak * 0.32));
        g.addColorStop(1, rgba(p, 0));
        ctx.fillStyle = g;
        ctx.fillRect(0, 0, W, H);
      });
      // one small blur melts the pools into each other; drawn oversize so the
      // blur has colour to pull from at the edges instead of a dark rim
      out.filter = "blur(4px)";
      out.drawImage(scratch, -6, -6, W + 12, H + 12);
      out.filter = "none";
    };

    const frame = (now: number) => {
      raf = 0;
      const dt = last ? Math.min(0.1, (now - last) / 1000) : 0;
      last = now;
      const cur = current.current;
      if (!cur) return;
      if (movingRef.current) clock.current += dt * SPEED;

      // glide toward the current palette
      const k = 1 - Math.exp(-dt / EASE_TAU);
      let settling = false;
      cur.forEach((p, i) => {
        const q = target.current[i];
        p.r += (q.r - p.r) * k; p.g += (q.g - p.g) * k; p.b += (q.b - p.b) * k;
        p.share += (q.share - p.share) * k;
        if (Math.abs(q.r - p.r) + Math.abs(q.g - p.g) + Math.abs(q.b - p.b) > 1.5 || Math.abs(q.share - p.share) > 0.004) settling = true;
      });

      draw(cur, clock.current);
      if (movingRef.current || settling) raf = requestAnimationFrame(frame);
      else last = 0;
    };

    kick.current = () => {
      if (!raf) raf = requestAnimationFrame(frame);
    };
    kick.current();
    return () => {
      cancelAnimationFrame(raf);
      kick.current = () => {};
    };
  }, []);

  return (
    <canvas
      ref={ref}
      width={W}
      height={H}
      aria-hidden
      style={{
        position: "absolute", inset: 0, width: "100%", height: "100%",
        opacity: ready ? 1 : 0,
        transition: "opacity 0.6s cubic-bezier(0.4, 0, 0.2, 1)",
      }}
    />
  );
}
