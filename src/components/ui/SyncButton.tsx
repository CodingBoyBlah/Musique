import { useEffect, useState } from "react";
import { useIsMutating } from "@tanstack/react-query";
import { RefreshCw } from "@/lib/icons";
import { useSyncLibrary, useLibraryStatus, LIBRARY_SYNC_KEY } from "../../hooks/useLibrary";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";

/* "Synced 2 min ago" rather than a wall-clock time with seconds. re-renders once
a minute so it stays true while the page is open. */
// at: epoch milliseconds
function useRelativeTime(at: number | null | undefined): string | null {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!at) return;
    setNow(Date.now());
    const t = window.setInterval(() => setNow(Date.now()), 60_000);
    return () => window.clearInterval(t);
  }, [at]);
  if (!at || !Number.isFinite(at)) return null;
  const mins = Math.round((now - at) / 60_000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins} min ago`;
  const hrs = Math.round(mins / 60);
  if (hrs < 24) return `${hrs} hr ago`;
  const days = Math.round(hrs / 24);
  return days === 1 ? "yesterday" : `${days} days ago`;
}

/* the Sync control. shared by the page header and every empty state, so an
empty tab offers the fix right where it says what's missing. */
export function SyncButton({ showStatus = true, prominent = false }: { showStatus?: boolean; prominent?: boolean }) {
  const { mutate } = useSyncLibrary();
  // busy while ANY library sync runs (e.g. Playlists' first-visit auto sync),
  // not just one started from this button
  const isPending = useIsMutating({ mutationKey: LIBRARY_SYNC_KEY }) > 0;
  const { data: status } = useLibraryStatus();
  const synced = useRelativeTime(status?.last_synced);

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
      {showStatus && synced && (
        <span className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
          Synced {synced}
        </span>
      )}
      <button
        type="button"
        className={prominent ? "btn-primary" : "btn-pill"}
        onClick={() => {
          if (isPending) return;
          mutate(undefined, { onError: (e) => toast.error(`Sync failed: ${errMsg(e)}`) });
        }}
        aria-disabled={isPending || undefined}
        aria-busy={isPending || undefined}
        style={prominent ? { height: 36, padding: "0 20px" } : { height: 30, fontSize: 12, fontWeight: 500 }}
      >
        <RefreshCw size={12} strokeWidth={2} style={{ animation: isPending ? "spin 1s linear infinite" : "none" }} />
        {isPending ? "Syncing…" : prominent ? "Sync library" : "Sync"}
      </button>
    </div>
  );
}
