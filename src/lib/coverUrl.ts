/* Pick the right-sized Spotify CDN image instead of always decoding the 640px one.

   The sync layer stores `images.first()` for every album, artist and playlist,
   and Spotify returns its image list largest-first - so essentially every cover
   in the database is the 640x640 variant. Checked against the live cache:
   5,860 of 5,861 albums and 760 artists store a 640px URL.

   A JPEG costs width * height * 4 bytes once the renderer decodes it, whatever
   size it is displayed at. So a 640x640 cover is ~1.56 MB of resident bitmap -
   and TrackRow draws it into a 38x38 box, Sidebar into 26x26. A 200-row liked
   songs list was holding on the order of 300 MB of decoded bitmaps to show
   thumbnails the size of a fingernail. That is what WebView2's memory was.

   Spotify serves the same image at several sizes, distinguished by a prefix on
   the image id. Verified live against 20 URLs pulled from this app's own cache,
   every one returning exactly the expected dimensions:

     album art   ab67616d00004851 =  64    ab67616d00001e02 = 300    ab67616d0000b273 = 640
     artist art  ab6761610000f178 = 160    ab67616100005174 = 320    ab6761610000e5eb = 640

   `coverUrl` swaps in the smallest variant that still has at least as many
   pixels as the display actually needs, so nothing is ever rendered from a
   source smaller than its box - the picture is identical, only the wasted
   decode is gone. Anything it does not recognise (playlist art, user avatars,
   other hosts) is returned untouched. */

type Family = ReadonlyArray<readonly [prefix: string, px: number]>;

// ascending by size; the last entry is what the sync layer stores
const FAMILIES: ReadonlyArray<Family> = [
  [["ab67616d00004851", 64], ["ab67616d00001e02", 300], ["ab67616d0000b273", 640]],
  [["ab6761610000f178", 160], ["ab67616100005174", 320], ["ab6761610000e5eb", 640]],
];

const MARKER = "/image/";
const ID_PREFIX_LEN = 16;

/* Headroom over the measured box: covers hover scales (ArtistCard grows 5% on
   hover), sub-pixel layout, and call sites whose size hint is a round number
   rather than the exact rendered width. Cheap insurance - the savings come from
   thumbnails, where even with the margin we still drop several tiers. */
const SAFETY = 1.15;

export function coverUrl(url: string | null | undefined, displayPx: number): string | null | undefined {
  if (!url || displayPx <= 0) return url;

  const at = url.indexOf(MARKER);
  if (at < 0) return url;

  const id = url.slice(at + MARKER.length);
  if (id.length <= ID_PREFIX_LEN) return url;

  const prefix = id.slice(0, ID_PREFIX_LEN);
  const family = FAMILIES.find((f) => f.some(([p]) => p === prefix));
  if (!family) return url;

  // Never upgrade: if this URL is already a smaller variant, that is the
  // ceiling. Only ever trade down from what is stored.
  const stored = family.find(([p]) => p === prefix)![1];

  const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
  const needed = displayPx * dpr * SAFETY;

  for (const [p, px] of family) {
    if (px >= needed && px <= stored) {
      return p === prefix ? url : url.slice(0, at + MARKER.length) + p + id.slice(ID_PREFIX_LEN);
    }
  }
  return url;
}
