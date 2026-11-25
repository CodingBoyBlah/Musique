import { memo } from "react";
import type { BlendMember } from "../../api/blend";
import { Tooltip } from "./Tooltip";

/* Blend bits: member faces for the header and for the right side of each track row */

const HUES = [152, 330, 205, 28, 268, 92];
function hueFor(name: string) {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
  return HUES[h % HUES.length];
}

export const MemberFace = memo(function MemberFace({ member, size = 16, ring }: { member: BlendMember; size?: number; ring?: string }) {
  const style: React.CSSProperties = {
    width: size, height: size, borderRadius: "50%", flexShrink: 0, objectFit: "cover",
    boxShadow: ring ? `0 0 0 2px ${ring}` : undefined,
  };
  if (member.imageUrl) {
    return <img src={member.imageUrl} alt="" draggable={false} decoding="async" loading="lazy" style={style} />;
  }
  return (
    <span
      aria-hidden
      style={{
        ...style, display: "inline-flex", alignItems: "center", justifyContent: "center",
        background: `oklch(0.62 0.15 ${hueFor(member.username)})`, color: "#0b0b0b",
        fontSize: Math.max(8, size * 0.46), fontWeight: 800, lineHeight: 1,
      }}
    >
      {(member.name || member.username).trim().charAt(0).toUpperCase()}
    </span>
  );
});

/** right side of a track row: faces of whoever the track was picked for */
export function TrackFaces({ members }: { members: BlendMember[] }) {
  const label = joinNames(members.map((m) => m.name));
  return (
    <Tooltip label={label} side="top">
      <span className="blend-track-faces" aria-label={`Picked for ${label}`}>
        {members.slice(0, 3).map((m) => <MemberFace key={m.username} member={m} size={24} ring="var(--blend-face-ring)" />)}
      </span>
    </Tooltip>
  );
}

function joinNames(names: string[]) {
  if (names.length <= 1) return names[0] ?? "";
  if (names.length === 2) return `${names[0]} and ${names[1]}`;
  return `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

interface Props {
  members: BlendMember[];
  tasteMatch: number | null;
  trackCount: number;
}

/* faces + "anshuman and sky" and a quiet meta line */
export function BlendHeader({ members, tasteMatch, trackCount }: Props) {
  return (
    <div className="blend-header">
      <div className="blend-header-row">
        {members.length > 0 && (
          <span className="blend-faces" aria-hidden>
            {members.slice(0, 4).map((m, i) => (
              <span key={m.username} style={{ marginLeft: i ? -8 : 0, zIndex: 10 - i, display: "inline-flex" }}>
                <MemberFace member={m} size={26} ring="var(--blend-face-ring)" />
              </span>
            ))}
          </span>
        )}
        <div style={{ minWidth: 0 }}>
          <p className="blend-madefor">{members.length ? joinNames(members.map((m) => m.name)) : "Your Blend"}</p>
          <p className="blend-meta tnum">
            Blend · {trackCount} {trackCount === 1 ? "song" : "songs"} · Updates daily
            {tasteMatch != null && <> · <span className="blend-match">{Math.round(tasteMatch)}% taste match</span></>}
          </p>
        </div>
      </div>

    </div>
  );
}
