import { describe, it, expect, afterEach } from "vitest";
import { coverUrl } from "./coverUrl";

const ALBUM_640 = "https://i.scdn.co/image/ab67616d0000b273a388a3f20d1bf2123249cc79";
const ALBUM_300 = "https://i.scdn.co/image/ab67616d00001e02a388a3f20d1bf2123249cc79";
const ALBUM_64 = "https://i.scdn.co/image/ab67616d00004851a388a3f20d1bf2123249cc79";
const ARTIST_640 = "https://i.scdn.co/image/ab6761610000e5eb98856eea770468af6dd999d9";
const ARTIST_320 = "https://i.scdn.co/image/ab6761610000517498856eea770468af6dd999d9";
const ARTIST_160 = "https://i.scdn.co/image/ab6761610000f17898856eea770468af6dd999d9";

const SIZES: Record<string, number> = {
  ab67616d00004851: 64,
  ab67616d00001e02: 300,
  ab67616d0000b273: 640,
  ab6761610000f178: 160,
  ab67616100005174: 320,
  ab6761610000e5eb: 640,
};

/* vitest runs in node here, with no DOM. coverUrl only ever reads
   window.devicePixelRatio, so a minimal stand-in is enough. */
function setDpr(v: number) {
  (globalThis as { window?: { devicePixelRatio: number } }).window = { devicePixelRatio: v };
}
afterEach(() => setDpr(1));

describe("coverUrl", () => {
  it("keeps the same image id, only changing the size prefix", () => {
    expect(coverUrl(ALBUM_640, 38)).toMatch(/a388a3f20d1bf2123249cc79$/);
  });

  it("drops a 38px thumbnail down from 640", () => {
    setDpr(1);
    expect(coverUrl(ALBUM_640, 38)).toBe(ALBUM_64);
    setDpr(2);
    expect(coverUrl(ALBUM_640, 38)).toBe(ALBUM_300);
  });

  it("keeps 640 for large art where nothing smaller is sufficient", () => {
    setDpr(2);
    expect(coverUrl(ALBUM_640, 520)).toBe(ALBUM_640);
    expect(coverUrl(ALBUM_640, 240)).toBe(ALBUM_640);
  });

  it("handles artist images on their own size ladder", () => {
    setDpr(1);
    expect(coverUrl(ARTIST_640, 26)).toBe(ARTIST_160);
    expect(coverUrl(ARTIST_640, 160)).toBe(ARTIST_320);
  });

  it("never upgrades past what is stored", () => {
    setDpr(2);
    expect(coverUrl(ALBUM_300, 400)).toBe(ALBUM_300);
    expect(coverUrl(ARTIST_160, 400)).toBe(ARTIST_160);
  });

  it("passes through anything it does not recognise", () => {
    const other = "https://example.com/pic.jpg";
    expect(coverUrl(other, 40)).toBe(other);
    expect(coverUrl(null, 40)).toBe(null);
    expect(coverUrl(undefined, 40)).toBe(undefined);
    const unknownPrefix = "https://i.scdn.co/image/0123456789abcdefdeadbeef";
    expect(coverUrl(unknownPrefix, 40)).toBe(unknownPrefix);
  });

  /* The guarantee that makes this safe to ship: for every display size and
     every device pixel ratio the app can realistically see, the variant we
     choose must still have at least as many pixels as the box needs. If this
     ever fails, images would render softer than before. */
  it("never selects a source smaller than the box needs", () => {
    for (const dpr of [1, 1.25, 1.5, 1.75, 2, 2.5, 3]) {
      setDpr(dpr);
      for (const box of [22, 26, 34, 38, 40, 44, 48, 108, 136, 160, 164, 200, 240, 520]) {
        for (const original of [ALBUM_640, ARTIST_640]) {
          const chosen = coverUrl(original, box)!;
          const prefix = chosen.slice(chosen.indexOf("/image/") + 7).slice(0, 16);
          expect(SIZES[prefix]).toBeGreaterThanOrEqual(Math.min(box * dpr, 640));
        }
      }
    }
  });
});
