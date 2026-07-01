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
