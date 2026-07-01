import { useMemo, useState, type ReactNode } from "react";
import { Search, X } from "lucide-react";
import type { TrackItem } from "../../types/spotify";
import { Dropdown, type DropdownOption } from "./Dropdown";

type OrderKey = "default" | "name" | "artist" | "album" | "duration";
type Dir = "asc" | "desc";

const firstArtist = (t: TrackItem) => t.artists[0]?.name ?? "";
const albumName   = (t: TrackItem) => t.album?.name ?? "";


export function useTrackTools(tracks: TrackItem[], defaultOrderLabel: string): {
  view: TrackItem[];
  /* stable per-row keys aligned 1:1 with `view` so rows keep their identity
  across re-sorts (lets the list FLIP-animate instead of snapping) */
  keys: string[];
  toolbar: ReactNode;
} {
  const [query, setQuery] = useState("");
  const [order, setOrder] = useState<OrderKey>("default");
  const [dir,   setDir]   = useState<Dir>("asc");

  /* identity that survives sorting/filtering: each original track object gets a
  fixed key (id + its original position, so dupe tracks stay distinct) */
  const keyOf = useMemo(() => {
    const m = new Map<TrackItem, string>();
    tracks.forEach((t, i) => m.set(t, `${t.id}#${i}`));
    return m;
  }, [tracks]);

  const view = useMemo(() => {
    let arr = tracks;

    const q = query.trim().toLowerCase();
    if (q) {
      arr = arr.filter(
        (t) =>
          t.name.toLowerCase().includes(q) ||
          t.artists.some((a) => a.name.toLowerCase().includes(q)),
      );
    }

    if (order !== "default") {
      const cmp: Record<Exclude<OrderKey, "default">, (a: TrackItem, b: TrackItem) => number> = {
        name:     (a, b) => a.name.localeCompare(b.name),
        artist:   (a, b) => firstArtist(a).localeCompare(firstArtist(b)),
        album:    (a, b) => albumName(a).localeCompare(albumName(b)),
        duration: (a, b) => a.duration_ms - b.duration_ms,
      };
      arr = [...arr].sort(cmp[order]);
    } else if (dir === "desc") {
      // default order has no comparator, so descending just reverses it
      arr = [...arr].reverse();
    }

    if (order !== "default" && dir === "desc") arr = [...arr].reverse();
    return arr;
  }, [tracks, query, order, dir]);

  const keys = useMemo(() => view.map((t) => keyOf.get(t) ?? t.id), [view, keyOf]);

  const orderOptions: DropdownOption<OrderKey>[] = [
    { value: "default",  label: defaultOrderLabel },
    { value: "name",     label: "Name" },
    { value: "artist",   label: "Artist" },
    { value: "album",    label: "Album" },
    { value: "duration", label: "Duration" },
  ];
  const dirOptions: DropdownOption<Dir>[] = [
    { value: "asc",  label: "Ascending" },
    { value: "desc", label: "Descending" },
  ];

  const toolbar = (
    <div style={{ display: "flex", alignItems: "center", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 12px" }}>
      {/* filter box */}
      <div
        style={{
          display: "flex", alignItems: "center", gap: 8, height: 32,
          flex: "0 1 280px", minWidth: 0, padding: "0 11px", borderRadius: 8,
          background: "var(--color-glass)", border: "1px solid var(--color-glass-border)",
        }}
      >
        <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Find in tracks"
          spellCheck={false}
          style={{
            flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none",
            background: "transparent", color: "var(--color-text-hi)",
            fontSize: 13, fontWeight: 400, fontFamily: "inherit",
          }}
        />
        {query && (
          <button
            onClick={() => setQuery("")}
            title="Clear"
            style={{ display: "flex", border: "none", background: "transparent", color: "var(--color-text-dim)", cursor: "pointer", padding: 0, flexShrink: 0 }}
          >
            <X size={13} strokeWidth={2.4} />
          </button>
        )}
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 10, marginLeft: "auto", flexShrink: 0 }}>
        <Dropdown value={order} options={orderOptions} onChange={setOrder} align="right" minWidth={150} title="Sort by" />
        <Dropdown value={dir}   options={dirOptions}   onChange={setDir}   align="right" minWidth={140} title="Direction" />
      </div>
    </div>
  );

  return { view, keys, toolbar };
}
