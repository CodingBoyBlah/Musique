import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { WindowEffect } from "../api/window";

interface UIState {
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  windowEffect: WindowEffect;
  setWindowEffect: (e: WindowEffect) => void;

  materialTransparency: number;
  setMaterialTransparency: (v: number) => void;

  pageTint: string | null;
  setPageTint: (url: string | null) => void;
  
  backdropActive: boolean;
  setBackdropActive: (v: boolean) => void;
  
  fastMode: boolean;
  setFastMode: (enabled: boolean) => void;
  savedWindowEffect?: WindowEffect;

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
      materialTransparency: 0.4,
      setMaterialTransparency: (v) => set({ materialTransparency: Math.max(0.1, Math.min(0.7, v))}),
      pageTint: null,
      setPageTint: (url) => set({ pageTint: url }),
      backdropActive: false,
      setBackdropActive: (v) => set({ backdropActive: v }),
      fastMode: false,
      setFastMode: (enabled) =>
        set((s) => {
          if (enabled) {
            return {
              fastMode: true,
              savedWindowEffect: s.windowEffect !== "none" ? s.windowEffect : s.savedWindowEffect || "mica",
              windowEffect: "none",
            };
          } else {
            return {
              fastMode: false,
              windowEffect: s.savedWindowEffect || "mica",
            };
          }
        }),
      quitConfirmOpen: false,
      setQuitConfirmOpen: (v) => set({ quitConfirmOpen: v }),
    }),
    {
      name: "spotify-ui",
     
      partialize: (s) => ({
        fastMode:         s.fastMode,
        savedWindowEffect: s.savedWindowEffect,
        windowEffect:     s.windowEffect,
        sidebarCollapsed: s.sidebarCollapsed,
        materialTransparency: s.materialTransparency,
      }),
    },
  ),
);
