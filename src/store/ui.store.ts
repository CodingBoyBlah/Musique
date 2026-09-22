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
  
  quitConfirmOpen: boolean;
  setQuitConfirmOpen: (v: boolean) => void;

  macSimulated: boolean;
  toggleMacSimulated: () => void;
  setMacSimulated: (v: boolean) => void;

  searchPaletteOpen: boolean;
  setSearchPaletteOpen: (v: boolean) => void;
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
      quitConfirmOpen: false,
      setQuitConfirmOpen: (v) => set({ quitConfirmOpen: v }),
      macSimulated: false,
      toggleMacSimulated: () => set((s) => ({ macSimulated: !s.macSimulated })),
      setMacSimulated: (v) => set({ macSimulated: v }),
      searchPaletteOpen: false,
      setSearchPaletteOpen: (v) => set({ searchPaletteOpen: v }),
    }),
    {
      name: "spotify-ui",
     
      partialize: (s) => ({
        windowEffect:     s.windowEffect,
        sidebarCollapsed: s.sidebarCollapsed,
        materialTransparency: s.materialTransparency,
      }),
    },
  ),
);
