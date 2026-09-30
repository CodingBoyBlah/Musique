import type { CSSProperties } from "react";
import type { Transition } from "framer-motion";


export const gpuLayer: CSSProperties = {
  // Leaving empty at rest so Chromium/WebView2 renders icons and text
  // directly with subpixel precision rather than blurring on an isolated texture.
};

// Passthrough transform template (avoids forcing static translateZ(0) layer caching)
export const zTransform = (_: unknown, generated: string) => generated;

/* ── house motion tokens ───────────────────────────────────────────────────
 * One vocabulary for the whole app, so the same kind of motion always has the
 * same curve. Mirrors the --ease-* custom properties in index.css. */

// strong ease-out: entering, responding to a press
export const EASE_OUT = [0.23, 1, 0.32, 1] as const;
// strong ease-in-out: something moving or morphing while on screen
export const EASE_IN_OUT = [0.77, 0, 0.175, 1] as const;
// iOS-style sheet curve: drawers and side panels
export const EASE_DRAWER = [0.32, 0.72, 0, 1] as const;

// Critically damped default: settles without overshoot. Nothing a user merely
// clicked should bounce; bounce is reserved for things that were flicked.
export const SPRING = { type: "spring" as const, bounce: 0, duration: 0.3 };
// Slightly slower sibling for larger surfaces (panels, sheets).
export const SPRING_PANEL = { type: "spring" as const, bounce: 0, duration: 0.4 };

// The right rail (lyrics / queue / friends). Every piece of a rail slide runs
// on these: the rail's frame (Layout), each panel's sheet, and the tile grids'
// FLIP (lib/railFlip, which drives the Web Animations API and so needs the
// same curve as a CSS string). One curve for all of them is what keeps the
// card's edge, the panel and the cards travelling as one piece.
//
// A tween on the sheet curve rather than a spring: it leaves at speed, so the
// panel answers the click on the very next frame, and spends the rest of its
// time landing softly. A spring had to be long to be smooth, and read as slow.
export const RAIL_OPEN_S = 0.36;
export const RAIL_CLOSE_S = 0.3;
export const RAIL_EASE_CSS = `cubic-bezier(${EASE_DRAWER.join(", ")})`;
export const RAIL_OPEN = { duration: RAIL_OPEN_S, ease: EASE_DRAWER };
export const RAIL_CLOSE = { duration: RAIL_CLOSE_S, ease: EASE_DRAWER };

// Press feedback for framer-driven buttons. Subtle on purpose: these are
// pressed dozens of times a day.
export const PRESS = { scale: 0.96 };
export const PRESS_TRANSITION: Transition = { duration: 0.12, ease: EASE_OUT };

// Universal spring physics for responsive grid reflow and layout morphing.
// damping = 2*sqrt(stiffness) -> critically damped (zeta = 1.0), so a reflow
// glides into place without the small overshoot the old 32 had.
export const REFLOW_SPRING = {
  type: "spring" as const,
  stiffness: 320,
  damping: 36,
};

// Staggered layout transition helper inspired by Codrops / GSAP Flip
export const getGridItemTransition = (index: number = 0): { layout: Transition } => ({
  layout: {
    ...REFLOW_SPRING,
    delay: (index % 8) * 0.018,
  },
});
