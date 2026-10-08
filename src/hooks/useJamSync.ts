import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { getJam, type JamSession, type JamUpdate } from "../api/social";
import type { ConnectState } from "../api/playback";
import { useJamStore } from "../store/jam.store";
import { useAuthStore } from "../store/auth.store";
import { toast } from "../store/toast.store";
import { applyConnectState, leftJam, refreshConnectState } from "../lib/jam";

function hostName(s: JamSession | null): string {
  return s?.members.find((m) => m.is_host)?.name ?? "The host";
}

function namesIn(a: JamSession, b: JamSession | null): string[] {
  const other = new Set((b?.members ?? []).map((m) => m.id));
  return a.members.filter((m) => !other.has(m.id) && !m.is_current_user).map((m) => m.name);
}

function adopt(next: JamSession | null) {
  const prev = useJamStore.getState().session;
  useJamStore.getState().setSession(next);
  // just joined (or the app started inside a jam): pick up spotify's state
  if (next && !prev) refreshConnectState(true);
}

export async function refreshJam(): Promise<JamSession | null> {
  const generation = useAuthStore.getState().generation;
  const jam = await getJam().catch(() => undefined);
  if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return null;
  if (jam === undefined) return useJamStore.getState().session; // offline: keep what we know
  const prev = useJamStore.getState().session;
  if (!jam && prev) {
    await leftJam(prev.is_host);
    return null;
  }
  adopt(jam);
  return jam;
}

/* keeps the jam store in step with spotify: the session from social-connect
   pushes (with a slow poll as a backstop), the queue and now playing from
   this device's connect state. mounted once, while signed in. */
export function useJamSync(enabled: boolean) {
  const qc = useQueryClient();

  useEffect(() => {
    if (!enabled) return;
    let gone = false;
    const offs: Array<() => void> = [];
    const keep = (p: Promise<() => void>) =>
      p.then((u) => (gone ? u() : offs.push(u))).catch(() => {});

    refreshJam().then(() => qc.setQueryData(["jam"], useJamStore.getState().session));

    keep(
      listen<JamUpdate | null>("social:jam-updated", async (e) => {
        if (gone || !useAuthStore.getState().loggedIn) return;
        const update = e.payload;
        const prev = useJamStore.getState().session;
        const reason = update?.reason ?? "UNKNOWN_UPDATE_TYPE";

        switch (reason) {
          case "SESSION_DELETED":
            if (prev) {
              toast(prev.is_host ? "Your Jam ended" : `${hostName(prev)} ended the Jam`);
              await leftJam(prev.is_host);
            }
            break;
          case "YOU_WERE_KICKED":
            if (prev) toast("You were removed from the Jam");
            await leftJam(false);
            break;
          case "YOU_LEFT":
            if (prev) await leftJam(prev.is_host);
            break;
          default: {
            const next = update?.session ?? (await refreshJam());
            if (!next) break;
            if (prev && prev.session_id === next.session_id) {
              if (reason === "USER_JOINED") namesIn(next, prev).forEach((n) => toast(`${n} joined the Jam`));
              if (reason === "USER_LEFT") namesIn(prev, next).forEach((n) => toast(`${n} left the Jam`));
              if (reason === "PARTICIPANT_PROMOTED_TO_HOST" && next.is_host && !prev.is_host) toast("You're now the host of this Jam");
            }
            adopt(next);
          }
        }
        qc.setQueryData(["jam"], useJamStore.getState().session);
      }),
    );

    keep(listen<ConnectState>("connect:state", (e) => {
      if (!gone && useAuthStore.getState().loggedIn) applyConnectState(e.payload);
    }));

    // pushes can be missed across a reconnect; a slow poll catches up
    const poll = setInterval(() => {
      refreshJam().then(() => qc.setQueryData(["jam"], useJamStore.getState().session));
    }, 60_000);

    return () => {
      gone = true;
      clearInterval(poll);
      offs.forEach((u) => u());
    };
  }, [enabled, qc]);
}
