import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { useUpdaterStore } from "../store/updater.store";

// The pending Update handle lives here (module scope), NOT in the store: it holds
// a live native handle + methods and isn't serializable.
let pending: Update | null = null;

// ONE silent check per startup. No polling, no interval. If there's no update —
// or we're in dev / offline / the endpoint isn't reachable — it stays completely
// silent (no dialog, no toast), which is exactly the "do nothing" requirement.
export async function runUpdateCheck(): Promise<void> {
  try {
    const update = await check();
    if (!update) return; // up to date -> nothing at all
    pending = update;
    useUpdaterStore
      .getState()
      .showAvailable(update.version, update.body ?? null);
  } catch {
    // dev build (no updater), no endpoint, or offline -> silent no-op
  }
}

// Download + install with live progress, then flip to the "installed" stage so the
// UI can offer "Restart now". Uses the official plugin's streaming callback.
export async function startDownload(): Promise<void> {
  const s = useUpdaterStore.getState();
  if (!pending) return;
  s.setDownloading();
  try {
    let downloaded = 0;
    let total = 0;
    await pending.downloadAndInstall((e) => {
      switch (e.event) {
        case "Started":
          total = e.data.contentLength ?? 0;
          break;
        case "Progress":
          downloaded += e.data.chunkLength;
          if (total > 0) {
            s.setProgress(
              Math.min(100, Math.round((downloaded / total) * 100)),
            );
          }
          break;
        case "Finished":
          s.setProgress(100);
          break;
      }
    });
    s.setInstalled();
  } catch (err) {
    s.setError(err instanceof Error ? err.message : String(err));
  }
}

// relaunch into the freshly-installed version.
export async function restartApp(): Promise<void> {
  await relaunch();
}
