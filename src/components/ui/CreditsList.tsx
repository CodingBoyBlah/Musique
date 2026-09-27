import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { getTrackCredits } from "../../api/internal";
import { CoverArt } from "./CoverArt";
import { errMsg } from "../../lib/err";

/* the performers / writers / producers of one track, grouped the way spotify
groups them. people who are also spotify artists link to their page. */
export function CreditsList({ trackId, onNavigate, dark }: { trackId: string; onNavigate?: () => void; dark?: boolean }) {
  const { data, isLoading, error } = useQuery({
    queryKey: ["credits", trackId],
    queryFn: () => getTrackCredits(trackId),
    staleTime: Infinity,
    retry: false,
  });
  const dim = dark ? "rgba(255,255,255,0.6)" : "var(--color-text-dim)";
  const hi = dark ? "#fff" : "var(--color-text-hi)";

  if (isLoading) return <p className="t-caption" style={{ color: dim, padding: "8px 0" }}>Loading credits...</p>;
  if (error) return <p className="t-caption" style={{ color: dim, padding: "8px 0" }}>Couldn't load credits: {errMsg(error)}</p>;
  if (!data || data.sections.length === 0) {
    return <p className="t-caption" style={{ color: dim, padding: "8px 0" }}>Spotify has no credits for this track.</p>;
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
      {data.sections.map((sec) => (
        <section key={sec.title}>
          <h3 style={{ margin: "0 0 8px", fontSize: 13, fontWeight: 700, letterSpacing: "0.02em", color: hi }}>{sec.title}</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {sec.people.map((p, i) => {
              const body = (
                <>
                  {p.image_url ? (
                    <CoverArt url={p.image_url} alt="" size={36} rounded style={{ width: 36, height: 36, flexShrink: 0 }} />
                  ) : (
                    <span aria-hidden style={{ width: 36, height: 36, borderRadius: "50%", background: "rgba(255,255,255,0.08)", flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 14, fontWeight: 700, color: dim }}>
                      {p.name.slice(0, 1).toUpperCase()}
                    </span>
                  )}
                  <span style={{ minWidth: 0, display: "flex", flexDirection: "column" }}>
                    <span style={{ fontSize: 13.5, fontWeight: 600, color: hi, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
                    {p.roles.length > 0 && (
                      <span className="t-caption" style={{ fontSize: 12, color: dim }}>{p.roles.join(", ")}</span>
                    )}
                  </span>
                </>
              );
              const style = { display: "flex", alignItems: "center", gap: 10, textDecoration: "none", color: "inherit", borderRadius: 8, padding: "2px 4px" } as const;
              return p.artist_id ? (
                <Link key={`${p.name}-${i}`} to={`/artist/${p.artist_id}`} onClick={onNavigate} className="row-btn focus-ring" style={style}>
                  {body}
                </Link>
              ) : (
                <div key={`${p.name}-${i}`} style={style}>{body}</div>
              );
            })}
          </div>
        </section>
      ))}
      {data.sources.length > 0 && (
        <p className="t-caption" style={{ fontSize: 11.5, color: dim, margin: 0 }}>Source: {data.sources.join(", ")}</p>
      )}
    </div>
  );
}
