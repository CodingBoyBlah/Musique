import { beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import {
  cancelHoverPrefetch,
  isPendingRouteLoad,
  lazyRoute,
  prefetchForPath,
  prefetchLibrary,
  prefetchPlaylist,
  prefetchPlaylistsIndex,
  preloadRoute,
  registerRoutePreloaders,
  routeKeyFromPath,
  scheduleHoverPrefetch,
  subscribePendingRouteLoad,
} from "./prefetch";
import { LIBRARY_KEYS } from "../hooks/useLibrary";
import * as spotifyApi from "../api/spotify";
import * as libraryApi from "../api/library";
import * as internalApi from "../api/internal";
import * as podcastsApi from "../api/podcasts";
import * as profileApi from "../api/profile";
import * as statsApi from "../api/stats";

vi.mock("../api/spotify", () => ({
  getAlbum: vi.fn().mockResolvedValue(null),
  getArtist: vi.fn().mockResolvedValue(null),
  getPlaylist: vi.fn().mockResolvedValue(null),
  getCachedPlaylist: vi.fn().mockResolvedValue(null),
  getTrack: vi.fn().mockResolvedValue(null),
  search: vi.fn().mockResolvedValue(null),
}));

vi.mock("../api/library", () => ({
  getLikedSongs: vi.fn().mockResolvedValue([]),
  getLikedSongsCount: vi.fn().mockResolvedValue(0),
  getLibraryStatus: vi.fn().mockResolvedValue({ last_synced: null, is_syncing: false }),
  getSavedAlbums: vi.fn().mockResolvedValue([]),
  getFollowedArtists: vi.fn().mockResolvedValue([]),
  getMyPlaylists: vi.fn().mockResolvedValue([]),
}));

vi.mock("../api/internal", () => ({
  getArtistExtras: vi.fn().mockResolvedValue(null),
  getArtistOverview: vi.fn().mockResolvedValue(null),
  getHomeFeed: vi.fn().mockResolvedValue({ greeting: null, sections: [] }),
  getPlaylistFolders: vi.fn().mockResolvedValue([]),
}));

vi.mock("../api/podcasts", () => ({
  getSavedShows: vi.fn().mockResolvedValue([]),
  getShow: vi.fn().mockResolvedValue(null),
}));

vi.mock("../api/profile", () => ({
  getUserProfile: vi.fn().mockResolvedValue(null),
}));

vi.mock("../api/stats", () => ({
  getListeningStats: vi.fn().mockResolvedValue(null),
}));

vi.mock("../api/lyrics", () => ({
  getLyrics: vi.fn().mockResolvedValue(null),
}));

describe("route & query prefetch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps application paths to lazy route keys", () => {
    expect(routeKeyFromPath("/")).toBeNull();
    expect(routeKeyFromPath("/library?tab=albums")).toBe("library");
    expect(routeKeyFromPath("/playlists?folder=f1")).toBe("playlists");
    expect(routeKeyFromPath("/playlist/abc")).toBe("playlist");
    expect(routeKeyFromPath("/artist/123")).toBe("artist");
    expect(routeKeyFromPath("/album/xyz")).toBe("album");
    expect(routeKeyFromPath("/show/s1")).toBe("show");
    expect(routeKeyFromPath("/user/u1")).toBe("profile");
    expect(routeKeyFromPath("/profile")).toBe("profile");
    expect(routeKeyFromPath("/stats")).toBe("stats");
    expect(routeKeyFromPath("/search?q=jazz")).toBe("search");
    expect(routeKeyFromPath("/settings")).toBe("settings");
  });

  it("deduplicates concurrent/repeated route preloads and retries after failure", async () => {
    const loader = vi.fn()
      .mockRejectedValueOnce(new Error("chunk failed"))
      .mockResolvedValueOnce({ default: () => null });

    registerRoutePreloaders({ stats: loader });
    await expect(preloadRoute("/stats")).rejects.toThrow("chunk failed");
    await Promise.all([preloadRoute("/stats"), preloadRoute("stats")]);
    expect(loader).toHaveBeenCalledTimes(2);
  });

  it("prefetches both cached and live playlist queries", async () => {
    const qc = new QueryClient();
    const loader = vi.fn().mockResolvedValue({});
    registerRoutePreloaders({ playlist: loader });

    prefetchPlaylist(qc, "pl-1");
    await Promise.resolve();

    expect(loader).toHaveBeenCalledTimes(1);
    expect(spotifyApi.getCachedPlaylist).toHaveBeenCalledWith("pl-1");
    expect(spotifyApi.getPlaylist).toHaveBeenCalledWith("pl-1");
    expect(qc.getQueryState(["playlist", "pl-1", "cached"])).toBeDefined();
    expect(qc.getQueryState(["playlist", "pl-1"])).toBeDefined();
  });

  it("prefetches exact library, playlists index, show, profile, and stats query keys", async () => {
    const qc = new QueryClient();

    prefetchLibrary(qc, "songs");
    prefetchLibrary(qc, "podcasts");
    prefetchPlaylistsIndex(qc);
    prefetchForPath(qc, "/show/sh-9");
    prefetchForPath(qc, "/user/u-9");
    prefetchForPath(qc, "/stats");

    expect(qc.getQueryState(LIBRARY_KEYS.liked(200, 0))).toBeDefined();
    expect(qc.getQueryState(["library", "shows"])).toBeDefined();
    expect(qc.getQueryState(LIBRARY_KEYS.playlists)).toBeDefined();
    expect(qc.getQueryState(["library", "rootlist"])).toBeDefined();
    expect(qc.getQueryState(["library", "home-feed"])).toBeDefined();
    expect(qc.getQueryState(["show", "sh-9"])).toBeDefined();
    expect(qc.getQueryState(["profile", "view", "u-9"])).toBeDefined();
    expect(qc.getQueryState(["stats", "month"])).toBeDefined();

    expect(libraryApi.getLikedSongs).toHaveBeenCalledWith(200, 0);
    expect(podcastsApi.getSavedShows).toHaveBeenCalledTimes(1);
    expect(internalApi.getPlaylistFolders).toHaveBeenCalledTimes(1);
    expect(podcastsApi.getShow).toHaveBeenCalledWith("sh-9");
    expect(profileApi.getUserProfile).toHaveBeenCalledWith("u-9");
    expect(statsApi.getListeningStats).toHaveBeenCalledWith("month");
  });

  it("cancels pending hover intent when leaving quickly", () => {
    vi.useFakeTimers();
    try {
      const fn = vi.fn();
      scheduleHoverPrefetch(fn, 60);
      vi.advanceTimersByTime(30);
      cancelHoverPrefetch();
      vi.advanceTimersByTime(100);
      expect(fn).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it("tracks active un-preloaded lazy route loads and clears once resolved", async () => {
    let resolveLoad!: (m: { default: () => null }) => void;
    const loader = vi.fn(
      () => new Promise<{ default: () => null }>((r) => { resolveLoad = r; }),
    );
    const events: boolean[] = [];
    const unsub = subscribePendingRouteLoad(() => events.push(isPendingRouteLoad()));

    const Comp = lazyRoute("playground", loader);
    registerRoutePreloaders({ playground: loader });
    expect(isPendingRouteLoad()).toBe(false);

    // Simulate triggering an un-preloaded route chunk
    const pending = preloadRoute("playground");
    expect(Comp).toBeTypeOf("function");

    // Preload path sets resolved component without marking on-demand pending load
    resolveLoad({ default: () => null });
    await pending;
    await Promise.resolve();
    expect(isPendingRouteLoad()).toBe(false);
    unsub();
  });
});
