import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { TrackItem } from "../types/spotify";

export type SpeedDialType = "liked-songs" | "playlist" | "album" | "artist" | "track";

export interface SpeedDialEntry {
  id: string;
  type: SpeedDialType;
  title: string;
  subtitle?: string;
  imageUrl?: string | null;
  to: string;
  track?: TrackItem;
  albumId?: string;
  playlistId?: string;
  artistId?: string;
  isLikedSongs?: boolean;
  playCount?: number;
  lastPlayedAt?: number;
  timestamp?: number;
}

interface SpeedDialStore {
  entries: SpeedDialEntry[];
  recordEntry: (entry: Omit<SpeedDialEntry, "timestamp" | "lastPlayedAt" | "playCount"> & { playCount?: number }) => void;
  recordTrack: (track: TrackItem, contextId?: string | null) => void;
  recordAlbum: (album: { id: string; name: string; image_url?: string | null }) => void;
  recordPlaylist: (playlist: { id: string; name: string; image_url?: string | null }) => void;
  recordArtist: (artist: { id: string; name: string; image_url?: string | null }) => void;
  recordLikedSongs: () => void;
  clear: () => void;
}

export const useSpeedDialStore = create<SpeedDialStore>()(
  persist(
    (set, get) => ({
      entries: [
        {
          id: "liked-songs",
          type: "liked-songs",
          title: "Liked Songs",
          to: "/library?tab=songs",
          isLikedSongs: true,
          playCount: 10,
          lastPlayedAt: Date.now(),
          timestamp: Date.now(),
        },
      ],

      recordEntry: (entry) => {
        set((s) => {
          const existing = s.entries.find((e) => e.id === entry.id);
          const playCount = (existing?.playCount || 0) + 1;
          const now = Date.now();
          const filtered = s.entries.filter((e) => e.id !== entry.id);
          const updated: SpeedDialEntry = {
            ...entry,
            playCount,
            lastPlayedAt: now,
            timestamp: now,
          };
          // Sort entries by MOST PLAYED (playCount desc, then recency)
          const all = [updated, ...filtered].sort((a, b) => {
            const pa = a.playCount ?? 1;
            const pb = b.playCount ?? 1;
            if (pb !== pa) return pb - pa;
            return (b.lastPlayedAt ?? b.timestamp ?? 0) - (a.lastPlayedAt ?? a.timestamp ?? 0);
          });
          return { entries: all.slice(0, 50) };
        });
      },

      recordTrack: (track, contextId) => {
        if (!track || !track.id) return;

        // If playing in Liked Songs context
        if (contextId === "liked-songs" || contextId === "liked") {
          get().recordLikedSongs();
          return;
        }

        // If contextId matches an artist
        if (contextId && contextId.startsWith("artist-top-")) {
          const artistId = contextId.replace("artist-top-", "");
          const artist = track.artists?.find((a) => a.id === artistId) || track.artists?.[0];
          if (artist) {
            get().recordArtist({
              id: artist.id,
              name: artist.name,
              image_url: track.album?.image_url,
            });
            return;
          }
        }

        // If playing in an album context, prefer recording the album (matches Spotify speed dial UX)
        if (track.album?.id && contextId === track.album.id) {
          get().recordAlbum({
            id: track.album.id,
            name: track.album.name,
            image_url: track.album.image_url,
          });
          return;
        }

        // Otherwise record the individual track
        get().recordEntry({
          id: `track-${track.id}`,
          type: "track",
          title: track.name,
          subtitle: track.artists?.map((a) => a.name).join(", "),
          imageUrl: track.album?.image_url ?? null,
          to: track.album?.id ? `/album/${track.album.id}` : "/library?tab=songs",
          track,
          albumId: track.album?.id,
        });
      },

      recordAlbum: (album) => {
        if (!album || !album.id) return;
        get().recordEntry({
          id: `album-${album.id}`,
          type: "album",
          title: album.name,
          imageUrl: album.image_url,
          to: `/album/${album.id}`,
          albumId: album.id,
        });
      },

      recordPlaylist: (playlist) => {
        if (!playlist || !playlist.id) return;
        get().recordEntry({
          id: `playlist-${playlist.id}`,
          type: "playlist",
          title: playlist.name,
          imageUrl: playlist.image_url,
          to: `/playlist/${playlist.id}`,
          playlistId: playlist.id,
        });
      },

      recordArtist: (artist) => {
        if (!artist || !artist.id) return;
        get().recordEntry({
          id: `artist-${artist.id}`,
          type: "artist",
          title: artist.name,
          imageUrl: artist.image_url,
          to: `/artist/${artist.id}`,
          artistId: artist.id,
        });
      },

      recordLikedSongs: () => {
        get().recordEntry({
          id: "liked-songs",
          type: "liked-songs",
          title: "Liked Songs",
          to: "/library?tab=songs",
          isLikedSongs: true,
        });
      },

      clear: () => set({ entries: [] }),
    }),
    {
      name: "musique-speed-dial-v2",
    },
  ),
);
