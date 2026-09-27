import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "@/lib/icons";
import { getTrackCredits, type CreditPerson } from "../../api/internal";
import { CoverArt } from "./CoverArt";
import { meshGradient } from "../../lib/mesh";
import { errMsg } from "../../lib/err";

function Avatar({ person, size }: { person: CreditPerson; size: number }) {
  if (person.image_url) {
    return <CoverArt url={person.image_url} alt="" size={size} rounded style={{ width: size, height: size, flexShrink: 0 }} />;
  }
  // no photo: the same seeded gradient the app uses for art-less tracks
  return (
    <span
      aria-hidden
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: size * 0.38,
        fontWeight: 700,
        color: "rgba(255,255,255,0.92)",
        textShadow: "0 1px 2px rgba(0,0,0,0.35)",
        overflow: "hidden",
        position: "relative",
      }}
    >
      {/* the gradient spins/flips itself for variety, so it gets its own layer */}
      <span style={{ position: "absolute", inset: 0, ...meshGradient(person.name) }} />
      <span style={{ position: "relative" }}>{person.name.slice(0, 1).toUpperCase()}</span>
    </span>
  );
}

function Skeleton() {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }} aria-hidden>
      {[0, 1, 2, 3].map((i) => (
        <div key={i} style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <span style={{ width: 40, height: 40, borderRadius: "50%", background: "rgba(255,255,255,0.06)" }} />
          <span style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <span style={{ width: 120 + i * 18, height: 11, borderRadius: 4, background: "rgba(255,255,255,0.08)" }} />
            <span style={{ width: 70, height: 9, borderRadius: 4, background: "rgba(255,255,255,0.05)" }} />
          </span>
        </div>
      ))}
    </div>
  );
}

/* the performers / writers / producers of one track, grouped the way spotify
groups them. people who are also spotify artists link through to their page.
`dark` is the immersive view, which sits on the artwork instead of a panel. */
export function CreditsList({ trackId, onNavigate, dark }: { trackId: string; onNavigate?: () => void; dark?: boolean }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["credits", trackId],
    queryFn: () => getTrackCredits(trackId),
    staleTime: Infinity,
    retry: false,
  });
  const dim = dark ? "rgba(255,255,255,0.62)" : "var(--color-text-dim)";
  const hi = dark ? "#fff" : "var(--color-text-hi)";
  const chipBg = dark ? "rgba(255,255,255,0.12)" : "rgba(255,255,255,0.07)";

  if (isLoading) return <Skeleton />;
  if (error) return <p className="t-caption" style={{ color: dim, margin: 0 }}>Couldn't load credits: {errMsg(error)}</p>;
  if (!data || data.sections.length === 0) {
    return <p className="t-caption" style={{ color: dim, margin: 0 }}>Spotify has no credits for this track.</p>;
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 22 }}>
      {data.sections.map((sec) => (
        <section key={sec.title}>
          <h3 style={{ margin: "0 0 10px", fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: dim }}>{sec.title}</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {sec.people.map((p, i) => {
              const body = (
                <>
                  <Avatar person={p} size={40} />
                  <span style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", gap: 5 }}>
                    <span style={{ fontSize: 14, fontWeight: 650, color: hi, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
                    {p.roles.length > 0 && (
                      <span style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
                        {p.roles.map((r) => (
                          <span key={r} style={{ fontSize: 11, fontWeight: 600, lineHeight: "18px", padding: "0 7px", borderRadius: 99, background: chipBg, color: dark ? "rgba(255,255,255,0.85)" : "var(--color-text)" }}>
                            {r}
                          </span>
                        ))}
                      </span>
                    )}
                  </span>
                </>
              );
              const style = { display: "flex", alignItems: "center", gap: 12, textDecoration: "none", color: "inherit", borderRadius: 10, padding: "6px 8px", margin: "0 -8px" } as const;
              return p.artist_id ? (
                <Link key={`${p.name}-${i}`} to={`/artist/${p.artist_id}`} onClick={onNavigate} className="row-btn focus-ring credit-link" style={style}>
                  {body}
                  <ChevronRight size={15} className="credit-chev" style={{ color: dim, flexShrink: 0 }} />
                </Link>
              ) : (
                <div key={`${p.name}-${i}`} style={style}>{body}</div>
              );
            })}
          </div>
        </section>
      ))}
      {data.sources.length > 0 && (
        <p className="t-caption" style={{ fontSize: 11.5, color: dim, margin: 0, paddingTop: 12, borderTop: `1px solid ${dark ? "rgba(255,255,255,0.12)" : "var(--color-divider)"}` }}>
          Source: {data.sources.join(", ")}
        </p>
      )}
    </div>
  );
}
