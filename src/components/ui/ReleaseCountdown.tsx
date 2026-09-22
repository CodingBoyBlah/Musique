import { useEffect, useState } from "react";
import { Clock } from "@/lib/icons";
import { msUntilRelease, fmtCountdown } from "../../utils/fmt";

{/* live countdown, erros in fetching from spotify, fix TODO*/}
export function ReleaseCountdown({ date }: { date: string }) {
  const [remaining, setRemaining] = useState(() => msUntilRelease(date));

  useEffect(() => {
    setRemaining(msUntilRelease(date));
    const id = setInterval(() => setRemaining(msUntilRelease(date)), 1000);
    return () => clearInterval(id);
  }, [date]);

  if (remaining <= 0) return null;

  return (
    <span
      className="text-xs"
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 4,
        height: 15,
        lineHeight: "15px",
        alignSelf: "flex-start",
        width: "100%",
        minWidth: 0,
        maxWidth: "100%",
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
        color: "var(--color-accent)",
        fontWeight: 600,
        fontVariantNumeric: "tabular-nums",
      }}
    >
      <Clock size={11} strokeWidth={2.4} style={{ flexShrink: 0 }} />
      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {fmtCountdown(remaining)}
      </span>
    </span>
  );
}
