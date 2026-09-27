import { getCurrentWebview } from "@tauri-apps/api/webview";
import { usePrefsStore } from "../store/prefs.store";
import { useToastStore } from "../store/toast.store";
import { isMac } from "./platform";

/* Whole-app zoom, the browser's Ctrl +/- for the entire window.

The webview's own zoom (setZoom) scales every CSS pixel, so type, spacing,
artwork and hit areas all grow together and the responsive layouts reflow for
the smaller effective viewport, exactly like zooming a web page. WebView2's
built-in zoom keys stay off (tauri's zoomHotkeysEnabled): they exist only on
Windows and forget the level on restart. This works on every platform and
the level is kept in prefs. */

// the browser's ladder, trimmed at the small end where nothing is legible
export const ZOOM_STEPS = [0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2] as const;
export const ZOOM_MIN = ZOOM_STEPS[0];
export const ZOOM_MAX = ZOOM_STEPS[ZOOM_STEPS.length - 1];

export const zoomLabel = (z: number) => `${Math.round(z * 100)}%`;

function nearestIndex(z: number) {
  let best = 0;
  for (let i = 1; i < ZOOM_STEPS.length; i++) {
    if (Math.abs(ZOOM_STEPS[i] - z) < Math.abs(ZOOM_STEPS[best] - z)) best = i;
  }
  return best;
}

export function canZoom(dir: -1 | 1) {
  const i = nearestIndex(usePrefsStore.getState().uiZoom);
  return dir > 0 ? i < ZOOM_STEPS.length - 1 : i > 0;
}

let toastId: number | null = null;

// one toast that updates in place, not a stack of them while you hold the key
function announce(z: number) {
  const store = useToastStore.getState();
  if (toastId !== null) store.remove(toastId);
  toastId = store.push(`Zoom ${zoomLabel(z)}`, { kind: "info" });
}

export function setZoom(z: number, { quiet = false } = {}) {
  const next = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, z));
  if (next === usePrefsStore.getState().uiZoom) return;
  usePrefsStore.getState().setUiZoom(next);
  if (!quiet) announce(next);
}

export function stepZoom(dir: -1 | 1, opts?: { quiet?: boolean }) {
  const i = nearestIndex(usePrefsStore.getState().uiZoom);
  const j = Math.min(ZOOM_STEPS.length - 1, Math.max(0, i + dir));
  setZoom(ZOOM_STEPS[j], opts);
}

export const resetZoom = (opts?: { quiet?: boolean }) => setZoom(1, opts);

function apply(z: number) {
  // --zoom lets the few things pinned to native window geometry (the macOS
  // traffic lights) undo the scale; see chromePx
  document.documentElement.style.setProperty("--zoom", String(z));
  try {
    getCurrentWebview().setZoom(z).catch((err) => console.error("[zoom] setZoom failed:", err));
  } catch {
    // not inside tauri (plain browser dev): nothing to zoom
  }
}

/* A length tied to the OS window chrome rather than to the page. The macOS
traffic lights are native and never scale, so the strip and rail width
reserved for them have to stay the same physical size at any zoom. Everywhere
else a CSS pixel should scale, so this is a no-op off macOS. */
export function chromePx(px: number, zoom: number) {
  return isMac ? px / zoom : px;
}

let installed = false;

export function installZoom() {
  if (installed) return;
  installed = true;

  apply(usePrefsStore.getState().uiZoom);
  usePrefsStore.subscribe((s, prev) => {
    if (s.uiZoom !== prev.uiZoom) apply(s.uiZoom);
  });

  window.addEventListener("keydown", (e) => {
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
    // match the key produced and the numpad, so Ctrl + works with or without Shift
    if (e.key === "=" || e.key === "+" || e.code === "NumpadAdd") {
      e.preventDefault();
      stepZoom(1);
    } else if (e.key === "-" || e.key === "_" || e.code === "NumpadSubtract") {
      e.preventDefault();
      stepZoom(-1);
    } else if (e.key === "0" || e.code === "Numpad0") {
      e.preventDefault();
      resetZoom();
    }
  });

  // Ctrl + wheel, and trackpad pinch (which arrives as Ctrl + wheel). Deltas
  // are summed so a pinch's many small events step the zoom smoothly instead
  // of jumping a whole step per event; one mouse notch (~100) is one step.
  let acc = 0;
  let accTimer: ReturnType<typeof setTimeout> | undefined;
  window.addEventListener(
    "wheel",
    (e) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      acc += e.deltaY;
      clearTimeout(accTimer);
      accTimer = setTimeout(() => { acc = 0; }, 250);
      while (Math.abs(acc) >= 100) {
        stepZoom(acc < 0 ? 1 : -1);
        acc -= Math.sign(acc) * 100;
      }
    },
    { passive: false },
  );
}
