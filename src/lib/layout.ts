import type { CSSProperties } from "react";

/* One alignment rule for every tile in the app (album, artist, playlist,
podcast, mix, folder cards).

A tile carries TILE_PAD of padding so its hover fill has room around the
artwork. Grids and carousels are then pulled out by exactly that much on both
sides, so the ARTWORK - not the invisible hover box - sits on the same edge as
the section title above it and the page heading above that. Before this each
container did its own thing and art landed 8, 10, 14 or 16px in from the title
depending on the page. */
export const TILE_PAD = 10;

export const TILE_BLEED: CSSProperties = {
  marginLeft: -TILE_PAD,
  marginRight: -TILE_PAD,
  width: `calc(100% + ${TILE_PAD * 2}px)`,
};

// the one tile grid: same column size and gutter everywhere
export const TILE_GRID: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fill, minmax(clamp(120px, 14vw, 175px), 1fr))",
  gap: "clamp(10px, 1.4vw, 16px)",
  ...TILE_BLEED,
};
