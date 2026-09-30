import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowUpRight, ChevronLeft, ChevronRight, X, Globe } from "@/lib/icons";
import { CoverArt } from "./CoverArt";
import { SectionTitle } from "./SectionTitle";
import { Modal } from "./Modal";
import { coverUrl } from "../../lib/coverUrl";
import { EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";

export interface AboutStats {
  monthlyListeners?: number | null;
  followers?: number | null;
  worldRank?: number | null;
  activeYears?: string | null;
}

interface Props {
  name: string;
  biography: string | null;
  images: string[];
  stats: AboutStats;
  cities?: { city: string; country: string | null; listeners: number | null }[];
  links?: { name: string; url: string }[];
}

const compact = new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 });

function linkLabel(name: string) {
  const n = name.toLowerCase();
  return n === "twitter" ? "X / Twitter" : n.charAt(0).toUpperCase() + n.slice(1);
}

function Links({ links }: { links: Props["links"] }) {
  if (!links?.length) return null;
  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
      {links.map((l) => (
        <motion.button
          key={l.url}
          type="button"
          className="ghost-pill focus-ring"
          whileTap={PRESS}
          transition={PRESS_TRANSITION}
          onClick={() => openUrl(l.url).catch(() => {})}
          style={{ height: 30, padding: "0 12px", borderRadius: 99, color: "#fff", fontSize: 12.5, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 5, cursor: "pointer" }}
        >
          {linkLabel(l.name)}
          <ArrowUpRight size={12} />
        </motion.button>
      ))}
    </div>
  );
}

// where the listening comes from, each city as a bar against the biggest
function Cities({ cities }: { cities: NonNullable<Props["cities"]> }) {
  const max = Math.max(1, ...cities.map((c) => c.listeners ?? 0));
  return (
    <ol style={{ margin: 0, padding: 0, listStyle: "none", display: "flex", flexDirection: "column", gap: 10 }}>
      {cities.slice(0, 5).map((c, i) => (
        <li key={`${c.city}-${c.country}`} title={c.listeners != null ? `${c.listeners.toLocaleString()} listeners` : undefined}>
          <div style={{ display: "flex", justifyContent: "space-between", gap: 10, fontSize: 13, marginBottom: 5 }}>
            <span style={{ color: "var(--color-text-hi)", fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              <span className="tnum" style={{ color: "var(--color-text-dim)", marginRight: 8 }}>{i + 1}</span>
              {c.city}
              {c.country && <span style={{ color: "var(--color-text-dim)", fontWeight: 500 }}>, {c.country}</span>}
            </span>
            {c.listeners != null && <span className="tnum" style={{ color: "var(--color-text-dim)", flexShrink: 0 }}>{compact.format(c.listeners)}</span>}
          </div>
          <div style={{ height: 4, borderRadius: 4, background: "rgba(255,255,255,0.06)", overflow: "hidden" }}>
            <div style={{ width: `${((c.listeners ?? 0) / max) * 100}%`, height: "100%", borderRadius: 4, background: "var(--color-accent)" }} />
          </div>
        </li>
      ))}
    </ol>
  );
}

function StatTile({ value, label }: { value: string; label: string }) {
  return (
    <div style={{ padding: "14px 16px", borderRadius: 14, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", minWidth: 0 }}>
      <div className="tnum" style={{ fontSize: 22, fontWeight: 800, letterSpacing: "-0.02em", color: "var(--color-text-hi)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{value}</div>
      <div className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)", marginTop: 2 }}>{label}</div>
    </div>
  );
}

// the full about: gallery, whole biography, every number
function AboutModal({ open, onClose, name, biography, images, stats, cities, links }: Props & { open: boolean; onClose: () => void }) {
  const [i, setI] = useState(0);
  const img = images[i % Math.max(1, images.length)];
  return (
    <Modal open={open} onClose={onClose} labelledBy="artist-about-title" panelStyle={{ width: "min(720px, calc(100vw - 32px))", maxHeight: "86vh", padding: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}>
      <div style={{ position: "relative", height: 340, flexShrink: 0, background: "#000" }}>
        <AnimatePresence initial={false}>
          {img && (
            <motion.img
              key={img}
              src={coverUrl(img, 900) ?? img}
              alt=""
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.3, ease: EASE_OUT }}
              style={{ position: "absolute", inset: 0, width: "100%", height: "100%", objectFit: "cover", objectPosition: "center 30%" }}
            />
          )}
        </AnimatePresence>
        <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(0,0,0,0.25) 0%, transparent 30%, transparent 55%, var(--color-popover) 100%)" }} />
        <button onClick={onClose} aria-label="Close" className="btn-icon" style={{ position: "absolute", top: 12, right: 12, width: 32, height: 32, borderRadius: 99, background: "rgba(0,0,0,0.45)", color: "#fff" }}>
          <X size={16} />
        </button>
        {images.length > 1 && (
          <div style={{ position: "absolute", right: 14, bottom: 14, display: "flex", gap: 6 }}>
            <button className="btn-icon" aria-label="Previous photo" onClick={() => setI((v) => (v - 1 + images.length) % images.length)} style={{ width: 30, height: 30, borderRadius: 99, background: "rgba(0,0,0,0.5)", color: "#fff" }}><ChevronLeft size={15} /></button>
            <button className="btn-icon" aria-label="Next photo" onClick={() => setI((v) => (v + 1) % images.length)} style={{ width: 30, height: 30, borderRadius: 99, background: "rgba(0,0,0,0.5)", color: "#fff" }}><ChevronRight size={15} /></button>
          </div>
        )}
        <h2 id="artist-about-title" style={{ position: "absolute", left: 22, bottom: 14, margin: 0, fontSize: 30, fontWeight: 800, letterSpacing: "-0.02em", color: "#fff" }}>{name}</h2>
      </div>
      <div className="scroll-y" style={{ overflowY: "auto", padding: "8px 22px 22px", display: "flex", flexDirection: "column", gap: 18 }}>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: 8 }}>
          {stats.monthlyListeners != null && <StatTile value={stats.monthlyListeners.toLocaleString()} label="Monthly listeners" />}
          {stats.followers != null && <StatTile value={stats.followers.toLocaleString()} label="Followers" />}
          {stats.worldRank ? <StatTile value={`#${stats.worldRank}`} label="In the world" /> : null}
          {stats.activeYears && (
            stats.activeYears.includes("present")
              ? <StatTile value={stats.activeYears.split(" ")[0]} label="Active since" />
              : <StatTile value={stats.activeYears.replace(" - ", "–")} label="Active" />
          )}
        </div>
        {biography && <p style={{ margin: 0, fontSize: 14.5, lineHeight: 1.65, color: "var(--color-text)", whiteSpace: "pre-line" }}>{biography}</p>}
        {cities && cities.length > 0 && (
          <div>
            <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-dim)", marginBottom: 10 }}>Where people listen</div>
            <Cities cities={cities} />
          </div>
        )}
        <Links links={links} />
      </div>
    </Modal>
  );
}

/* the artist page's About: a photo card carrying the headline numbers and the
start of the bio (opens the full about), with where-people-listen beside it */
export function ArtistAbout(props: Props) {
  const { name, biography, images, stats, cities = [], links = [] } = props;
  const [open, setOpen] = useState(false);
  const [hover, setHover] = useState(false);
  if (!biography && stats.monthlyListeners == null && cities.length === 0) return null;
  const hero = images[0];

  return (
    <section aria-labelledby="artist-about">
      <SectionTitle id="artist-about">About</SectionTitle>
      <div style={{ display: "grid", gridTemplateColumns: cities.length > 0 ? "repeat(auto-fit, minmax(min(100%, 320px), 1fr))" : "1fr", gap: 14 }}>
        <motion.button
          type="button"
          onClick={() => setOpen(true)}
          onMouseEnter={() => setHover(true)}
          onMouseLeave={() => setHover(false)}
          whileTap={{ scale: 0.99 }}
          className="focus-ring"
          aria-label={`About ${name}`}
          style={{
            position: "relative",
            minHeight: 360,
            borderRadius: 18,
            overflow: "hidden",
            border: "none",
            padding: 0,
            cursor: "pointer",
            textAlign: "left",
            background: "#0c0c10",
            color: "#fff",
          }}
        >
          {hero ? (
            <motion.img
              src={coverUrl(hero, 900) ?? hero}
              alt=""
              animate={{ scale: hover ? 1.03 : 1 }}
              transition={{ duration: 0.5, ease: EASE_OUT }}
              style={{ position: "absolute", inset: 0, width: "100%", height: "100%", objectFit: "cover", objectPosition: "center 25%" }}
            />
          ) : (
            <CoverArt url={null} alt="" size={360} style={{ position: "absolute", inset: 0, width: "100%", height: "100%" }} />
          )}
          <div style={{ position: "absolute", inset: 0, background: "linear-gradient(180deg, rgba(0,0,0,0.05) 0%, rgba(0,0,0,0.1) 38%, rgba(0,0,0,0.82) 100%)" }} />
          {stats.worldRank ? (
            <span style={{ position: "absolute", top: 16, left: 16, width: 78, height: 78, borderRadius: "50%", background: "var(--color-accent)", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", boxShadow: "0 8px 24px rgba(0,0,0,0.35)" }}>
              <span className="tnum" style={{ fontSize: 20, fontWeight: 800, lineHeight: 1 }}>#{stats.worldRank}</span>
              <span style={{ fontSize: 10, fontWeight: 600, opacity: 0.9, marginTop: 2 }}>in the world</span>
            </span>
          ) : null}
          <div style={{ position: "absolute", left: 20, right: 20, bottom: 18, display: "flex", flexDirection: "column", gap: 8 }}>
            {stats.monthlyListeners != null && (
              <div>
                <span className="tnum" style={{ fontSize: 28, fontWeight: 800, letterSpacing: "-0.02em" }}>{compact.format(stats.monthlyListeners)}</span>
                <span style={{ fontSize: 14, fontWeight: 600, opacity: 0.85, marginLeft: 8 }}>monthly listeners</span>
              </div>
            )}
            {biography && (
              <p style={{ margin: 0, fontSize: 14, lineHeight: 1.5, color: "rgba(255,255,255,0.86)", display: "-webkit-box", WebkitLineClamp: 3, WebkitBoxOrient: "vertical", overflow: "hidden" }}>
                {biography}
              </p>
            )}
            <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 13, fontWeight: 700 }}>
              Read more <ChevronRight size={14} />
            </span>
          </div>
        </motion.button>

        {cities.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: 14, minWidth: 0 }}>
            <div style={{ padding: "18px 18px", borderRadius: 18, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", flex: 1 }}>
              <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-dim)", marginBottom: 14 }}>
                <Globe size={13} /> Where people listen
              </div>
              <Cities cities={cities} />
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14 }}>
              {stats.followers != null && <StatTile value={compact.format(stats.followers)} label="Followers" />}
              {stats.activeYears ? <StatTile value={stats.activeYears.split(" ")[0]} label={stats.activeYears.includes("present") ? "Active since" : `Active ${stats.activeYears}`} /> : null}
            </div>
            <Links links={links} />
          </div>
        )}
      </div>
      <AboutModal {...props} open={open} onClose={() => setOpen(false)} />
    </section>
  );
}
