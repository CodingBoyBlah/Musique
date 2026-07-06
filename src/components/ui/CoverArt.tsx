import { Music } from "lucide-react";
import type { CSSProperties } from "react";

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

export function CoverArt({ url, alt, size, rounded, className = "", style, loading, decoding }: Props) {
  const radius  = rounded ? "50%" : 6;
  const baseStyle = { width: size, height: size, borderRadius: radius, flexShrink: 0 as const };

  if (url) {
    return (
      <img
        src={url}
        alt={alt}
        referrerPolicy="no-referrer"
        className={className}
        style={{ ...baseStyle, objectFit: "cover" as const, ...style }}
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
        background:     "rgba(255,255,255,0.06)",
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
