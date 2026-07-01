import { describe, it, expect } from "vitest";
import { backdropScrim } from "./backdrop";

const OPAQUE = "var(--color-base)";

describe("backdropScrim", () => {
  // the load-bearing one: white-screen guard. no live material = opaque.
  it("is opaque whenever no native material is active", () => {
    expect(backdropScrim(false, "mica")).toBe(OPAQUE);
    expect(backdropScrim(false, "acrylic")).toBe(OPAQUE);
  });

  it("never returns transparent without a material (no white-on-white)", () => {
    for (const effect of ["mica", "acrylic"] as const) {
      expect(backdropScrim(false, effect)).not.toBe("transparent");
    }
  });

  it("stays transparent under Mica / macOS vibrancy so the OS material shows", () => {
    expect(backdropScrim(true, "mica")).toBe("transparent");
  });

  it("darkens with a scrim under Windows acrylic", () => {
    expect(backdropScrim(true, "acrylic")).toBe("rgba(8, 8, 10, 0.6)");
  });

  it("is opaque when the material is explicitly turned off (none)", () => {
    expect(backdropScrim(true, "none")).toBe(OPAQUE);
    expect(backdropScrim(false, "none")).toBe(OPAQUE);
  });
});
