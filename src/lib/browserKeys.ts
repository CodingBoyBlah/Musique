/* The webview still answers the browser's own shortcuts: Ctrl+P and
Ctrl+Shift+P open a print dialog, Ctrl+F a find bar, Ctrl+U the page source,
F7 caret browsing, and so on. None of that belongs in an app window.

This cancels the browser's default action only. It doesn't stop propagation,
so the app's own handlers (Ctrl+K search, zoom) still see the key. Shipped
Windows builds also turn these keys off in WebView2 itself (lib.rs). Dev
builds keep reload and devtools so `pnpm tauri dev` stays debuggable. */

const DEV = import.meta.env.DEV;

function isBrowserKey(e: KeyboardEvent): boolean {
  const mod = e.ctrlKey || e.metaKey;
  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;

  // devtools and reload: dev only
  if (!DEV) {
    if (key === "F12") return true;
    if (mod && e.shiftKey && (key === "i" || key === "j" || key === "c")) return true;
    // Cmd+Option+I / J / C on macOS
    if (e.metaKey && e.altKey && (e.code === "KeyI" || e.code === "KeyJ" || e.code === "KeyC")) return true;
    if (key === "F5" || key === "BrowserRefresh") return true;
    if (mod && key === "r") return true;
  }

  if (key === "F3" || key === "F7") return true; // find next, caret browsing
  if (!mod || e.altKey) return false;
  // print, with or without Shift (Ctrl+Shift+P is the system print dialog)
  if (key === "p") return true;
  // find, view source, save page, open file, history, downloads. Shift is left
  // alone so Ctrl+Shift+J/C still reach devtools in dev
  return !e.shiftKey && ["f", "g", "u", "s", "o", "h", "j"].includes(key);
}

let installed = false;

export function installBrowserKeyGuard() {
  if (installed) return;
  installed = true;
  // capture phase, so it runs before anything can stop the event
  window.addEventListener(
    "keydown",
    (e) => {
      if (isBrowserKey(e)) e.preventDefault();
    },
    { capture: true },
  );
}
