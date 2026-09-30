import { describe, it, expect } from "vitest";
import { fmtMs } from "./fmt";

describe("fmtMs", () => {
  it("formats minutes and hours", () => {
    expect(fmtMs(0)).toBe("0:00");
    expect(fmtMs(65_000)).toBe("1:05");
    expect(fmtMs((118 * 60 + 7) * 1000)).toBe("1:58:07");
  });
});
