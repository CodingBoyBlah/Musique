/* display names for the lyric providers. the backend hands back a bare slug
and more than one screen has to print it, so the spelling lives here rather
than in a ternary that drifts every time a provider is added. */
const LABELS: Record<string, string> = {
  spotify:    "Spotify",
  musixmatch: "Musixmatch",
  netease:    "NetEase",
  amll:       "AMLL",
  qq:         "QQ Music",
  kugou:      "Kugou",
  lrclib:     "LRCLIB",
};

export function sourceLabel(source: string | undefined): string {
  if (!source) return "LRCLIB";
  return LABELS[source] ?? source.toUpperCase();
}
