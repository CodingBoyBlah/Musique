import { describe, expect, it } from "vitest";
import { lyricSeekTarget } from "./lyrics";

describe("clicking a lyric line", () => {
  it("seeks to the line's start when no offset is set", () => {
    expect(lyricSeekTarget(30_237, 0)).toBe(30_237);
  });

  it("lands on the line under the user's offset, which the clock adds to the position", () => {
    // offset +200: lines show 200ms early, so the line's audio is at 30_037
    expect(lyricSeekTarget(30_237, 200)).toBe(30_037);
    expect(lyricSeekTarget(30_237, -150)).toBe(30_387);
  });

  it("never seeks before the start of the track", () => {
    expect(lyricSeekTarget(100, 400)).toBe(0);
  });
});
