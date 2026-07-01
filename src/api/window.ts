import { invoke } from "@tauri-apps/api/core";

export type WindowEffect = "mica" | "acrylic" | "none";

// switch the window backdrop material at runtime (windows only) so "none" clears  the material so the app paints its own solid background.
export const setWindowEffect = (mode: WindowEffect): Promise<void> =>
  invoke("set_window_effect", { mode });

/* true when a native backdrop material (win Mica OR acrylic or macos vibrancy) 
  actually applied. when false the UI has to paint an opaque dark bg (-- if set to none, then no TODO - done), otherwise
 the transparent window shows white behind it and white text vanishes. */
export const getBackdropActive = (): Promise<boolean> =>
  invoke("get_backdrop_active");
