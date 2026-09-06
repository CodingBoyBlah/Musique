// ms -> m:ss. no hours, tracks are never that long
export function fmtMs(ms: number): string {
  const s = Math.floor(ms / 1000);
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}

// "2024-03-15" -> "2024"
export function releaseYear(date: string | null | undefined): string {
  return date ? date.slice(0, 4) : "";
}


export function releaseDateMs(date: string | null | undefined): number | null {
  if (!date) return null;
  const [y, m = "01", d = "01"] = date.split("-");
  const t = Date.parse(`${y}-${m.padStart(2, "0")}-${d.padStart(2, "0")}T00:00:00`);
  return Number.isNaN(t) ? null : t;
}

// ms from now until release. 0 if missing or already out
export function msUntilRelease(date: string | null | undefined): number {
  const t = releaseDateMs(date);
  return t == null ? 0 : t - Date.now();
}

export function isUpcoming(date: string | null | undefined): boolean {
  return msUntilRelease(date) > 0;
}

// coarsen the units as the wait shrinks down
export function fmtCountdown(ms: number): string {
  if (ms <= 0) return "Out now";
  const s = Math.floor(ms / 1000);
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${sec}s`;
  return `${sec}s`;
}

// Strips raw HTML tags and decodes entities from Spotify playlist/album descriptions
export function cleanDescription(raw: string | null | undefined): string {
  if (!raw) return "";
  try {
    const withBreaks = raw
      .replace(/<br\s*\/?>/gi, "\n")
      .replace(/<\/(p|div|li)>/gi, "\n");
    if (typeof DOMParser !== "undefined") {
      const doc = new DOMParser().parseFromString(withBreaks, "text/html");
      const text = doc.body.textContent || "";
      return text
        .replace(/[ \t]+/g, " ")
        .replace(/\n\s*\n\s*\n+/g, "\n\n")
        .trim();
    }
    return withBreaks
      .replace(/<[^>]*>/g, "")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'")
      .replace(/&apos;/g, "'")
      .replace(/&nbsp;/g, " ")
      .replace(/[ \t]+/g, " ")
      .trim();
  } catch {
    return (raw || "").replace(/<[^>]*>/g, "").trim();
  }
}
