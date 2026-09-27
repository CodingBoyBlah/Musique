import type { RootItem } from "../api/internal";

export type FolderItem = Extract<RootItem, { kind: "folder" }>;

// depth-first search for a folder, with the folders you pass through on the way
export function findFolder(tree: RootItem[], id: string, trail: FolderItem[] = []): { folder: FolderItem; trail: FolderItem[] } | null {
  for (const item of tree) {
    if (item.kind !== "folder") continue;
    if (item.id === id) return { folder: item, trail };
    const hit = findFolder(item.children, id, [...trail, item]);
    if (hit) return hit;
  }
  return null;
}

// how many playlists live anywhere under a folder
export function countPlaylists(items: RootItem[]): number {
  return items.reduce((n, it) => n + (it.kind === "playlist" ? 1 : countPlaylists(it.children)), 0);
}

export type SidebarRow =
  | { kind: "folder"; id: string; name: string; depth: number; open: boolean; count: number }
  | { kind: "playlist"; id: string; depth: number };

/* the tree as sidebar rows: folders become headers, a closed folder hides
everything under it. depth drives the indent. */
export function flattenRows(tree: RootItem[], closed: ReadonlySet<string>, depth = 0): SidebarRow[] {
  const rows: SidebarRow[] = [];
  for (const item of tree) {
    if (item.kind === "playlist") {
      rows.push({ kind: "playlist", id: item.id, depth });
      continue;
    }
    const open = !closed.has(item.id);
    rows.push({ kind: "folder", id: item.id, name: item.name, depth, open, count: countPlaylists(item.children) });
    if (open) rows.push(...flattenRows(item.children, closed, depth + 1));
  }
  return rows;
}

export function hasFolders(tree: RootItem[]): boolean {
  return tree.some((it) => it.kind === "folder");
}
