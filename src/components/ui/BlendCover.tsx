import { memo } from "react";
import { coverUrl } from "../../lib/coverUrl";
import "../../styles/blend.css";

export interface BlendAvatarMember {
  username: string;
  name: string;
  imageUrl: string | null;
}

// vivid two-tone pairs in the spirit of the mobile Blend covers
const PAIRS: [string, string][] = [
  ["#ff4632", "#f573a0"],
  ["#1e3264", "#27856a"],
  ["#8d67ab", "#e8115b"],
  ["#0d73ec", "#9cf0e1"],
  ["#e91429", "#ffc864"],
  ["#509bf5", "#c39687"],
  ["#148a08", "#cdf564"],
  ["#af2896", "#ff6437"],
  ["#4100f4", "#f037a5"],
  ["#006450", "#b49bc8"],
];

function hash(s: string) {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return h >>> 0;
}

export function blendColors(id: string): [string, string] {
  return PAIRS[hash(id) % PAIRS.length];
}

function initials(name: string) {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  return (parts[0][0] + (parts.length > 1 ? parts[parts.length - 1][0] : "")).toUpperCase();
}

/* members of a Blend's name ("Alex + Sam", "Alex, Sam + 2 more") for when
getBlendInfo is not there (yet) */
export function namesFromBlendTitle(name: string): string[] {
  if (!/\+/.test(name)) return [];
  return name
    .split(/\s*[+,]\s*/)
    .map((s) => s.trim())
    .filter((s) => s && !/^\d+ more$/i.test(s))
    .slice(0, 4);
}

function Avatar({ m, size, ring, overlap }: { m: BlendAvatarMember; size: number; ring?: string; overlap: number }) {
  return (
    <span
      className="blend-avatar"
      title={m.name || m.username}
      style={{
        width: size, height: size, marginLeft: overlap, fontSize: Math.max(9, size * 0.36),
        ...(ring ? ({ "--blend-ring": ring } as React.CSSProperties) : null),
        ["--blend-ring-w" as string]: `${Math.max(2, Math.round(size * 0.05))}px`,
      }}
    >
      {m.imageUrl ? (
        <img src={coverUrl(m.imageUrl, size > 64 ? 300 : 64) ?? m.imageUrl} alt="" loading="lazy" decoding="async" />
      ) : (
        initials(m.name || m.username)
      )}
    </span>
  );
}

/** overlapping avatar stack: up to 3 faces, then "+N" */
export function BlendAvatars({
  members,
  size = 28,
  className,
  ring,
}: {
  members: { username: string; name: string; imageUrl: string | null }[];
  size?: number;
  className?: string;
  /** internal: ring colour (defaults to the page background) */
  ring?: string;
}) {
  if (members.length === 0) return null;
  const shown = members.slice(0, 3);
  const extra = members.length - shown.length;
  const overlap = -Math.round(size * 0.3);
  return (
    <span className={className ? `blend-avatars ${className}` : "blend-avatars"} aria-label={members.map((m) => m.name || m.username).join(", ")}>
      {shown.map((m, i) => (
        <Avatar key={m.username || i} m={m} size={size} ring={ring} overlap={i === 0 ? 0 : overlap} />
      ))}
      {extra > 0 && (
        <span
          className="blend-avatar blend-avatar--more tnum"
          style={{
            width: size, height: size, marginLeft: overlap, fontSize: Math.max(9, size * 0.32),
            ...(ring ? ({ "--blend-ring": ring } as React.CSSProperties) : null),
          }}
        >
          +{extra}
        </span>
      )}
    </span>
  );
}

/* the mobile-style Blend tile cover: the list's own cover when it has one,
otherwise a two-tone gradient (stable per id) with the members' faces
overlapping in the middle and a "Blend" wordmark */
export const BlendCover = memo(function BlendCover({
  id,
  name,
  imageUrl,
  members,
  size = 160,
  className,
}: {
  id: string;
  name: string;
  imageUrl?: string | null;
  members?: BlendAvatarMember[] | null;
  /** rendered px, picks the image/avatars size */
  size?: number;
  className?: string;
}) {
  const cls = className ? `blend-cover ${className}` : "blend-cover";
  if (imageUrl) {
    return (
      <div className={cls}>
        <img className="blend-cover__img" src={coverUrl(imageUrl, size > 200 ? 640 : 300) ?? imageUrl} alt={name} loading="lazy" decoding="async" />
      </div>
    );
  }
  const [a, b] = blendColors(id);
  const faces: BlendAvatarMember[] =
    members && members.length > 0
      ? members
      : namesFromBlendTitle(name).map((n) => ({ username: n, name: n, imageUrl: null }));
  const avatarSize = Math.round(size * 0.36);
  return (
    <div className={cls} role="img" aria-label={name}>
      <div className="blend-cover__bg" style={{ background: `linear-gradient(135deg, ${a} 0%, ${a} 48%, ${b} 52%, ${b} 100%)` }}>
        <BlendAvatars members={faces.slice(0, 2)} size={avatarSize} ring={a} />
      </div>
      <span className="blend-cover__wordmark">Blend</span>
    </div>
  );
});
