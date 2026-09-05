import { invoke } from "@tauri-apps/api/core";
import { queryClient } from "../main";

let lastTrim = 0;

/**
 * Proactively evicts unobserved React Query caches, invokes V8 garbage collection
 * (if enabled via --expose-gc), and calls Win32 K32EmptyWorkingSet to collapse
 * WebView2 and Tauri process memory in physical RAM.
 */
export async function trimMemory(force = false): Promise<void> {
  const now = Date.now();
  if (!force && now - lastTrim < 10_000) return;
  lastTrim = now;

  try {
    if (queryClient) {
      const cache = queryClient.getQueryCache();
      const queries = cache.getAll();
      for (const q of queries) {
        if (q.getObserversCount() === 0) {
          cache.remove(q);
        }
      }
    }

    if (typeof (window as unknown as { gc?: () => void }).gc === "function") {
      (window as unknown as { gc: () => void }).gc();
    }

    await invoke("trim_memory");
  } catch {
    // Non-fatal if unsupported or outside Tauri
  }
}
