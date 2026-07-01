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

export const isArtistFollowed = (id: string): Promise<boolean> =>
  invoke("is_artist_followed", { id });

export const addTrackToPlaylist = (playlistId: string, trackId: string): Promise<void> =>
  invoke("add_track_to_playlist", { playlistId, trackId });

export const removeTrackFromPlaylist = (playlistId: string, trackId: string): Promise<void> =>
  invoke("remove_track_from_playlist", { playlistId, trackId });

export const createPlaylist = (name: string, description: string | null, isPublic: boolean): Promise<string> =>
  invoke("create_playlist", { name, description, public: isPublic });
