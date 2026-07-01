import type { CSSProperties } from "react";


export const gpuLayer: CSSProperties = {
  willChange: "transform",
  backfaceVisibility: "hidden",
};

// transformTemplate that always prepends translateZ(0) (forces a 3D layer)
export const zTransform = (_: unknown, generated: string) => `translateZ(0) ${generated}`;
