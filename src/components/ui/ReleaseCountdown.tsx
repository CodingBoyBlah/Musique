import { useEffect, useState } from "react";
import { Clock } from "lucide-react";
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
        color: "var(--color-accent)",
        fontWeight: 600,
        fontVariantNumeric: "tabular-nums",
      }}
    >
      <Clock size={11} strokeWidth={2.4} />
      {fmtCountdown(remaining)}
    </span>
  );
}
