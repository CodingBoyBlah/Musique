import { useEffect, useState } from "react";

/* Is the user actually looking at this window?
 *
 * A CSS transform animation damages the frame on every tick, so WebView2 keeps
 * recompositing the whole window at 60fps for as long as one is running -
 * whether or not anyone can see it. The immersive backdrop drifts forever by
 * design, which means it was quietly costing a chunk of a core while the app
 * sat behind a browser or a code editor.
 *
 * Chromium throttles rAF for a genuinely hidden document, but a window that is
 * merely unfocused - the common case on a desktop - is not hidden, and its
 * animations keep running at full rate. So we watch focus as well as
 * visibility, and the ambient parks itself when neither holds. */
export function useWindowActive(): boolean {
  const [active, setActive] = useState(true);

  useEffect(() => {
    const read = () => setActive(!document.hidden && document.hasFocus());
    read();

    window.addEventListener("focus", read);
    window.addEventListener("blur", read);
    document.addEventListener("visibilitychange", read);
    return () => {
      window.removeEventListener("focus", read);
      window.removeEventListener("blur", read);
      document.removeEventListener("visibilitychange", read);
    };
  }, []);

  return active;
}
