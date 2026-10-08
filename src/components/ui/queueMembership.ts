type Queue = readonly { id: string }[];

// Queue updates replace the array. Share one index across all row selectors;
// old indexes can be collected as soon as their queue snapshots are released.
const membership = new WeakMap<Queue, Set<string>>();

export function queueHasTrack(queue: Queue, id: string): boolean {
  let ids = membership.get(queue);
  if (!ids) {
    ids = new Set<string>();
    for (const track of queue) ids.add(track.id);
    membership.set(queue, ids);
  }
  return ids.has(id);
}
