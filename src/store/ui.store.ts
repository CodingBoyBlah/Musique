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

  // bumped when a closing side panel has finished sliding out and the right
  // rail actually gives its width back. Layout holds the rail open until then
  // (so the exit is seen), and useReflowPulse subscribes to it so the grid
  // cards re-measure on the same render the column widens - they glide into
  // the freed space rather than teleport.
  railSettleTick: number;
  bumpRailSettle: () => void;

  // the immersive overlay has finished fading in and fully covers the shell.
  // set by Immersive from its own onAnimationComplete; Layout hides (and
  // inerts) the shell underneath on it, instead of guessing with a timer.
  immersiveCovered: boolean;
  setImmersiveCovered: (v: boolean) => void;
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
      railSettleTick: 0,
      bumpRailSettle: () => set((s) => ({ railSettleTick: s.railSettleTick + 1 })),
      immersiveCovered: false,
      setImmersiveCovered: (v) => set({ immersiveCovered: v }),
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
