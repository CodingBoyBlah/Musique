import { startTransition, useEffect, useState } from "react";

/* A long list's first paint without the freeze.

   A liked-songs or playlist page used to mount all of its rows (200-250, each
   with its tooltips, buttons and menus) in the one synchronous render that
   ran when the data arrived: a 150-170ms task that froze the window and ate
   the page's enter animation. Now the first `first` rows - more than a
   window's worth - render with the data, and the rest follow in a React
   transition, which renders in small slices between frames instead of one
   block. Nothing on screen changes; the rows below the fold arrive a few
   frames later.

   Staging happens once per mount. After that every list the component
   renders (a new sort, a filter) is shown in full. */
export function useStagedList<T>(items: readonly T[], first = 40): readonly T[] {
  const [full, setFull] = useState(false);
  const staged = !full && items.length > first;

  useEffect(() => {
    if (staged) startTransition(() => setFull(true));
  }, [staged]);

  return staged ? items.slice(0, first) : items;
}
