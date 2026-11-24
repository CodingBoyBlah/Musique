import { useMemo, useState, type ReactNode } from "react";
import { Search, X } from "@/lib/icons";
import type { TrackItem } from "../../types/spotify";
import { Dropdown, type DropdownOption } from "./Dropdown";
import { SortDirectionToggle } from "./SortDirectionToggle";
import { Tooltip } from "./Tooltip";

type OrderKey = "default" | "name" | "artist" | "album" | "duration";
type Dir = "asc" | "desc";

const firstArtist = (t: TrackItem) => t.artists[0]?.name ?? "";
const albumName   = (t: TrackItem) => t.album?.name ?? "";
const textCollator = new Intl.Collator();

interface PreparedTrack {
  track: TrackItem;
  key: string;
  nameLower: string;
  artistsLower: string[];
}

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
  fixed key (id + its original position, so dupe tracks stay distinct) and
  precomputed lowercase fields so filter keystrokes allocate nothing */
  const prepared = useMemo<PreparedTrack[]>(
    () =>
      tracks.map((t, i) => ({
        track: t,
        key: `${t.id}#${i}`,
        nameLower: t.name.toLowerCase(),
        artistsLower: t.artists.map((a) => a.name.toLowerCase()),
      })),
    [tracks],
  );

  const sorted = useMemo(() => {
    let arr = prepared;
    if (order !== "default") {
      const cmp: Record<Exclude<OrderKey, "default">, (a: PreparedTrack, b: PreparedTrack) => number> = {
        name:     (a, b) => textCollator.compare(a.track.name, b.track.name),
        artist:   (a, b) => textCollator.compare(firstArtist(a.track), firstArtist(b.track)),
        album:    (a, b) => textCollator.compare(albumName(a.track), albumName(b.track)),
        duration: (a, b) => a.track.duration_ms - b.track.duration_ms,
      };
      arr = [...arr].sort(cmp[order]);
      if (dir === "desc") arr.reverse();
    } else if (dir === "desc") {
      // default order has no comparator, so descending just reverses it
      arr = [...arr].reverse();
    }
    return arr;
  }, [prepared, order, dir]);

  const sortedIdentity = useMemo(
    () => ({
      view: sorted.map((p) => p.track),
      keys: sorted.map((p) => p.key),
    }),
    [sorted],
  );

  const { view, keys } = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sortedIdentity;
    const filteredView: TrackItem[] = [];
    const filteredKeys: string[] = [];
    for (let i = 0; i < sorted.length; i++) {
      const p = sorted[i];
      if (p.nameLower.includes(q) || p.artistsLower.some((a) => a.includes(q))) {
        filteredView.push(p.track);
        filteredKeys.push(p.key);
      }
    }
    return { view: filteredView, keys: filteredKeys };
  }, [sorted, sortedIdentity, query]);

  const orderOptions = useMemo<DropdownOption<OrderKey>[]>(() => [
    { value: "default",  label: defaultOrderLabel },
    { value: "name",     label: "Name" },
    { value: "artist",   label: "Artist" },
    { value: "album",    label: "Album" },
    { value: "duration", label: "Duration" },
  ], [defaultOrderLabel]);

  const toolbar = useMemo(() => (
    <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 12px" }}>
      {/* filter box */}
      <div
        className="focus-within-ring"
        style={{
          display: "flex", alignItems: "center", gap: 8, height: 32,
          flex: "0 1 280px", minWidth: 0, padding: "0 11px", borderRadius: 8,
          background: "var(--color-glass)", border: "1px solid var(--color-glass-border)",
          transition: "border-color 0.14s ease, box-shadow 0.14s ease",
        }}
      >
        <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Find in tracks"
          aria-label="Find in tracks"
          spellCheck={false}
          style={{
            flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none",
            background: "transparent", color: "var(--color-text-hi)",
            fontSize: 13, fontWeight: 400, fontFamily: "inherit",
          }}
        />
        {query && (
          <Tooltip label="Clear filter" side="top">
            <button
              onClick={() => setQuery("")}
              aria-label="Clear filter"
              className="btn-icon"
              style={{ width: 20, height: 20, borderRadius: 6, padding: 0 }}
            >
              <X size={13} strokeWidth={2.4} />
            </button>
          </Tooltip>
        )}
      </div>

      <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, marginLeft: "auto" }}>
        <Dropdown value={order} options={orderOptions} onChange={setOrder} align="right" minWidth={150} title="Sort by" />
        <SortDirectionToggle dir={dir} onChange={setDir} />
      </div>
    </div>
  ), [query, order, dir, orderOptions]);

  return { view, keys, toolbar };
}
