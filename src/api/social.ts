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

export interface JamSession {
  session_id: string;
  join_token: string | null;
  join_url: string | null;
  is_host: boolean;
  members: { id: string; name: string; image_url: string | null; is_host: boolean; is_listening: boolean }[];
}

export const getJam = (): Promise<JamSession | null> => invoke("get_jam");
export const startJam = (): Promise<JamSession> => invoke("start_jam");
export const joinJam = (link: string): Promise<JamSession | null> => invoke("join_jam", { link });
export const leaveJam = (sessionId: string, isHost: boolean): Promise<void> =>
  invoke("leave_jam", { sessionId, isHost });
