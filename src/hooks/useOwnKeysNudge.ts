import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getCredentials } from "../api/credentials";
import { toast } from "../store/toast.store";

const SEEN_KEY = "own-keys-nudge-at";
const EVERY_MS = 7 * 24 * 60 * 60 * 1000;

/** A quiet suggestion to connect your own Spotify app, at most once a week,
 *  only while on the shared built-in access. */
export function useOwnKeysNudge(loggedIn: boolean) {
  const navigate = useNavigate();

  useEffect(() => {
    if (!loggedIn) return;
    const last = Number(localStorage.getItem(SEEN_KEY) ?? 0);
    if (Date.now() - last < EVERY_MS) return;

    // let startup settle so it doesn't land on top of the first paint
    const timer = setTimeout(() => {
      getCredentials()
        .then((creds) => {
          if (creds?.is_custom) return;
          localStorage.setItem(SEEN_KEY, String(Date.now()));
          toast("Tip: your own Spotify app makes search and sync more reliable", {
            kind:   "info",
            action: { label: "Set up", onClick: () => navigate("/settings") },
          });
        })
        .catch(() => {});
    }, 8000);
    return () => clearTimeout(timer);
  }, [loggedIn, navigate]);
}
