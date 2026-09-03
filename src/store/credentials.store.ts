import { create } from "zustand";
import type { Credentials } from "../api/credentials";

export type ConnectionStatus =
  | "unconfigured"
  | "configured"
  | "validating"
  | "valid"
  | "invalid";

interface CredentialsState {
  clientId: string | null;
  hasSecret: boolean;
  isCustom: boolean;
  status: ConnectionStatus;
  setStatus: (status: ConnectionStatus) => void;
  setFromCredentials: (creds: Credentials | null) => void;
  clear: () => void;
}

export const useCredentialsStore = create<CredentialsState>((set) => ({
  clientId: null,
  hasSecret: false,
  isCustom: false,
  status: "valid",

  setStatus: (status) => set({ status }),

  setFromCredentials: (creds) => {
    if (!creds) {
      set({ clientId: null, hasSecret: false, isCustom: false, status: "valid" });
      return;
    }
    const isCustom = Boolean(creds.is_custom);
    set({
      clientId: creds.client_id,
      hasSecret: creds.has_secret,
      isCustom,
      status: isCustom ? (creds.has_secret ? "configured" : "valid") : "valid",
    });
  },

  clear: () => set({ clientId: null, hasSecret: false, isCustom: false, status: "valid" }),
}));
