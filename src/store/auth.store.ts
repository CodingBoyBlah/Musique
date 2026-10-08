import { create } from "zustand";
import type { AuthStatus } from "../types/ipc";

interface AuthStore {
  phase: "idle" | "login" | "logout";
  generation: number;
  begin: (phase: "login" | "logout") => number | null;
  finish: (generation: number) => void;
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
  phase: "idle",
  generation: 0,
  begin: (phase) => {
    let generation: number | null = null;
    set((s) => {
      if (s.phase === "logout" || (phase === "login" && s.phase !== "idle")) return s;
      generation = s.generation + 1;
      return { phase, generation };
    });
    return generation;
  },
  finish: (generation) => set((s) => s.generation === generation ? { phase: "idle" } : s),
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
