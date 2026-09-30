import { describe, it, expect } from "vitest";
import { findFolder, flattenRows, countPlaylists, hasFolders } from "./rootlist";
import type { RootItem } from "../api/internal";

const pl = (id: string): RootItem => ({ kind: "playlist", id, name: id, image_url: null, length: null });
const tree: RootItem[] = [
  pl("a"),
  { kind: "folder", id: "f1", name: "Trips", children: [pl("b"), { kind: "folder", id: "f2", name: "Night", children: [pl("c")] }] },
  pl("d"),
];

describe("rootlist helpers", () => {
  it("finds nested folders with their trail", () => {
    const hit = findFolder(tree, "f2");
    expect(hit?.folder.name).toBe("Night");
    expect(hit?.trail.map((f) => f.id)).toEqual(["f1"]);
    expect(findFolder(tree, "nope")).toBeNull();
  });

  it("counts playlists through nesting", () => {
    expect(countPlaylists(tree)).toBe(4);
    expect(hasFolders(tree)).toBe(true);
  });

  it("flattens with closed folders hidden", () => {
    const open = flattenRows(tree, new Set());
    expect(open.map((r) => r.id)).toEqual(["a", "f1", "b", "f2", "c", "d"]);
    expect(open.find((r) => r.id === "c")?.depth).toBe(2);
    const closed = flattenRows(tree, new Set(["f1"]));
    expect(closed.map((r) => r.id)).toEqual(["a", "f1", "d"]);
  });
});
