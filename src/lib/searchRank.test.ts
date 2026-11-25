import { describe, expect, it } from "vitest";
import { rankSearch, topResult, nameScore, normalize } from "./searchRank";
import type { AlbumItem, ArtistItem, SearchResults, TrackItem } from "../types/spotify";

const artist = (id: string, name: string, popularity?: number): ArtistItem => ({ id, name, image_url: null, popularity });
const album = (id: string, name: string, by: ArtistItem): AlbumItem =>
  ({ id, name, album_type: "album", image_url: null, release_date: null, artists: [by] });
const track = (id: string, name: string, by: ArtistItem, popularity?: number): TrackItem =>
  ({ id, name, duration_ms: 1, explicit: false, artists: [by], album: null, popularity });

const radiohead = artist("rh", "Radiohead", 80);
const data = (over: Partial<SearchResults> = {}): SearchResults => ({
  tracks: [track("t1", "Paranoid Android", radiohead), track("t2", "Fake Plastic Trees", radiohead), track("t3", "Creep", radiohead, 85)],
  albums: [album("a1", "OK Computer", radiohead), album("a2", "The Bends", radiohead)],
  artists: [radiohead, artist("x", "Radiohead Tribute Band", 5)],
  playlists: [{ id: "p1", name: "This Is Radiohead", description: null, image_url: null, owner_name: "Spotify" }],
  ...over,
});

describe("searchRank", () => {
  it("normalizes case, accents and punctuation", () => {
    expect(normalize("  Beyoncé — HALO! ")).toBe("beyonce halo");
  });

  it("scores exact over prefix over word over substring", () => {
    expect(nameScore("Radiohead", "radiohead")).toBe(100);
    expect(nameScore("Creep - Remastered", "creep")).toBeGreaterThan(nameScore("Creeping Death", "creep"));
    expect(nameScore("Creeping Death", "creep")).toBeGreaterThan(nameScore("Slow Creep", "creep"));
    expect(nameScore("Slow Creep", "creep")).toBeGreaterThan(nameScore("Supercreep", "creep"));
    expect(nameScore("Nothing", "creep")).toBe(0);
  });

  it("puts the artist first when the query is the artist's name", () => {
    const r = rankSearch(data(), "radiohead");
    expect(r[0]).toMatchObject({ type: "artist", item: { id: "rh" } });
    // songs and albums found through the artist still come before the playlist
    const types = r.map((x) => x.type);
    expect(types.indexOf("track")).toBeLessThan(types.indexOf("playlist"));
  });

  it("puts the song first when the query is a song title", () => {
    expect(rankSearch(data(), "creep")[0]).toMatchObject({ type: "track", item: { id: "t3" } });
    expect(topResult(data(), "paranoid android")).toMatchObject({ type: "track", item: { id: "t1" } });
  });

  it("puts the album first when the query is an album title", () => {
    expect(topResult(data(), "ok computer")).toMatchObject({ type: "album", item: { id: "a1" } });
  });

  it("interleaves types instead of grouping them", () => {
    const types = rankSearch(data(), "radiohead").map((x) => x.type);
    const firstAlbum = types.indexOf("album");
    const lastTrack = types.lastIndexOf("track");
    expect(firstAlbum).toBeLessThan(lastTrack);
  });

  it("respects per-type caps and the total limit", () => {
    const r = rankSearch(data(), "radiohead", { track: 1, playlist: 0 }, 4);
    expect(r).toHaveLength(4);
    expect(r.filter((x) => x.type === "track")).toHaveLength(1);
    expect(r.some((x) => x.type === "playlist")).toBe(false);
  });

  it("topResult ignores playlists and podcasts", () => {
    const d = data({ artists: [], tracks: [], albums: [] });
    expect(topResult(d, "this is radiohead")).toBeNull();
  });
});
