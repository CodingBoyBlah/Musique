import { useMemo, useState, type ReactNode } from "react";
import { Search, X } from "lucide-react";
import { Dropdown, type DropdownOption } from "./Dropdown";

type SortKey = "name" | "popularity";
type Dir = "asc" | "desc";

interface NamedPop {
  name: string;
  popularity?: number | null;
}


export function useSortTools<T extends NamedPop>(
  items: T[],
  placeholder = "Find",
): { view: T[]; toolbar: ReactNode } {
  const [query, setQuery] = useState("");
  const [sort,  setSort]  = useState<SortKey>("name");
  const [dir,   setDir]   = useState<Dir>("asc");

  const view = useMemo(() => {
    let arr = items;
    const q = query.trim().toLowerCase();
    if (q) arr = arr.filter((i) => i.name.toLowerCase().includes(q));

    const cmp = sort === "name"
      ? (a: T, b: T) => a.name.localeCompare(b.name)
      : (a: T, b: T) => (a.popularity ?? 0) - (b.popularity ?? 0);
    arr = [...arr].sort(cmp);
    if (dir === "desc") arr.reverse();
    return arr;
  }, [items, query, sort, dir]);

  const sortOptions: DropdownOption<SortKey>[] = [
    { value: "name",       label: "Name" },
    { value: "popularity", label: "Popularity" },
  ];
  const dirOptions: DropdownOption<Dir>[] = [
    { value: "asc",  label: "Ascending" },
    { value: "desc", label: "Descending" },
  ];

  const toolbar = (
    <div style={{ display: "flex", alignItems: "center", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 14px" }}>
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
          placeholder={placeholder}
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
        <Dropdown value={sort} options={sortOptions} onChange={setSort} align="right" minWidth={150} title="Sort by" />
        <Dropdown value={dir}  options={dirOptions}  onChange={setDir}  align="right" minWidth={140} title="Direction" />
      </div>
    </div>
  );

  return { view, toolbar };
}
