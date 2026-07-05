import type { WindowEffect } from "../api/window";

/* pick the CSS background for the window root.

the invariant that matters: when no native material is live (backdropActive
=== false) this ALWAYS returns the opaque app base colour, never transparent.
thats what stops the UI ever rendering white-on-white - if Mica/vibrancy
didnt apply (Linux, an old windows build, a failed macOS layer, or just
before rust has answered) we paint a solid dark bg instead of letting the
transparent window show through.

 - no material     -> opaque --color-base
 - effect "none"   -> opaque --color-base (material cleared on purpose)
 - windows acrylic -> dark scrim (OS ignores acrylic's tint param)
 - mica / vibrancy -> transparent, the OS material is the background */
export function backdropScrim(
  backdropActive: boolean,
  effect: WindowEffect,
  transparency = 0.4,
  isMac = false,
): string {
  if (!backdropActive) return "var(--color-base)";
  // material turned off -> paint the solid base so the now-transparent window
  // doesn't show white/the desktop behind it
  if (effect === "none") return "var(--color-base)";
  if (effect === "acrylic") return `rgba(8, 8, 10, ${(1 - transparency).toFixed(3)})`;
  if (effect === "mica" && isMac) return `rgba(10,10,14, ${(1 - transparency).toFixed(3)})`;
  return "transparent";
}
