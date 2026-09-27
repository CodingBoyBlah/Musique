import { openUrl } from "@tauri-apps/plugin-opener";
import { SectionTitle } from "./SectionTitle";
import { CoverArt } from "./CoverArt";
import type { ArtistOverview } from "../../api/internal";

function dateParts(iso: string | null): { month: string; day: string; rest: string } | null {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  return {
    month: d.toLocaleDateString(undefined, { month: "short" }).toUpperCase(),
    day: String(d.getDate()),
    rest: d.toLocaleString(undefined, { weekday: "short", hour: "numeric", minute: "2-digit" }),
  };
}

// upcoming shows and the artist's shop, when spotify lists them
export function ArtistTour({ concerts, merch }: { concerts: ArtistOverview["concerts"]; merch: ArtistOverview["merch"] }) {
  return (
    <>
      {concerts.length > 0 && (
        <section aria-labelledby="artist-tour">
          <SectionTitle id="artist-tour">On tour</SectionTitle>
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            {concerts.slice(0, 8).map((c, i) => {
              const d = dateParts(c.date);
              return (
                <div key={`${c.title}-${i}`} style={{ display: "flex", alignItems: "center", gap: 14, padding: "8px 10px", borderRadius: 10, background: "var(--color-glass)" }}>
                  <div className="tnum" style={{ width: 44, textAlign: "center", flexShrink: 0 }}>
                    <div style={{ fontSize: 11, fontWeight: 700, color: "var(--color-accent)" }}>{d?.month ?? "TBA"}</div>
                    <div style={{ fontSize: 20, fontWeight: 800, color: "var(--color-text-hi)", lineHeight: 1.1 }}>{d?.day ?? ""}</div>
                  </div>
                  <div style={{ minWidth: 0 }}>
                    <div style={{ fontSize: 14, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {c.city ?? c.title}
                    </div>
                    <div className="t-caption" style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>
                      {[c.venue, d?.rest].filter(Boolean).join(" · ")}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}
      {merch.length > 0 && (
        <section aria-labelledby="artist-merch">
          <SectionTitle id="artist-merch">Merch</SectionTitle>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))", gap: 12 }}>
            {merch.slice(0, 8).map((m) => (
              <button
                key={m.name}
                type="button"
                className="row-btn focus-ring"
                disabled={!m.url}
                onClick={() => m.url && openUrl(m.url).catch(() => {})}
                style={{ display: "flex", flexDirection: "column", gap: 6, padding: 8, borderRadius: 10, border: "none", background: "transparent", textAlign: "left", cursor: m.url ? "pointer" : "default", color: "inherit" }}
              >
                <div style={{ width: "100%", aspectRatio: "1 / 1", borderRadius: 8, overflow: "hidden" }}>
                  <CoverArt url={m.image_url} alt={m.name} size={160} style={{ width: "100%", height: "100%" }} />
                </div>
                <span style={{ fontSize: 13, fontWeight: 600, color: "var(--color-text-hi)", display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical", overflow: "hidden" }}>{m.name}</span>
                {m.price && <span className="t-caption tnum" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>{m.price}</span>}
              </button>
            ))}
          </div>
        </section>
      )}
    </>
  );
}
