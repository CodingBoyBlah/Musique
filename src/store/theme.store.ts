import { create } from "zustand";
import { persist } from "zustand/middleware";


export type ThemeSource = "default" | "wallpaper" | "system";

interface ThemeStore {
  source: ThemeSource;
  setSource: (s: ThemeSource) => void;


  albumColors: boolean;
  setAlbumColors: (v: boolean) => void;

  
  baseAccent: string | null;
  setBaseAccent: (hex: string | null) => void;

  refreshKey: number;
  refreshTheme: () => void;
}

export const useThemeStore = create<ThemeStore>()(
  persist(
    (set) => ({
      source: "default",
      setSource: (s) => set({ source: s }),
      albumColors: false,
      setAlbumColors: (v) => set({ albumColors: v }),
      baseAccent: null,
      setBaseAccent: (hex) => set({ baseAccent: hex }),
      refreshKey: 0,
      refreshTheme: () => set((s) => ({ refreshKey: s.refreshKey + 1})),
    }),
    
    {
      name: "musique-theme",
      
      partialize: (s) => ({ source: s.source, albumColors: s.albumColors }),
    },
  ),
);
