import { invoke } from "@tauri-apps/api/core";
import type { ArtistItem, PlaylistCard } from "../types/spotify";

export interface UserProfile {
  id: string;
  name: string;
  image_url: string | null;
  followers: number | null;
  following: number | null;
  total_public_playlists: number | null;
  playlists: PlaylistCard[];
  recently_played_artists: ArtistItem[];
  is_verified: boolean;
  spotify_url: string;
}

export interface ProfileCard {
  id: string;
  kind: "user" | "artist";
  name: string;
  image_url: string | null;
  followers: number | null;
}

// omit the id for your own profile
export const getUserProfile = (id?: string | null): Promise<UserProfile> =>
  invoke("get_user_profile", { id: id ?? null });

export const getUserFollowers = (id?: string | null): Promise<ProfileCard[]> =>
  invoke("get_user_followers", { id: id ?? null });

export const getUserFollowing = (id?: string | null): Promise<ProfileCard[]> =>
  invoke("get_user_following", { id: id ?? null });

export const setUserFollowed = (id: string, follow: boolean): Promise<void> =>
  invoke("set_user_followed", { id, follow });

export const isUserFollowed = (id: string): Promise<boolean> =>
  invoke("is_user_followed", { id });
