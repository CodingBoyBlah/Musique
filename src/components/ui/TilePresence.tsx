import { Children, isValidElement, useEffect, useRef, type ReactElement, type ReactNode } from "react";
import { motion, AnimatePresence, type Transition, type Variants } from "framer-motion";
import { zTransform, EASE_OUT } from "../../lib/motion";

/* Tiles that arrive or leave when a grid's items change: a range switch, a
search, a filter, a refresh. The cards that stay already glide to their new
cells on their own layout spring. Without this the newcomers just appeared in
the gaps and the leavers vanished.

Newcomers rise out of a slight scale-down, staggered among themselves. Leavers
shrink out faster and are popped out of flow at once (popLayout), so the cards
that stay start moving into their cells straight away instead of waiting for
the exit. Opacity carries it under reduced motion (App's MotionConfig drops
the scale).

Kept cheap for long grids (a Library page can hold hundreds of tiles):
- the grid's first paint is instant (initial={false}); only later changes animate
- a pure tail append (useStagedList filling in, a wider window fitting another
  column) is a reveal, not a change, and doesn't animate
- only the first ENTER_MAX newcomers animate, in DOM order, which are the ones
  near the top; the rest appear in place
- a mass removal (clearing past EXIT_MAX tiles at once) leaves instantly

Children must be keyed elements. The parent grid needs a non-static position,
since a leaving tile is positioned against it. */

const ENTER_S = 0.22;
const EXIT_S = 0.15;
const STAGGER_S = 0.04;
const STAGGER_MAX = 6;
const ENTER_MAX = 24;
const EXIT_MAX = 48;

type Key = string | number;

const variants: Variants = {
  out: { opacity: 0, scale: 0.95 },
  // custom: this tile's place among the newcomers, or -1 to arrive without fuss
  in: (order: number) => ({
    opacity: 1,
    scale: 1,
    transition:
      order < 0
        ? { duration: 0 }
        : { duration: ENTER_S, ease: EASE_OUT, delay: Math.min(order, STAGGER_MAX) * STAGGER_S },
  }),
  // custom (from AnimatePresence, read at exit time): true on a mass removal
  gone: (bulk: boolean) => ({
    opacity: 0,
    scale: 0.95,
    transition: bulk ? { duration: 0 } : { duration: EXIT_S, ease: EASE_OUT },
  }),
};

const TILE_STYLE = { minWidth: 0, width: "100%" } as const;

export function TilePresence({
  children,
  layoutTransition,
}: {
  children: ReactNode;
  /** give each tile's wrapper its own layout animation (for grids whose cards
   * don't move themselves); per index, so it can carry a stagger */
  layoutTransition?: (index: number) => { layout: Transition };
}) {
  const kids = Children.toArray(children).filter(isValidElement) as ReactElement[];
  const keys: Key[] = kids.map((k, i) => k.key ?? i);

  const shownRef = useRef<Key[]>([]);
  const shown = shownRef.current;
  const shownSet = new Set(shown);
  const keySet = new Set(keys);

  const order = new Map<Key, number>();
  const isAppend = shown.length > 0 && shown.length < keys.length && shown.every((k, i) => keys[i] === k);
  if (!isAppend) {
    for (const k of keys) {
      if (shownSet.has(k)) continue;
      if (order.size >= ENTER_MAX) break;
      order.set(k, order.size);
    }
  }
  let leaving = 0;
  for (const k of shown) if (!keySet.has(k)) leaving++;
  const bulkExit = leaving > EXIT_MAX;

  useEffect(() => {
    shownRef.current = keys;
  });

  return (
    <AnimatePresence mode="popLayout" initial={false} custom={bulkExit}>
      {kids.map((child, index) => {
        const key = keys[index];
        return (
          <motion.div
            key={key}
            layout={layoutTransition ? "position" : undefined}
            transformTemplate={zTransform}
            transition={layoutTransition?.(index)}
            variants={variants}
            custom={order.get(key) ?? -1}
            initial={order.has(key) ? "out" : false}
            animate="in"
            exit="gone"
            style={TILE_STYLE}
          >
            {child}
          </motion.div>
        );
      })}
    </AnimatePresence>
  );
}
