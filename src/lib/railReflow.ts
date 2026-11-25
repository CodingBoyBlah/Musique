/* The page through a right-rail slide, without reflowing it every frame.

A slide changes the page's width in one frame (Layout, "The right rail
slides"): at the start of a close it widens under the panel, at the end of an
open it narrows into the room that is left. Laid out in one step, everything
the width moves would jump: a track's duration from one edge to the other, a
shelf's arrows, an album cover sized off the header's width - and with the
cover, every row below it.

So each slide measures the visible page three times: where things are on
screen now (`before`), where they sit in the layout the slide runs in (`wide`
- the rail's room handed back to the card), and where they will be once it
lands (`landing`). Each element is then played from its `before` to its
`landing` place as a transform on top of the `wide` layout, on the compositor,
on the same curve and clock as the card's edge - which reads exactly like the
page reflowing with the edge, at no per-frame cost.

Per element, top-down:
- the same size in all three layouts: a move (the duration column, the arrows,
  the rows under a header that shrank) - translate it, and its subtree rides
  along;
- resized in proportion (the cover): scale it from its top-left corner;
- resized any other way (a row, a heading, a shelf - wider or narrower): its
  changing side is hidden behind the card's travelling edge anyway, so it is
  left to snap, and its children are looked at instead, relative to it.
Tile grids are FLIPped by lib/railFlip; here they only follow their own
position (a header above them changing height). */

interface Box { x: number; y: number; w: number; h: number }

interface Node {
  el: Element;
  parent: number;
  grid: boolean;
  before: Box;
  wide?: Box;
  landing?: Box;
}

const MAX_NODES = 2500;
const MAX_ANIMS = 240;

function box(el: Element): Box {
  const r = el.getBoundingClientRect();
  return { x: r.left, y: r.top, w: r.width, h: r.height };
}

export interface PageSnapshot {
  nodes: Node[];
  view: Box;
}

/** Where the visible page is on screen right now (running animations
 * included). Subtrees entirely off screen are not walked. */
export function snapshotPage(root: HTMLElement): PageSnapshot {
  const v = root.getBoundingClientRect();
  // the strip a closing panel uncovers is part of what will be seen
  const view = { x: v.left, y: v.top - 120, w: v.width + 400, h: v.height + 240 };
  const nodes: Node[] = [];
  const stack: [Element, number][] = [];
  for (let i = root.children.length - 1; i >= 0; i--) stack.push([root.children[i], -1]);
  while (stack.length && nodes.length < MAX_NODES) {
    const [el, parent] = stack.pop()!;
    const b = box(el);
    const grid = el.hasAttribute("data-rail-lock");
    const idx = nodes.push({ el, parent, grid, before: b }) - 1;
    if (grid || el instanceof SVGElement) continue;
    if (b.w === 0 && b.h === 0) {
      // no box: hidden (Home kept alive under display: none - thousands of
      // elements) unless it is display: contents, whose children have boxes
      if (getComputedStyle(el).display !== "contents") continue;
    } else if (!overlaps(b, view)) continue;
    for (let i = el.children.length - 1; i >= 0; i--) stack.push([el.children[i], idx]);
  }
  return { nodes, view };
}

/** Record where every snapshotted element sits in the current layout. */
export function measurePage(snap: PageSnapshot, as: "wide" | "landing"): void {
  for (const n of snap.nodes) n[as] = box(n.el);
}

function overlaps(a: Box, b: Box): boolean {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;
}

const near = (a: number, b: number) => Math.abs(a - b) < 0.5;

/** Play every element from its `before` place to its `landing` place over the
 * `wide` layout. Call after both measurePage passes, with the page back in its
 * wide layout. */
export function playPage(snap: PageSnapshot, opts: KeyframeAnimationOptions): Animation[] {
  const { nodes, view } = snap;
  const anims: Animation[] = [];
  // translation already applied by an ancestor, at the start and at the end
  const acc0x = new Float64Array(nodes.length);
  const acc0y = new Float64Array(nodes.length);
  const acc1x = new Float64Array(nodes.length);
  const acc1y = new Float64Array(nodes.length);
  const done = new Uint8Array(nodes.length);

  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i];
    const p = n.parent;
    if (p >= 0 && done[p]) {
      done[i] = 1;
      continue;
    }
    const p0x = p >= 0 ? acc0x[p] : 0;
    const p0y = p >= 0 ? acc0y[p] : 0;
    const p1x = p >= 0 ? acc1x[p] : 0;
    const p1y = p >= 0 ? acc1y[p] : 0;
    acc0x[i] = p0x;
    acc0y[i] = p0y;
    acc1x[i] = p1x;
    acc1y[i] = p1y;
    const { before: b, wide: w, landing: l } = n;
    if (!w || !l) {
      done[i] = 1;
      continue;
    }
    if (w.w === 0 && w.h === 0) continue; // nothing of its own to move; look inside
    if (!overlaps(b, view) && !overlaps(w, view) && !overlaps(l, view)) {
      done[i] = 1;
      continue;
    }

    // residual move, after what the ancestors already do
    let r0x = b.x - w.x - p0x;
    let r0y = b.y - w.y - p0y;
    const r1x = l.x - w.x - p1x;
    const r1y = l.y - w.y - p1y;

    if (n.grid) {
      // its cards are FLIPped from where they were on screen by railFlip, so
      // the grid itself starts still and only follows its own landing place
      r0x = -p0x;
      r0y = -p0y;
      if (anims.length < MAX_ANIMS) push(anims, n.el, r0x, r0y, 1, r1x, r1y, 1, opts);
      done[i] = 1;
      continue;
    }

    const moved = !near(r0x, 0) || !near(r0y, 0) || !near(r1x, 0) || !near(r1y, 0);
    const sameSize = near(b.w, w.w) && near(b.h, w.h) && near(l.w, w.w) && near(l.h, w.h);
    if (sameSize) {
      if (moved && anims.length < MAX_ANIMS) push(anims, n.el, r0x, r0y, 1, r1x, r1y, 1, opts);
      done[i] = 1;
      continue;
    }

    const s0 = w.w > 0 ? b.w / w.w : 1;
    const s1 = w.w > 0 ? l.w / w.w : 1;
    const proportional = w.h > 0 && near(b.h, w.h * s0) && near(l.h, w.h * s1) && w.w > 24 && w.h > 24;
    if (proportional && anims.length < MAX_ANIMS) {
      push(anims, n.el, r0x, r0y, s0, r1x, r1y, s1, opts);
      done[i] = 1;
      continue;
    }

    // resized another way: keep its top-left on course and look inside
    if (moved && anims.length < MAX_ANIMS) {
      push(anims, n.el, r0x, r0y, 1, r1x, r1y, 1, opts);
      acc0x[i] = p0x + r0x;
      acc0y[i] = p0y + r0y;
      acc1x[i] = p1x + r1x;
      acc1y[i] = p1y + r1y;
    }
  }
  return anims;
}

function push(
  anims: Animation[], el: Element,
  x0: number, y0: number, s0: number, x1: number, y1: number, s1: number,
  opts: KeyframeAnimationOptions,
) {
  // an element's own transform (a centring translate, framer's inline
  // transform) stays under the move rather than being replaced by it
  const base = getComputedStyle(el).transform;
  const own = base && base !== "none" ? ` ${base}` : "";
  const scaled = s0 !== 1 || s1 !== 1;
  if (scaled && own) {
    // scaling around a transform of its own: move it, don't scale it
    s0 = s1 = 1;
  }
  const frame = (x: number, y: number, s: number): Keyframe =>
    scaled && (s0 !== 1 || s1 !== 1)
      ? { transform: cornerScale(el, x, y, s) }
      : { transform: `translate(${x}px, ${y}px)${own}` };
  anims.push(el.animate([frame(x0, y0, s0), frame(x1, y1, s1)], opts));
}

/** A move by (x, y) plus a scale about the element's top-left corner, written
 * for the element's own transform-origin. Animating transform-origin itself
 * would take the animation off the compositor. */
export function cornerScale(el: Element, x: number, y: number, s: number): string {
  const [ox, oy] = getComputedStyle(el).transformOrigin.split(" ").map(parseFloat);
  const k = 1 - s;
  return `translate(${x - (ox || 0) * k}px, ${y - (oy || 0) * k}px) scale(${s})`;
}
