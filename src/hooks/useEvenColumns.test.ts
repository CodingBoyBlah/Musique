import { describe, it, expect } from "vitest";
import { evenColumns } from "./useEvenColumns";

describe("evenColumns", () => {
  it("never leaves an orphan row", () => {
    // 6 tiles, 150px min, 10px gap
    expect(evenColumns(1100, 6, 150, 10)).toBe(6);
    expect(evenColumns(900, 6, 150, 10)).toBe(3); // 5 fit, but 5 doesn't divide 6
    expect(evenColumns(500, 6, 150, 10)).toBe(3);
    expect(evenColumns(400, 6, 150, 10)).toBe(2);
    expect(evenColumns(200, 6, 150, 10)).toBe(1);
  });
});
