import { invoke } from "@tauri-apps/api/core";

export interface FriendActivity {
  user_id: string;
  name: string;
  image_url: string | null;
  timestamp: number;
  track_id: string | null;
  track_name: string | null;
  track_image: string | null;
  artist_id: string | null;
  artist_name: string | null;
  album_id: string | null;
  album_name: string | null;
  context_uri: string | null;
  context_name: string | null;
}

// what the people you follow are playing (spclient buddylist)
export const getFriendActivity = (): Promise<FriendActivity[]> => invoke("get_friend_activity");
