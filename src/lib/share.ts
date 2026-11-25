import { resolveOdesli } from "../api/share";
import { toast } from "../store/toast.store";
import { isApple } from "../hooks/useProvider";

export type ShareKind = "track" | "album" | "artist" | "playlist" | "user" | "show" | "episode";

const APPLE_KINDS: Partial<Record<ShareKind, string>> = {
  track: "song", album: "album", artist: "artist", playlist: "playlist",
};

/** Apple Music's public page for a catalog item. Library-only ids (`i.`, `l.`,
 *  `p.`, `r.`) have no public page, so they get none. */
export function appleMusicUrl(kind: ShareKind, id: string): string | null {
  const path = APPLE_KINDS[kind];
  if (!path || /^[ilpr]\./.test(id)) return null;
  return `https://music.apple.com/${path}/${id}`;
}

// canonical link for any entity on the signed-in service: open.spotify.com,
// or music.apple.com for an Apple Music account
export function spotifyUrl(kind: ShareKind, id: string): string {
  if (isApple()) return appleMusicUrl(kind, id) ?? "";
  return `https://open.spotify.com/${kind}/${id}`;
}

/** The menu label for copying the signed-in service's link. */
export const serviceLinkLabel = (): string => (isApple() ? "Copy Apple Music link" : "Copy Spotify link");

// copy text to clipboard, with an execCommand fallback for odd contexts
async function copy(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(ta);
      return ok;
    } catch {
      return false;
    }
  }
}

// copy the spotify link for an entity (instant, no network)
export async function shareSpotifyLink(kind: ShareKind, id: string) {
  const url = spotifyUrl(kind, id);
  if (!url) {
    toast.error("This is only in your library, so it has no public link");
    return;
  }
  const ok = await copy(url);
  if (ok) toast(isApple() ? "Apple Music link copied" : "Spotify link copied");
  else toast.error("Couldn't copy link");
}


export async function shareUniversalLink(kind: ShareKind, id: string) {
  const sp = spotifyUrl(kind, id);
  if (!sp) {
    toast.error("This is only in your library, so it has no public link");
    return;
  }
  let link = `https://song.link/${encodeURIComponent(sp)}`;
  try {
    const page = await resolveOdesli(sp);
    if (page) link = page;
  } catch {
    /* keep the fallback link */
  }
  const ok = await copy(link);
  if (ok) toast("Universal link copied");
  else toast.error("Couldn't copy link");
}
