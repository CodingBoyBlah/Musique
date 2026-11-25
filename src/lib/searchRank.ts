import type { AlbumItem, ArtistItem, PlaylistCard, SearchResults, TrackItem } from "../types/spotify";
import type { EpisodeItem, ShowItem } from "../types/podcast";

/* one relevance-ordered list out of the per-type lists search returns.

   the api answers "radiohead" with ten songs, ten albums, ten artists... each
   ranked within its own type. showing them as fixed sections (songs, then
   albums, then artists) buries the one thing the query plainly named - the
   artist - under four of their songs. instead every result gets a score from
   how well its own name matches, a smaller one for matching through its
   artist, the api's own rank inside its type, and a little popularity, and
   the lists are merged on that. */

export type RankedResult =
  | { type: "artist"; item: ArtistItem }
  | { type: "track"; item: TrackItem }
  | { type: "album"; item: AlbumItem }
  | { type: "playlist"; item: PlaylistCard }
  | { type: "show"; item: ShowItem }
  | { type: "episode"; item: EpisodeItem };

export type RankedType = RankedResult["type"];

// how many of each type may make the merged list
export type TypeCaps = Partial<Record<RankedType, number>>;

export function normalize(s: string): string {
  return s
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

// "Creep - Remastered 2009" / "Creep (Acoustic)" still count as "creep"
function baseName(n: string): string {
  return n.replace(/\s+[-–(\[].*$/, "").trim() || n;
}

/* 0..100 for how well a name answers the query */
export function nameScore(name: string, q: string): number {
  if (!q) return 0;
  const n = normalize(name);
  if (!n) return 0;
  if (n === q) return 100;
  const b = normalize(baseName(name));
  if (b === q) return 92;
  if (n.startsWith(q)) return 72;
  if (` ${n}`.includes(` ${q}`)) return 52; // a later word starts with it
  if (n.includes(q)) return 30;
  return 0;
}

function bestArtistScore(artists: { name: string }[] | undefined, q: string): number {
  let best = 0;
  for (const a of artists ?? []) best = Math.max(best, nameScore(a.name, q));
  return best;
}

// a type nudge, so equal matches settle the way people usually mean them:
// a bare name is most often an artist, then a song
const TYPE_BIAS: Record<RankedType, number> = {
  artist: 6,
  track: 3,
  album: 0,
  playlist: -12,
  show: -8,
  episode: -14,
};

// each step down the api's own list within a type costs this much
const RANK_STEP = 4;

function pop(p: number | null | undefined): number {
  return (typeof p === "number" ? p : 50) * 0.15;
}

function score(r: RankedResult, q: string, rank: number): number {
  let match: number;
  switch (r.type) {
    case "artist":
      match = nameScore(r.item.name, q) + pop(r.item.popularity);
      break;
    case "track":
      // matching only through the artist counts for less than the song's own name
      match = Math.max(nameScore(r.item.name, q), bestArtistScore(r.item.artists, q) * 0.4)
        + pop(r.item.popularity);
      break;
    case "album":
      match = Math.max(nameScore(r.item.name, q), bestArtistScore(r.item.artists, q) * 0.4)
        + pop(r.item.popularity);
      break;
    case "playlist":
      match = nameScore(r.item.name, q);
      break;
    case "show":
      match = Math.max(nameScore(r.item.name, q), nameScore(r.item.publisher ?? "", q) * 0.4);
      break;
    case "episode":
      match = Math.max(nameScore(r.item.name, q), nameScore(r.item.show_name ?? "", q) * 0.4);
      break;
  }
  return match + TYPE_BIAS[r.type] - rank * RANK_STEP;
}

export function rankSearch(data: SearchResults | undefined, query: string, caps: TypeCaps = {}, limit = Infinity): RankedResult[] {
  if (!data) return [];
  const q = normalize(query);
  const pool: { r: RankedResult; s: number; order: number }[] = [];
  let order = 0;
  const add = <T,>(type: RankedType, list: T[] | undefined) => {
    const cap = caps[type] ?? Infinity;
    (list ?? []).slice(0, cap).forEach((item, rank) => {
      const r = { type, item } as RankedResult;
      pool.push({ r, s: score(r, q, rank), order: order++ });
    });
  };
  add("artist", data.artists);
  add("track", data.tracks);
  add("album", data.albums);
  add("playlist", data.playlists);
  add("show", data.shows);
  add("episode", data.episodes);
  // stable on ties: the api's own order, artists first
  pool.sort((a, b) => b.s - a.s || a.order - b.order);
  return pool.slice(0, limit).map((p) => p.r);
}

export type TopResult = Extract<RankedResult, { type: "artist" | "track" | "album" }>;

// the search page's big top-result card: the best artist, song or album
export function topResult(data: SearchResults | undefined, query: string): TopResult | null {
  const best = rankSearch(data, query, { playlist: 0, show: 0, episode: 0 }, 1)[0];
  return (best as TopResult | undefined) ?? null;
}
