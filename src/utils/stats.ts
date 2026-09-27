// formatting for the stats page, kept pure so it can be tested

// "12 hr 5 min" / "42 min" / "0 min"
export function listenTime(ms: number): string {
  const mins = Math.floor(ms / 60000);
  if (mins < 60) return `${mins} min`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h.toLocaleString()} hr ${m} min` : `${h.toLocaleString()} hr`;
}

// cell intensity for the heatmap: 0 for empty, then a floor so a single play
// is still visible, rising to 1 at the busiest hour
export function heatLevel(value: number, max: number): number {
  if (value <= 0 || max <= 0) return 0;
  return 0.18 + 0.82 * (value / max);
}

export function heatMax(grid: number[][]): number {
  return grid.reduce((m, row) => Math.max(m, ...row), 0);
}

// the weekday + hour you listen most, or null with no data
export function peakSlot(grid: number[][]): { day: number; hour: number; plays: number } | null {
  let best: { day: number; hour: number; plays: number } | null = null;
  grid.forEach((row, day) =>
    row.forEach((plays, hour) => {
      if (plays > 0 && (!best || plays > best.plays)) best = { day, hour, plays };
    }),
  );
  return best;
}

export function hourLabel(h: number): string {
  const suffix = h < 12 ? "am" : "pm";
  const hr = h % 12 === 0 ? 12 : h % 12;
  return `${hr}${suffix}`;
}
