import { TILE_GRID, TILE_PAD } from "../../lib/layout";
/* loading placeholders shaped like the content they stand in for, so the page
keeps its layout while data arrives instead of flashing a "Loading…" line.
Static (no shimmer): a shimmer is one more moving thing to ignore, and it would
have to be switched off for reduced motion anyway. */

const BLOCK = "var(--color-surface)";

// a list of track rows, matching TrackRow's height and column rhythm
export function TrackRowsSkeleton({ count = 8, showCover = true }: { count?: number; showCover?: boolean }) {
  return (
    <div aria-hidden style={{ display: "flex", flexDirection: "column" }}>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 14,
            height: 54,
            padding: "0 12px",
            // later rows fade out, so the list reads as "more coming" not a wall
            opacity: Math.max(0.25, 1 - i * 0.09),
          }}
        >
          <div style={{ width: 16, height: 10, borderRadius: 3, background: BLOCK, flexShrink: 0 }} />
          {showCover && <div style={{ width: 38, height: 38, borderRadius: 6, background: BLOCK, flexShrink: 0 }} />}
          <div style={{ display: "flex", flexDirection: "column", gap: 7, flex: 1, minWidth: 0 }}>
            <div style={{ width: `${46 + ((i * 17) % 30)}%`, height: 11, borderRadius: 4, background: BLOCK }} />
            <div style={{ width: `${24 + ((i * 11) % 20)}%`, height: 9, borderRadius: 4, background: BLOCK, opacity: 0.7 }} />
          </div>
          <div style={{ width: 30, height: 9, borderRadius: 4, background: BLOCK, flexShrink: 0 }} />
        </div>
      ))}
    </div>
  );
}

// a card grid matching AlbumGrid / ArtistGrid / the playlist grid
export function CardGridSkeleton({
  count = 12,
  round = false,
  minCol = "clamp(120px, 14vw, 175px)",
}: {
  count?: number;
  // artists are circles
  round?: boolean;
  minCol?: string;
}) {
  return (
    <div
      aria-hidden
      style={{ ...TILE_GRID, gridTemplateColumns: `repeat(auto-fill, minmax(${minCol}, 1fr))` }}
    >
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} style={{ display: "flex", flexDirection: "column", gap: 10, padding: TILE_PAD }}>
          <div style={{ width: "100%", aspectRatio: "1 / 1", borderRadius: round ? "50%" : 8, background: BLOCK }} />
          <div style={{ width: "70%", height: 11, borderRadius: 4, background: BLOCK, alignSelf: round ? "center" : undefined }} />
          <div style={{ width: "45%", height: 9, borderRadius: 4, background: BLOCK, opacity: 0.7, alignSelf: round ? "center" : undefined }} />
        </div>
      ))}
    </div>
  );
}

// the Search "top result + songs" row
export function TopResultSkeleton() {
  return (
    <div
      aria-hidden
      style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 320px), 1fr))",
        gap: "clamp(16px, 2.2vw, 28px)",
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div style={{ width: 110, height: 18, borderRadius: 5, background: BLOCK }} />
        <div style={{ height: 230, borderRadius: 14, background: BLOCK }} />
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div style={{ width: 70, height: 18, borderRadius: 5, background: BLOCK }} />
        <TrackRowsSkeleton count={4} />
      </div>
    </div>
  );
}
