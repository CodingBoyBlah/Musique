import { invoke } from "@tauri-apps/api/core";

/* Blend support. Spotify Blends are playlist4 lists whose `format` attribute is
"blend"; each item carries the member it was picked for. Rust reads that off
spclient playlist/v2 (`get_blend_info`). */

export interface BlendMember {
  username: string;
  name: string;
  imageUrl: string | null;
}

export interface BlendInfo {
  isBlend: boolean;
  /** raw playlist4 format, e.g. "blend" */
  format: string | null;
  members: BlendMember[];
  /** 0..100 taste match when spotify ships it */
  tasteMatch: number | null;
  /** track id -> usernames the track was picked for */
  contributors: Record<string, string[]>;
}

export const getBlendInfo = (id: string): Promise<BlendInfo | null> => invoke("get_blend_info", { id });

/** cheap heuristic for lists we have not opened yet. Blends are owned by
spotify, named "A + B" (or "A, B + 2 more") and described "A blend of music for ..." */
export const looksLikeBlend = (name?: string | null, description?: string | null, ownerId?: string | null) => {
  if (description && /\bblend of\b/i.test(description)) return true;
  if (name && /^blend\b|\bblend$/i.test(name)) return true;
  // owner unknown (library rows): only trust the name when there is no description
  const spotifyOwned = ownerId ? ownerId.toLowerCase() === "spotify" : !description;
  return spotifyOwned && !!name && /^[^+]{1,40} \+ [^+]{1,40}$/.test(name);
};
