export type { ArtistItem, AlbumItem, TrackItem } from "./spotify";

export interface AppError {
  kind: "Database" | "Keyring" | "Network" | "Auth" | "NotFound" | "InvalidInput";
  message: string;
}

export interface AuthStatus {
  logged_in:    boolean;
  user_id:      string | null;
  display_name: string | null;
  email:        string | null;
  product:      string | null;
  image_url:    string | null;
}
