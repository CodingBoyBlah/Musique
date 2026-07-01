import { resolveOdesli } from "../api/share";
import { toast } from "../store/toast.store";

export type ShareKind = "track" | "album" | "artist" | "playlist";

// canonical open.spotify.com URL for any entity
export function spotifyUrl(kind: ShareKind, id: string): string {
  return `https://open.spotify.com/${kind}/${id}`;
}

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
  const ok = await copy(spotifyUrl(kind, id));
  toast(ok ? "Spotify link copied" : "Couldn't copy link");
}


export async function shareUniversalLink(kind: ShareKind, id: string) {
  const sp = spotifyUrl(kind, id);
  let link = `https://song.link/${encodeURIComponent(sp)}`;
  try {
    const page = await resolveOdesli(sp);
    if (page) link = page;
  } catch {
    /* keep the fallback link */
  }
  const ok = await copy(link);
  toast(ok ? "Universal link copied" : "Couldn't copy link");
}
