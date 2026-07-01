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
  status: ConnectionStatus;
  setStatus: (status: ConnectionStatus) => void;
  setFromCredentials: (creds: Credentials | null) => void;
  clear: () => void;
}

export const useCredentialsStore = create<CredentialsState>((set) => ({
  clientId: null,
  hasSecret: false,
  status: "unconfigured",

  setStatus: (status) => set({ status }),

  setFromCredentials: (creds) => {
    if (!creds) {
      set({ clientId: null, hasSecret: false, status: "unconfigured" });
      return;
    }
    set({
      clientId: creds.client_id,
      hasSecret: creds.has_secret,
      status: creds.client_id && creds.has_secret ? "configured" : "unconfigured",
    });
  },

  clear: () => set({ clientId: null, hasSecret: false, status: "unconfigured" }),
}));
