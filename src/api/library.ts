import { invoke } from "@tauri-apps/api/core";
import type { TrackItem, ArtistItem, AlbumItem } from "../types/spotify";
import type { LibraryStatus, PlaylistSummary, SyncResult, TimeRange } from "../types/library";

export const syncLibrary = (): Promise<SyncResult> =>
  invoke("sync_library");

export const getLikedSongs = (limit: number, offset: number): Promise<TrackItem[]> =>
  invoke("get_liked_songs", { limit, offset });

export const getLikedSongsCount = (): Promise<number> =>
  invoke("get_liked_songs_count");

export const getMyPlaylists = (): Promise<PlaylistSummary[]> =>
  invoke("get_my_playlists");

export const getSavedAlbums = (): Promise<AlbumItem[]> =>
  invoke("get_saved_albums");

export const getFollowedArtists = (): Promise<ArtistItem[]> =>
  invoke("get_followed_artists");

export const getLibraryStatus = (): Promise<LibraryStatus> =>
  invoke("get_library_status");

export const getTopTracks = (timeRange: TimeRange = "medium_term"): Promise<TrackItem[]> =>
  invoke("get_top_tracks", { timeRange });

export const getTopArtists = (timeRange: TimeRange = "medium_term"): Promise<ArtistItem[]> =>
  invoke("get_top_artists", { timeRange });

export const getRecentlyPlayed = (): Promise<TrackItem[]> =>
  invoke("get_recently_played");

export const getNewReleases = (): Promise<AlbumItem[]> =>
  invoke("get_new_releases");

export const saveTrack = (id: string): Promise<void> =>
  invoke("save_track", { id });

export const unsaveTrack = (id: string): Promise<void> =>
  invoke("unsave_track", { id });

export const getSavedTrackIds = (ids: string[]): Promise<string[]> =>
  invoke("get_saved_track_ids", { ids });

export const followArtist = (id: string): Promise<void> =>
  invoke("follow_artist", { id });

export const unfollowArtist = (id: string): Promise<void> =>
  invoke("unfollow_artist", { id });

export const saveAlbum = (id: string): Promise<void> =>
  invoke("save_album", { id });

export const unsaveAlbum = (id: string): Promise<void> =>
  invoke("unsave_album", { id });

export const isAlbumSaved = (id: string): Promise<boolean> =>
  invoke("is_album_saved", { id });

export const isArtistFollowed = (id: string): Promise<boolean> =>
  invoke("is_artist_followed", { id });

// position: zero-based insert index; omitted appends
export const addTrackToPlaylist = (playlistId: string, trackId: string, position?: number): Promise<void> =>
  invoke("add_track_to_playlist", { playlistId, trackId, position: position ?? null });

export const removeTrackFromPlaylist = (playlistId: string, trackId: string): Promise<void> =>
  invoke("remove_track_from_playlist", { playlistId, trackId });

export const createPlaylist = (name: string, description: string | null, isPublic: boolean): Promise<string> =>
  invoke("create_playlist", { name, description, public: isPublic });

export const updatePlaylistDetails = (
  id: string,
  changes: { name?: string; description?: string; public?: boolean; collaborative?: boolean },
): Promise<void> =>
  invoke("update_playlist_details", {
    id,
    name:          changes.name ?? null,
    description:   changes.description ?? null,
    public:        changes.public ?? null,
    collaborative: changes.collaborative ?? null,
  });

export const followPlaylist = (id: string): Promise<void> =>
  invoke("follow_playlist", { id });

// on a playlist you own this deletes it (spotify has no separate delete)
export const unfollowPlaylist = (id: string): Promise<void> =>
  invoke("unfollow_playlist", { id });

export const isPlaylistFollowed = (id: string): Promise<boolean> =>
  invoke("is_playlist_followed", { id });
