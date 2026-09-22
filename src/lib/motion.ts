import type { CSSProperties } from "react";
import type { Transition } from "framer-motion";


export const gpuLayer: CSSProperties = {
  // Leaving empty at rest so Chromium/WebView2 renders icons and text
  // directly with subpixel precision rather than blurring on an isolated texture.
};

// Passthrough transform template (avoids forcing static translateZ(0) layer caching)
export const zTransform = (_: unknown, generated: string) => generated;

// Universal spring physics for responsive grid reflow and layout morphing
export const REFLOW_SPRING = {
  type: "spring" as const,
  stiffness: 320,
  damping: 32,
};

// Staggered layout transition helper inspired by Codrops / GSAP Flip
export const getGridItemTransition = (index: number = 0): { layout: Transition } => ({
  layout: {
    type: "spring" as const,
    stiffness: 320,
    damping: 32,
    delay: (index % 8) * 0.018,
  },
});
