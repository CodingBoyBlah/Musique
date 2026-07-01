import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { WindowEffect } from "../api/window";

interface UIState {
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  windowEffect: WindowEffect;
  setWindowEffect: (e: WindowEffect) => void;

  pageTint: string | null;
  setPageTint: (url: string | null) => void;
  
  backdropActive: boolean;
  setBackdropActive: (v: boolean) => void;
  
  quitConfirmOpen: boolean;
  setQuitConfirmOpen: (v: boolean) => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      toggleSidebar: () =>
        set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      windowEffect: "mica",
      setWindowEffect: (e) => set({ windowEffect: e }),
      pageTint: null,
      setPageTint: (url) => set({ pageTint: url }),
      backdropActive: false,
      setBackdropActive: (v) => set({ backdropActive: v }),
      quitConfirmOpen: false,
      setQuitConfirmOpen: (v) => set({ quitConfirmOpen: v }),
    }),
    {
      name: "spotify-ui",
     
      partialize: (s) => ({
        windowEffect:     s.windowEffect,
        sidebarCollapsed: s.sidebarCollapsed,
      }),
    },
  ),
);
