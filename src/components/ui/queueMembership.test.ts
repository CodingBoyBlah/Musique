import { describe, expect, it } from "vitest";
import { queueHasTrack } from "./queueMembership";

describe("queue membership", () => {
  it("keeps membership correct for duplicates, empty queues and missing tracks", () => {
    const queue = [{ id: "a" }, { id: "a" }, { id: "b" }];
    expect(queueHasTrack(queue, "a")).toBe(true);
    expect(queueHasTrack(queue, "b")).toBe(true);
    expect(queueHasTrack(queue, "c")).toBe(false);
    expect(queueHasTrack([], "a")).toBe(false);
  });

  it("reflects removals and additions in a new queue snapshot", () => {
    const before = [{ id: "a" }, { id: "b" }];
    const after = [{ id: "b" }, { id: "c" }];
    expect(queueHasTrack(before, "a")).toBe(true);
    expect(queueHasTrack(after, "a")).toBe(false);
    expect(queueHasTrack(after, "c")).toBe(true);
    expect(queueHasTrack(before, "c")).toBe(false);
  });

  it("reads queue entries once across a whole list of row subscriptions", () => {
    let reads = 0;
    const queue = Array.from({ length: 1000 }, (_, index) => ({
      get id() {
        reads++;
        return String(index);
      },
    }));
    for (let index = 0; index < 1000; index++) {
      expect(queueHasTrack(queue, String(index))).toBe(true);
    }
    expect(reads).toBe(1000);
    expect(queueHasTrack(queue, "missing")).toBe(false);
    expect(reads).toBe(1000);
  });
});
