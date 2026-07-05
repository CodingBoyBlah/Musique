import { useEffect, useState } from "react";
import { useThemeStore } from "../store/theme.store";
import { useUIStore } from "../store/ui.store";
import { getWallpaperDataUrl, getSystemAccent } from "../api/theme";
import { applyAccent, dataUrlAccent, loadCoverAccent } from "../lib/color";


export function ThemeEngine() {
  const source      = useThemeStore((s) => s.source);
  const albumColors = useThemeStore((s) => s.albumColors);
  const pageTint    = useUIStore((s) => s.pageTint);

  

  const [base, setBase] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    const resolve = async () => {
      let hex: string | null = null;
      if (source === "wallpaper") {
        const data = await getWallpaperDataUrl().catch(() => null);
        if (data) hex = await dataUrlAccent(data); 
      } else if (source === "system") {
        hex = await getSystemAccent().catch(() => null);
      }
      if (!cancelled) setBase(hex);
    };
    resolve();

    
    if (source === "wallpaper" || source === "system") {
      const refresh = () => { if (!document.hidden) resolve(); };
      window.addEventListener("focus", refresh);
      document.addEventListener("visibilitychange", refresh);
      const iv = source === "wallpaper" ? window.setInterval(resolve, 20000) : 0;
      return () => {
        cancelled = true;
        window.removeEventListener("focus", refresh);
        document.removeEventListener("visibilitychange", refresh);
        if (iv) window.clearInterval(iv);
      };
    }
    return () => { cancelled = true; };
  }, [source]);

  
  useEffect(() => {
    let cancelled = false;
    (async () => {
      let effective = base;
      if (albumColors && pageTint) {
        const cover = await loadCoverAccent(pageTint); 
        if (cover) effective = cover;
      }
      if (!cancelled) applyAccent(effective); 
    })();
    return () => { cancelled = true; };
  }, [base, albumColors, pageTint]);

  return null;
}
