import { afterEach, describe, expect, it, vi } from "vitest";
import { applyAccent, hexToRgb, luminance } from "./color";

afterEach(() => vi.unstubAllGlobals());

describe("accent text contrast", () => {
  it("keeps text readable across dark, midtone, and bright accent colors", () => {
    const values = new Map<string, string>();
    vi.stubGlobal("document", { documentElement: { style: {
      setProperty: (key: string, value: string) => values.set(key, value),
      removeProperty: (key: string) => values.delete(key),
    } } });
    for (const color of ["#000000", "#ffffff", "#5873d8", "#5068c4", "#777777", "#ff0000", "#00ff00", "#0000ff", "#ffcc00", "#dd44aa"]) {
      applyAccent(color);
      const bg = luminance(hexToRgb(color));
      const fg = luminance(hexToRgb(values.get("--color-accent-text")!));
      expect((Math.max(bg, fg) + 0.05) / (Math.min(bg, fg) + 0.05), color).toBeGreaterThanOrEqual(4.5);
    }
    applyAccent(null);
    expect(values.size).toBe(0);
  });
});
