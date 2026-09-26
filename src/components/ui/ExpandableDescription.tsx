import { useState, useMemo } from "react";
import { cleanDescription } from "../../utils/fmt";

interface Props {
  text: string | null | undefined;
  maxLength?: number;
  clampLines?: number;
  className?: string;
  style?: React.CSSProperties;
}

export function ExpandableDescription({
  text,
  maxLength = 130,
  clampLines = 2,
  className = "",
  style,
}: Props) {
  const [expanded, setExpanded] = useState(false);
  const cleaned = useMemo(() => cleanDescription(text), [text]);

  if (!cleaned) return null;

  const isLong = cleaned.length > maxLength;

  return (
    <div className={className} style={{ maxWidth: "min(680px, 100%)", ...style }}>
      <p
        style={{
          margin: 0,
          fontSize: 13.5,
          lineHeight: 1.5,
          color: "var(--color-text-dim)",
          whiteSpace: expanded ? "pre-line" : "normal",
          wordBreak: "break-word",
          ...(!expanded && isLong
            ? {
                display: "-webkit-box",
                WebkitLineClamp: clampLines,
                WebkitBoxOrient: "vertical",
                overflow: "hidden",
                textOverflow: "ellipsis",
              }
            : {}),
        }}
      >
        {cleaned}
      </p>
      {isLong && (
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            setExpanded(!expanded);
          }}
          className="btn-text t-caption"
          style={{
            padding: "3px 0 0",
            fontSize: 12,
            fontWeight: 600,
            color: "var(--color-text)",
            display: "inline-flex",
            alignItems: "center",
            gap: 4,
          }}
          aria-expanded={expanded}
        >
          {expanded ? "Show less" : "Show more"}
        </button>
      )}
    </div>
  );
}
