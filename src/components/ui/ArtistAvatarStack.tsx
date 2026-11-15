import { useState } from "react";
import { coverUrl } from "../../lib/coverUrl";

const SIZE = 22;
const OVERLAP = 7;
// past this the row just gets wider without telling you anything more
const MAX = 4;

interface Artist {
  id: string;
  name: string;
  /* undefined while the artist is loading, null when it has no portrait */
  image_url: string | null | undefined;
}

/* Overlapping round portraits for a release's artists, first artist on top.
   Every slot is reserved up front (a neutral disc), so portraits that arrive
   at different times fade in where they belong instead of shuffling the row. */
export function ArtistAvatarStack({ artists }: { artists: Artist[] }) {
  const shown = artists.slice(0, MAX);
  if (shown.length === 0) return null;

  return (
    <span
      aria-hidden
      style={{
        display: "inline-flex",
        flexShrink: 0,
        // the stack's own width, so the gap after it matches a single avatar
        width: SIZE + (shown.length - 1) * (SIZE - OVERLAP),
        height: SIZE,
      }}
    >
      {shown.map((a, i) => (
        <Avatar key={a.id} artist={a} index={i} count={shown.length} />
      ))}
    </span>
  );
}

function Avatar({ artist, index, count }: { artist: Artist; index: number; count: number }) {
  const [loaded, setLoaded] = useState(false);
  const src = artist.image_url ? coverUrl(artist.image_url, SIZE * 2) ?? artist.image_url : null;

  return (
    <span
      style={{
        position: "relative",
        width: SIZE,
        height: SIZE,
        flexShrink: 0,
        marginLeft: index === 0 ? 0 : -OVERLAP,
        zIndex: count - index,
        borderRadius: "50%",
        overflow: "hidden",
        background: "rgba(255, 255, 255, 0.12)",
        // the ring cuts each portrait out of the one beneath it
        boxShadow: "0 0 0 1.5px rgba(16, 16, 18, 0.85)",
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: 10,
        fontWeight: 700,
        color: "rgba(255, 255, 255, 0.7)",
      }}
    >
      {/* undefined = still loading (blank disc); null = no portrait (initial) */}
      {artist.image_url === null && artist.name.charAt(0).toUpperCase()}
      {src && (
        <img
          src={src}
          alt=""
          onLoad={() => setLoaded(true)}
          style={{
            position: "absolute",
            inset: 0,
            width: "100%",
            height: "100%",
            objectFit: "cover",
            opacity: loaded ? 1 : 0,
            transition: "opacity 0.25s ease-out",
          }}
        />
      )}
    </span>
  );
}
