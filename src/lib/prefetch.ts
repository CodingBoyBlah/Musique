import type { QueryClient } from "@tanstack/react-query";
import { getAlbum, getArtist, getCachedPlaylist, getPlaylist, getTrack, search } from "../api/spotify";
import {
  getFollowedArtists,
  getLibraryStatus,
  getLikedSongs,
  getLikedSongsCount,
  getMyPlaylists,
  getSavedAlbums,
} from "../api/library";
import { getArtistExtras, getArtistOverview, getHomeFeed, getPlaylistFolders } from "../api/internal";
import { getSavedShows, getShow } from "../api/podcasts";
import { getUserProfile } from "../api/profile";
import { getListeningStats, type StatsRange } from "../api/stats";
import { getLyrics } from "../api/lyrics";
import { LIBRARY_KEYS } from "../hooks/useLibrary";
import { isEpisodeId } from "../utils/episode";
import type { TrackItem } from "../types/spotify";

export type RouteKey =
  | "settings"
  | "search"
  | "library"
  | "playlists"
  | "playlist"
  | "artist"
  | "album"
  | "playground"
  | "profile"
  | "show"
  | "stats";

type RouteLoader = () => Promise<unknown>;

const routeLoaders: Partial<Record<RouteKey, RouteLoader>> = {};
const routePromises = new Map<RouteKey, Promise<unknown>>();
let idlePreloadStarted = false;

const IDLE_PRELOAD_ORDER: RouteKey[] = [
  "library",
  "playlists",
  "playlist",
  "search",
  "artist",
  "album",
  "show",
  "stats",
  "profile",
  "settings",
];

// One pending hover timer across link surfaces so fast cursor sweeps over
// dense track/card grids cancel previous targets without per-row hook overhead.
let hoverTimer: ReturnType<typeof setTimeout> | null = null;

export function scheduleHoverPrefetch(fn: () => void, delayMs = 60) {
  if (hoverTimer !== null) clearTimeout(hoverTimer);
  hoverTimer = setTimeout(() => {
    hoverTimer = null;
    fn();
  }, delayMs);
}

export function cancelHoverPrefetch() {
  if (hoverTimer !== null) {
    clearTimeout(hoverTimer);
    hoverTimer = null;
  }
}

export function routeKeyFromPath(path: string): RouteKey | null {
  const clean = path.split("?")[0].split("#")[0];
  if (clean === "/settings") return "settings";
  if (clean === "/search") return "search";
  if (clean === "/library") return "library";
  if (clean === "/playlists") return "playlists";
  if (clean === "/stats") return "stats";
  if (clean === "/profile" || clean.startsWith("/user/")) return "profile";
  if (clean === "/playground") return "playground";
  if (clean.startsWith("/playlist/")) return "playlist";
  if (clean.startsWith("/artist/")) return "artist";
  if (clean.startsWith("/album/")) return "album";
  if (clean.startsWith("/show/")) return "show";
  return null;
}

export function preloadRoute(keyOrPath: RouteKey | string): Promise<unknown> | undefined {
  const key = (keyOrPath.startsWith("/") ? routeKeyFromPath(keyOrPath) : (keyOrPath as RouteKey));
  if (!key) return undefined;
  const cached = routePromises.get(key);
  if (cached) return cached;
  const loader = routeLoaders[key];
  if (!loader) return undefined;
  const promise = loader().catch((err) => {
    routePromises.delete(key);
    throw err;
  });
  routePromises.set(key, promise);
  return promise;
}

function scheduleIdlePreloads() {
  if (idlePreloadStarted || typeof window === "undefined") return;
  idlePreloadStarted = true;

  let index = 0;
  const runNext = () => {
    while (index < IDLE_PRELOAD_ORDER.length) {
      const key = IDLE_PRELOAD_ORDER[index++];
      if (routeLoaders[key] && !routePromises.has(key)) {
        preloadRoute(key)?.catch(() => {});
        break;
      }
    }
    if (index < IDLE_PRELOAD_ORDER.length) scheduleStep();
  };

  const scheduleStep = () => {
    const ric = (window as unknown as { requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => number }).requestIdleCallback;
    if (typeof ric === "function") {
      ric(runNext, { timeout: 2500 });
    } else {
      window.setTimeout(runNext, 120);
    }
  };

  // Defer until after Home renders and paints its first frames.
  if (typeof requestAnimationFrame === "function") {
    requestAnimationFrame(() => requestAnimationFrame(scheduleStep));
  } else {
    window.setTimeout(scheduleStep, 300);
  }
}

export function registerRoutePreloaders(loaders: Partial<Record<RouteKey, RouteLoader>>) {
  Object.assign(routeLoaders, loaders);
  scheduleIdlePreloads();
}

export function prefetchAlbum(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  preloadRoute("album");
  qc.prefetchQuery({
    queryKey:  ["album", id],
    queryFn:   () => getAlbum(id),
    staleTime: 300_000,
  });
}

export function prefetchArtist(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  preloadRoute("artist");
  qc.prefetchQuery({
    queryKey:  ["artist", id],
    queryFn:   () => getArtist(id),
    staleTime: 300_000,
  });
}

export function prefetchArtistDetails(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  prefetchArtist(qc, id);
  qc.prefetchQuery({
    queryKey:  ["artist-extras", id],
    queryFn:   () => getArtistExtras(id),
    staleTime: 30 * 60_000,
  });
  qc.prefetchQuery({
    queryKey:  ["artist-overview", id],
    queryFn:   () => getArtistOverview(id),
    staleTime: 30 * 60_000,
  });
}

export function prefetchPlaylist(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  preloadRoute("playlist");
  qc.prefetchQuery({
    queryKey:  ["playlist", id, "cached"],
    queryFn:   () => getCachedPlaylist(id),
    staleTime: Infinity,
  });
  qc.prefetchQuery({
    queryKey:  ["playlist", id],
    queryFn:   () => getPlaylist(id),
    staleTime: 120_000,
  });
}

export function prefetchShow(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  preloadRoute("show");
  qc.prefetchQuery({
    queryKey:  ["show", id],
    queryFn:   () => getShow(id),
    staleTime: 120_000,
  });
}

export function prefetchProfile(qc: QueryClient, id: string | undefined | null) {
  preloadRoute("profile");
  const keyId = id ?? "me";
  qc.prefetchQuery({
    queryKey:  ["profile", "view", keyId],
    queryFn:   () => getUserProfile(id ?? null),
    staleTime: 300_000,
  });
}

export function prefetchStats(qc: QueryClient, range: StatsRange = "month") {
  preloadRoute("stats");
  qc.prefetchQuery({
    queryKey:  ["stats", range],
    queryFn:   () => getListeningStats(range),
    staleTime: 60_000,
  });
}

export function prefetchLibrary(qc: QueryClient, tab = "songs") {
  preloadRoute("library");
  if (tab === "albums") {
    qc.prefetchQuery({ queryKey: LIBRARY_KEYS.albums, queryFn: getSavedAlbums, staleTime: 5 * 60_000 });
  } else if (tab === "artists") {
    qc.prefetchQuery({ queryKey: LIBRARY_KEYS.artists, queryFn: getFollowedArtists, staleTime: 5 * 60_000 });
  } else if (tab === "podcasts") {
    qc.prefetchQuery({ queryKey: ["library", "shows"], queryFn: getSavedShows, staleTime: 300_000 });
  } else {
    qc.prefetchQuery({ queryKey: LIBRARY_KEYS.liked(200, 0), queryFn: () => getLikedSongs(200, 0), staleTime: 5 * 60_000 });
    qc.prefetchQuery({ queryKey: LIBRARY_KEYS.likedCount, queryFn: getLikedSongsCount, staleTime: 5 * 60_000 });
    qc.prefetchQuery({ queryKey: LIBRARY_KEYS.status, queryFn: getLibraryStatus, staleTime: 5 * 60_000 });
  }
}

export function prefetchPlaylistsIndex(qc: QueryClient) {
  preloadRoute("playlists");
  qc.prefetchQuery({ queryKey: LIBRARY_KEYS.playlists, queryFn: getMyPlaylists, staleTime: 5 * 60_000 });
  qc.prefetchQuery({ queryKey: ["library", "rootlist"], queryFn: getPlaylistFolders, staleTime: 5 * 60_000 });
  const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  qc.prefetchQuery({ queryKey: ["library", "home-feed"], queryFn: () => getHomeFeed(timeZone), staleTime: 30 * 60_000 });
}

export function prefetchSearch(qc: QueryClient, query?: string) {
  preloadRoute("search");
  const trimmed = query?.trim();
  if (trimmed) {
    qc.prefetchQuery({
      queryKey:  ["search", query!],
      queryFn:   () => search(query!),
      staleTime: 300_000,
    });
  }
}

export function prefetchTrack(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  qc.prefetchQuery({
    queryKey:  ["track", id],
    queryFn:   () => getTrack(id),
    staleTime: 300_000,
  });
}

export function prefetchForPath(qc: QueryClient, to: string | undefined | null) {
  if (!to || to === "/") return;
  const [clean, searchStr = ""] = to.split("?");
  if (clean.startsWith("/playlist/")) {
    prefetchPlaylist(qc, decodeURIComponent(clean.slice("/playlist/".length)));
  } else if (clean.startsWith("/album/")) {
    prefetchAlbum(qc, decodeURIComponent(clean.slice("/album/".length)));
  } else if (clean.startsWith("/artist/")) {
    prefetchArtist(qc, decodeURIComponent(clean.slice("/artist/".length)));
  } else if (clean.startsWith("/show/")) {
    prefetchShow(qc, decodeURIComponent(clean.slice("/show/".length)));
  } else if (clean.startsWith("/user/")) {
    prefetchProfile(qc, decodeURIComponent(clean.slice("/user/".length)));
  } else if (clean === "/profile") {
    prefetchProfile(qc, null);
  } else if (clean === "/stats") {
    prefetchStats(qc);
  } else if (clean === "/playlists") {
    prefetchPlaylistsIndex(qc);
  } else if (clean === "/library") {
    const tab = new URLSearchParams(searchStr).get("tab") ?? "songs";
    prefetchLibrary(qc, tab);
  } else if (clean === "/search") {
    const q = new URLSearchParams(searchStr).get("q") ?? undefined;
    prefetchSearch(qc, q);
  } else {
    preloadRoute(clean);
  }
}

// lyrics are cached hard (sqlite-backed), so warm them and the panel never spins
export function prefetchLyrics(qc: QueryClient, track: TrackItem | null | undefined) {
  if (!track || isEpisodeId(track.id)) return;
  qc.prefetchQuery({
    queryKey:  ["lyrics", track.id],
    queryFn:   () => getLyrics(track),
    staleTime: Infinity,
    gcTime:    60 * 60_000,
  });
}
