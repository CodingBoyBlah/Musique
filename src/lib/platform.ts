
export type OS = "windows" | "macos" | "linux";

function detect(): OS {
  const ua = navigator.userAgent;
  if (/Mac OS X|Macintosh/i.test(ua)) return "macos";
  if (/Windows/i.test(ua)) return "windows";
  return "linux";
}

export const os: OS = detect();
export const isMac = os === "macos";
export const isWindows = os === "windows";
export const isLinux = os === "linux";

// lets CSS pick per-OS details (the macOS system font)
if (typeof document !== "undefined") document.documentElement.dataset.os = os;
