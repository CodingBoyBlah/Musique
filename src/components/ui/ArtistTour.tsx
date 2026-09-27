import { openUrl } from "@tauri-apps/plugin-opener";
import { motion } from "framer-motion";
import { ArrowUpRight } from "@/lib/icons";
import { SectionTitle } from "./SectionTitle";
import { CoverArt } from "./CoverArt";
import { Shelf } from "./Shelf";
import { PRESS, PRESS_TRANSITION } from "../../lib/motion";
import type { ArtistOverview } from "../../api/internal";

function dateParts(iso: string | null) {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  return {
    month: d.toLocaleDateString(undefined, { month: "short" }).toUpperCase(),
    day: String(d.getDate()),
    weekday: d.toLocaleDateString(undefined, { weekday: "short" }),
    time: d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" }),
  };
}

// upcoming shows as date cards, and the artist's shop as a shelf
export function ArtistTour({ concerts, merch }: { concerts: ArtistOverview["concerts"]; merch: ArtistOverview["merch"] }) {
  return (
    <>
      {concerts.length > 0 && (
        <section aria-labelledby="artist-tour">
          <SectionTitle id="artist-tour">On tour</SectionTitle>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: 10 }}>
            {concerts.slice(0, 9).map((c, i) => {
              const d = dateParts(c.date);
              return (
                <div
                  key={`${c.title}-${i}`}
                  style={{ display: "flex", alignItems: "center", gap: 14, padding: 12, borderRadius: 14, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", minWidth: 0 }}
                >
                  <div className="tnum" style={{ width: 52, height: 56, flexShrink: 0, borderRadius: 10, overflow: "hidden", background: "rgba(255,255,255,0.06)", display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <div style={{ width: "100%", textAlign: "center", fontSize: 10.5, fontWeight: 800, letterSpacing: "0.06em", color: "#fff", background: "var(--color-accent)", padding: "3px 0" }}>{d?.month ?? "TBA"}</div>
                    <div style={{ flex: 1, display: "flex", alignItems: "center", fontSize: 21, fontWeight: 800, color: "var(--color-text-hi)" }}>{d?.day ?? "–"}</div>
                  </div>
                  <div style={{ minWidth: 0 }}>
                    <div style={{ fontSize: 14, fontWeight: 650, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{c.city ?? c.title}</div>
                    <div className="t-caption" style={{ fontSize: 12.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{c.venue ?? c.title}</div>
                    {d && <div className="t-caption tnum" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>{d.weekday} · {d.time}</div>}
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}
      <Shelf
        id="artist-merch"
        title="Merch"
        items={merch}
        getKey={(m) => m.name}
        renderItem={(m) => (
          <motion.button
            type="button"
            className="card-link focus-ring"
            disabled={!m.url}
            onClick={() => m.url && openUrl(m.url).catch(() => {})}
            whileHover={m.url ? { y: -3 } : undefined}
            whileTap={m.url ? PRESS : undefined}
            transition={PRESS_TRANSITION}
            style={{ display: "flex", flexDirection: "column", gap: 8, padding: 10, width: "100%", borderRadius: 12, border: "none", background: "transparent", textAlign: "left", cursor: m.url ? "pointer" : "default", color: "inherit" }}
          >
            <div style={{ position: "relative", width: "100%", aspectRatio: "1 / 1", borderRadius: 8, overflow: "hidden", background: "#fff", boxShadow: "0 4px 14px rgba(0,0,0,0.3)" }}>
              <CoverArt url={m.image_url} alt={m.name} size={175} style={{ width: "100%", height: "100%" }} />
              {m.url && (
                <span style={{ position: "absolute", top: 8, right: 8, width: 26, height: 26, borderRadius: 99, background: "rgba(0,0,0,0.55)", color: "#fff", display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <ArrowUpRight size={13} />
                </span>
              )}
            </div>
            <span style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{m.name}</span>
            {m.price && <span className="t-caption tnum" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>{m.price}</span>}
          </motion.button>
        )}
      />
    </>
  );
}
