import { create } from "zustand";
import type { AuthStatus } from "../types/ipc";

interface AuthStore {
  loggedIn:    boolean;
  userId:      string | null;
  displayName: string | null;
  email:       string | null;
  product:     string | null;
  imageUrl:    string | null;
  setFromStatus: (s: AuthStatus) => void;
  clear: () => void;
}

export const useAuthStore = create<AuthStore>((set) => ({
  loggedIn:    false,
  userId:      null,
  displayName: null,
  email:       null,
  product:     null,
  imageUrl:    null,

  setFromStatus: (s) =>
    set({
      loggedIn:    s.logged_in,
      userId:      s.user_id,
      displayName: s.display_name,
      email:       s.email,
      product:     s.product,
      imageUrl:    s.image_url,
    }),

  clear: () =>
    set({
      loggedIn:    false,
      userId:      null,
      displayName: null,
      email:       null,
      product:     null,
      imageUrl:    null,
    }),
}));
