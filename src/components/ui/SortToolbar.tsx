import { useMemo, useState, type ReactNode } from "react";
import { Search, X } from "@/lib/icons";
import { Dropdown, type DropdownOption } from "./Dropdown";
import { SortDirectionToggle } from "./SortDirectionToggle";

type SortKey = "recent" | "name" | "popularity";
type Dir = "asc" | "desc";

interface NamedPop {
  name: string;
  popularity?: number | null;
}


const recentOption: DropdownOption<SortKey> = { value: "recent", label: "Recently added" };
const baseOptions: DropdownOption<SortKey>[] = [
  { value: "name",       label: "Name" },
  { value: "popularity", label: "Popularity" },
];
const withRecent = [recentOption, ...baseOptions];

/* With `recent`, `items` must arrive newest-added first (the liked-songs and
   saved-albums queries order by added_at DESC). "Recently added" sorts on that
   arrival order, so no timestamp has to ride along on every item. Followed
   artists leave it off: spotify gives no follow date to sort by. */
const textCollator = new Intl.Collator();

export function useSortTools<T extends NamedPop>(
  items: T[],
  placeholder = "Find",
  { recent = false }: { recent?: boolean } = {},
): { view: T[]; toolbar: ReactNode } {
  const sortOptions = recent ? withRecent : baseOptions;
  const [query, setQuery] = useState("");
  const [sort,  setSort]  = useState<SortKey>("name");
  const [dir,   setDir]   = useState<Dir>("asc");

  // newest-first is the only useful default for recency, so picking it flips
  // the arrow to descending; leaving it goes back to A-Z style ascending
  const changeSort = (next: SortKey) => {
    if (next === sort) return;
    setSort(next);
    if (next === "recent") setDir("desc");
    else if (sort === "recent") setDir("asc");
  };

  const prepared = useMemo(
    () => items.map((item) => ({ item, nameLower: item.name.toLowerCase() })),
    [items],
  );

  const sorted = useMemo(() => {
    if (sort === "recent") {
      // items arrive newest-first: descending keeps arrival order, ascending reverses it
      return dir === "desc" ? prepared : [...prepared].reverse();
    }
    const cmp = sort === "name"
      ? (a: { item: T }, b: { item: T }) => textCollator.compare(a.item.name, b.item.name)
      : (a: { item: T }, b: { item: T }) => (a.item.popularity ?? 0) - (b.item.popularity ?? 0);
    const arr = [...prepared].sort(cmp);
    if (dir === "desc") arr.reverse();
    return arr;
  }, [prepared, sort, dir]);

  const sortedItems = useMemo(() => sorted.map((p) => p.item), [sorted]);

  const view = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sortedItems;
    const out: T[] = [];
    for (let i = 0; i < sorted.length; i++) {
      if (sorted[i].nameLower.includes(q)) out.push(sorted[i].item);
    }
    return out;
  }, [sorted, sortedItems, query]);

  const toolbar = useMemo(() => (
    <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 14px" }}>
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
          placeholder={placeholder}
          aria-label={placeholder}
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
            aria-label="Clear"
            className="btn-icon"
            style={{ width: 20, height: 20, borderRadius: 6, padding: 0 }}
          >
            <X size={13} strokeWidth={2.4} />
          </button>
        )}
      </div>

      <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, marginLeft: "auto" }}>
        <Dropdown value={sort} options={sortOptions} onChange={changeSort} align="right" minWidth={170} title="Sort by" />
        <SortDirectionToggle dir={dir} onChange={setDir} />
      </div>
    </div>
  ), [query, sort, dir, placeholder, sortOptions]);

  return { view, toolbar };
}
