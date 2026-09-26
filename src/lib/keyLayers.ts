/* Keyboard layers.

   Modals and menus all listen for keys on window, and stopPropagation can't
   stop other listeners on that same target - so with a menu open inside a
   modal, or a quit dialog over an update prompt, one Escape used to close
   every one of them. Each open layer pushes itself here; only the topmost one
   answers. */
const stack: symbol[] = [];

export function pushKeyLayer(): symbol {
  const id = Symbol("key-layer");
  stack.push(id);
  return id;
}

export function popKeyLayer(id: symbol) {
  const at = stack.indexOf(id);
  if (at !== -1) stack.splice(at, 1);
}

export function isTopKeyLayer(id: symbol) {
  return stack[stack.length - 1] === id;
}
