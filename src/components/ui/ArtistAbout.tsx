import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowUpRight } from "@/lib/icons";
import { CoverArt } from "./CoverArt";
import { SectionTitle } from "./SectionTitle";

/* the artist page's About card: a gallery portrait beside the biography, with
the facts spotify has (active years, monthly listeners, followers) underneath.
`stats` lines come pre-formatted from the page. */
export function ArtistAbout({
  name,
  biography,
  image,
  stats,
  cities = [],
  links = [],
}: {
  name: string;
  biography: string | null;
  image: string | null;
  stats: string[];
  cities?: { city: string; country: string | null; listeners: number | null }[];
  links?: { name: string; url: string }[];
}) {
  const [expanded, setExpanded] = useState(false);
  if (!biography && stats.length === 0 && cities.length === 0) return null;
  const long = (biography?.length ?? 0) > 420;
  const text = biography && long && !expanded ? `${biography.slice(0, 420).trimEnd()}...` : biography;

  return (
    <section aria-labelledby="artist-about">
      <SectionTitle id="artist-about">About</SectionTitle>
      <div
        style={{
          display: "flex",
          gap: "clamp(16px, 2vw, 24px)",
          flexWrap: "wrap",
          padding: "clamp(14px, 1.6vw, 20px)",
          borderRadius: 16,
          background: "var(--color-glass)",
          border: "1px solid var(--color-glass-border)",
        }}
      >
        {image && (
          <div style={{ width: "clamp(140px, 18vw, 220px)", aspectRatio: "1 / 1", borderRadius: 12, overflow: "hidden", flexShrink: 0 }}>
            <CoverArt url={image} alt={name} size={220} style={{ width: "100%", height: "100%" }} />
          </div>
        )}
        <div style={{ flex: "1 1 280px", minWidth: 0, display: "flex", flexDirection: "column", gap: 12 }}>
          {stats.length > 0 && (
            <div className="tnum" style={{ display: "flex", flexWrap: "wrap", gap: "6px 18px" }}>
              {stats.map((s) => (
                <span key={s} style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)" }}>{s}</span>
              ))}
            </div>
          )}
          {text && (
            <p style={{ margin: 0, fontSize: 14, lineHeight: 1.6, color: "var(--color-text)", whiteSpace: "pre-line" }}>{text}</p>
          )}
          {long && (
            <button
              type="button"
              className="btn-text focus-ring"
              onClick={() => setExpanded((v) => !v)}
              style={{ alignSelf: "flex-start", padding: 0, fontSize: 13, fontWeight: 600 }}
            >
              {expanded ? "Show less" : "Read more"}
            </button>
          )}
          {cities.length > 0 && (
            <div>
              <div className="t-caption" style={{ fontSize: 11.5, fontWeight: 700, letterSpacing: "0.05em", textTransform: "uppercase", color: "var(--color-text-dim)", marginBottom: 6 }}>
                Where people listen
              </div>
              <ol style={{ margin: 0, padding: 0, listStyle: "none", display: "flex", flexDirection: "column", gap: 4 }}>
                {cities.slice(0, 5).map((c) => (
                  <li key={`${c.city}-${c.country}`} className="tnum" style={{ fontSize: 13, color: "var(--color-text)", display: "flex", justifyContent: "space-between", gap: 12, maxWidth: 360 }}>
                    <span>{c.city}{c.country ? `, ${c.country}` : ""}</span>
                    {c.listeners != null && <span style={{ color: "var(--color-text-dim)" }}>{c.listeners.toLocaleString()} listeners</span>}
                  </li>
                ))}
              </ol>
            </div>
          )}
          {links.length > 0 && (
            <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
              {links.map((l) => (
                <button
                  key={l.url}
                  type="button"
                  className="btn-pill focus-ring"
                  onClick={() => openUrl(l.url).catch(() => {})}
                  style={{ display: "inline-flex", alignItems: "center", gap: 4, textTransform: "capitalize" }}
                >
                  {l.name.toLowerCase()}
                  <ArrowUpRight size={12} />
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
