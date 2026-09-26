import { Music } from "@/lib/icons";
import { coverUrl } from "../../lib/coverUrl";
import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";

interface Props {
  url:       string | null | undefined;
  alt:       string;
  size:      number;
  rounded?:  boolean;
  className?: string;
  // overrides merged last - eg width/height 100% to fill a flexible box
  style?:    CSSProperties;
  loading?: "lazy" | "eager";
  decoding?: "async" | "sync" | "auto";
}

// the tile a cover sits on until its pixels arrive, and what shows if they never do
const PLACEHOLDER_BG = "rgba(255,255,255,0.06)";

export function CoverArt({
  url,
  alt,
  size,
  rounded,
  className = "",
  style,
  loading = "lazy",
  decoding = "async",
}: Props) {
  const radius  = rounded ? "50%" : 6;
  const baseStyle = { width: size, height: size, borderRadius: radius, flexShrink: 0 as const };

  if (url) {
    return (
      <FadeImg
        src={coverUrl(url, size) ?? url}
        alt={alt}
        className={className}
        boxStyle={{ ...baseStyle, ...style }}
        loading={loading}
        decoding={decoding}
      />
    );
  }

  return (
    <div
      className={className}
      style={{
        ...baseStyle,
        ...style,
        background:     PLACEHOLDER_BG,
        border:         "1px solid rgba(255,255,255,0.06)",
        display:        "flex",
        alignItems:     "center",
        justifyContent: "center",
        color:          "rgba(255,255,255,0.25)",
      }}
    >
      {rounded
        ? <span style={{ fontSize: size * 0.4, fontWeight: 700, color: "rgba(255,255,255,0.5)" }}>
            {alt.charAt(0).toUpperCase()}
          </span>
        : <Music size={size * 0.35} strokeWidth={1.5} />
      }
    </div>
  );
}

/* A cover that arrives rather than pops.
 *
 * Grids load dozens of these at once, and each one snapping in on its own
 * frame reads as flicker. The image fades up over a quiet tile instead - but
 * only when it actually had to wait: a cover already in the memory cache is
 * `complete` by the time we can look at it, and fading that in on every
 * re-render or back-navigation would be motion for nothing.
 *
 * The caller's style lands on the outer box, exactly where it used to land on
 * the <img>, so every existing layout (100% fills, borders, radii, display)
 * keeps working; only the fit and position are forwarded to the image. */
function FadeImg({
  src,
  alt,
  className,
  boxStyle,
  loading,
  decoding,
}: {
  src: string;
  alt: string;
  className: string;
  boxStyle: CSSProperties;
  loading: "lazy" | "eager";
  decoding: "async" | "sync" | "auto";
}) {
  const ref = useRef<HTMLImageElement>(null);
  // "cached": already decoded on first look - show it with no transition at all
  const [state, setState] = useState<"waiting" | "loaded" | "cached">("waiting");

  useLayoutEffect(() => {
    const img = ref.current;
    if (img && img.complete && img.naturalWidth > 0) setState("cached");
    else setState("waiting");
  }, [src]);

  const { objectFit, objectPosition, ...outer } = boxStyle;

  return (
    <span
      className={className}
      style={{
        display: "inline-block",
        position: "relative",
        overflow: "hidden",
        background: PLACEHOLDER_BG,
        ...outer,
      }}
    >
      <img
        ref={ref}
        src={src}
        alt={alt}
        referrerPolicy="no-referrer"
        loading={loading}
        decoding={decoding}
        onLoad={() => setState((s) => (s === "cached" ? s : "loaded"))}
        style={{
          display: "block",
          width: "100%",
          height: "100%",
          borderRadius: "inherit",
          objectFit: objectFit ?? "cover",
          objectPosition,
          opacity: state === "waiting" ? 0 : 1,
          transition: state === "loaded" ? "opacity 150ms cubic-bezier(0.23, 1, 0.32, 1)" : "none",
        }}
      />
    </span>
  );
}
