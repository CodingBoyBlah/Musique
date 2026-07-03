import { useEffect } from "react";
import { useThemeStore } from "../store/theme.store";
import { getWallpaperDataUrl, getSystemAccent } from "../api/theme";
import { applyAccent, dataUrlAccent } from "../lib/color";

export function ThemeEngine() {
  const source = useThemeStore((s) => s.source);
  const setBaseAccent = useThemeStore((s) => s.setBaseAccent);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      let hex: string | null = null;
      if (source === "wallpaper") {
        const data = await getWallpaperDataUrl().catch(() => null);
        if (data) hex = await dataUrlAccent(data);
      } else if (source === "system") {
        hex = await getSystemAccent().catch(() => null); 
      }
      if (cancelled) return;
      setBaseAccent(hex);
      applyAccent(hex); 
    })();
    return () => { cancelled = true; };
  }, [source, setBaseAccent]);

  return null;
}
