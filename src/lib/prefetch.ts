import type { QueryClient } from "@tanstack/react-query";
import { getAlbum, getArtist, getPlaylist } from "../api/spotify";
import { getLyrics } from "../api/lyrics";
import type { TrackItem } from "../types/spotify";


export function prefetchAlbum(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  qc.prefetchQuery({
    queryKey:  ["album", id],
    queryFn:   () => getAlbum(id),
    staleTime: 300_000,
  });
}

export function prefetchArtist(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  qc.prefetchQuery({
    queryKey:  ["artist", id],
    queryFn:   () => getArtist(id),
    staleTime: 300_000,
  });
}

export function prefetchPlaylist(qc: QueryClient, id: string | undefined | null) {
  if (!id) return;
  qc.prefetchQuery({
    queryKey:  ["playlist", id],
    queryFn:   () => getPlaylist(id),
    staleTime: 120_000,
  });
}

// lyrics are cached hard (sqlite-backed), so warm them and the panel never spins
export function prefetchLyrics(qc: QueryClient, track: TrackItem | null | undefined) {
  if (!track) return;
  qc.prefetchQuery({
    queryKey:  ["lyrics", track.id],
    queryFn:   () => getLyrics(track),
    staleTime: Infinity,
    gcTime:    60 * 60_000,
  });
}
