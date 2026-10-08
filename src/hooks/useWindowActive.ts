import { useSyncExternalStore } from "react";

/* Is the user actually looking at this window?
 *
 * A CSS transform animation damages the frame on every tick, so WebView2 keeps
 * recompositing the whole window at 60fps for as long as one is running -
 * whether or not anyone can see it. The immersive backdrop drifts forever by
 * design, which means it was quietly costing a chunk of a core while the app
 * sat behind a browser or a code editor.
 *
 * Chromium throttles rAF for a genuinely hidden document, but a window that is
 * merely unfocused - the common case on a desktop - is not hidden, and its
 * animations keep running at full rate. So we watch focus as well as
 * visibility, and the ambient parks itself when neither holds. */

const listeners = new Set<() => void>();
let isSubscribed = false;

function onStateChange() {
  listeners.forEach((l) => l());
}

function subscribe(callback: () => void) {
  listeners.add(callback);
  if (!isSubscribed && typeof window !== "undefined") {
    isSubscribed = true;
    window.addEventListener("focus", onStateChange);
    window.addEventListener("blur", onStateChange);
    document.addEventListener("visibilitychange", onStateChange);
  }
  return () => {
    listeners.delete(callback);
    if (listeners.size === 0 && isSubscribed && typeof window !== "undefined") {
      isSubscribed = false;
      window.removeEventListener("focus", onStateChange);
      window.removeEventListener("blur", onStateChange);
      document.removeEventListener("visibilitychange", onStateChange);
    }
  };
}

function getSnapshot(): boolean {
  if (typeof document === "undefined") return true;
  return !document.hidden && document.hasFocus();
}

function getServerSnapshot(): boolean {
  return true;
}

export function useWindowActive(): boolean {
  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}
