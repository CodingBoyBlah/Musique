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

export interface JamMember {
  id: string;
  name: string;
  image_url: string | null;
  is_host: boolean;
  is_listening: boolean;
  is_current_user: boolean;
}

export interface JamSession {
  session_id: string;
  join_token: string | null;
  join_url: string | null;
  is_host: boolean;
  members: JamMember[];
  // guests can only add songs; playing and skipping stays with the host
  queue_only_mode: boolean;
  // a guest's commands go to the jam (jamCommand), not their own player
  redirect_commands: boolean;
  host_device_id: string | null;
  host_device_name: string | null;
  is_paused: boolean;
}

// what a social-connect push says happened
export type JamUpdateReason =
  | "UNKNOWN_UPDATE_TYPE" | "NEW_SESSION" | "USER_JOINED" | "USER_LEFT" | "SESSION_DELETED"
  | "YOU_LEFT" | "YOU_WERE_KICKED" | "YOU_JOINED" | "PARTICIPANT_PROMOTED_TO_HOST"
  | "DISCOVERABILITY_CHANGED" | "USER_KICKED" | "QUEUE_ONLY_MODE_CONTROL_CHANGED"
  | "ACTIVE_DEVICE_CHANGED" | "SESSION_MEMBER_UPDATED" | "SESSION_ACTIVATED" | (string & {});

export interface JamUpdate {
  reason: JamUpdateReason;
  session: JamSession | null;
}

export const getJam = (): Promise<JamSession | null> => invoke("get_jam");
export const startJam = (): Promise<JamSession> => invoke("start_jam");
export const joinJam = (link: string): Promise<JamSession | null> => invoke("join_jam", { link });
export const leaveJam = (sessionId: string, isHost: boolean): Promise<void> =>
  invoke("leave_jam", { sessionId, isHost });
export const setJamQueueOnly = (enabled: boolean): Promise<JamSession | null> =>
  invoke("set_jam_queue_only", { enabled });
export const kickJamMember = (sessionId: string, memberId: string): Promise<void> =>
  invoke("kick_jam_member", { sessionId, memberId });

// a guest's player command, relayed through the jam to everyone in it. the
// body is connect's own command json: { endpoint: "skip_next" }, ...
export type JamCommand =
  | { endpoint: "skip_next"; track?: { uri: string } }
  | { endpoint: "skip_prev" }
  | { endpoint: "add_to_queue"; track: { uri: string } }
  | { endpoint: "seek_to"; value: number }
  | { endpoint: "play"; context: { uri?: string; url?: string; pages?: { tracks: { uri: string }[] }[] }; options?: { skip_to?: { track_index?: number; track_uri?: string } } };

export const jamCommand = (sessionId: string, command: JamCommand): Promise<void> =>
  invoke("jam_command", { sessionId, command });
