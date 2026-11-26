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
  /** which service the account belongs to; absent on older builds = spotify */
  provider?:    Provider;
}

export type Provider = "spotify" | "apple";

/** Both services' sign-ins, and which one the app is showing. */
export interface Accounts {
  active:  Provider;
  spotify: AuthStatus;
  apple:   AuthStatus;
}
