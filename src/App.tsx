import { useCallback, useEffect, useRef, lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/layout/Layout";
import { Loader } from "./components/ui/Loader";

/* route level code splitting,  each page loads on first visit instead of all of  it landing in the initial bundle. cuts startup load on older machines
and keeps the heavy stuff (kuromoji/romanize via lyrics, charts, etc) off the
critical path. Home stays eager since it's the landing rou lazying it
ntoe to self: dont just add a flash */
import Home from "./pages/Home";
const Settings     = lazy(() => import("./pages/Settings"));
const Search       = lazy(() => import("./pages/Search"));
const Library      = lazy(() => import("./pages/Library"));
const Profile      = lazy(() => import("./pages/Profile"));
const Playlists    = lazy(() => import("./pages/Playlists"));
const PlaylistPage = lazy(() => import("./pages/PlaylistPage"));
const ArtistPage   = lazy(() => import("./pages/ArtistPage"));
const AlbumPage    = lazy(() => import("./pages/AlbumPage"));
import { getCredentials, validateCredentials } from "./api/credentials";
import { getAuthStatus } from "./api/auth";
import {
  getVolume, setVolume, setMuted,
  pausePlayback, resumePlayback, stopPlayback, playTrack,
  warmupPlayback, preloadTrack, seekPlayback, retryPlayTrack,
} from "./api/playback";
import { updateNowPlaying, requestNotificationPermission, showTrackNotification, setDiscordEnabled } from "./api/media";
import { useCredentialsStore } from "./store/credentials.store";
import { useAuthStore } from "./store/auth.store";
import { useUIStore } from "./store/ui.store";
import { setWindowEffect } from "./api/window";
import { getRecommendations } from "./api/spotify";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { usePlayerStore } from "./store/player.store";
import { useQueueStore } from "./store/queue.store";
import { usePrefsStore } from "./store/prefs.store";
import { toast } from "./store/toast.store";
import { useInvalidateLibrary } from "./hooks/useLibrary";
import { startRadio } from "./utils/radio";
import { prefetchLyrics } from "./lib/prefetch";
import { lastfmNowPlaying, lastfmScrobble } from "./api/lastfm";
import { getCurrentWindow } from "@tauri-apps/api/window";

function AppInit() {
  const setFromCredentials = useCredentialsStore((s) => s.setFromCredentials);
  const setCredStatus      = useCredentialsStore((s) => s.setStatus);
  const setFromStatus      = useAuthStore((s) => s.setFromStatus);
  const isLoggedIn         = useAuthStore((s) => s.loggedIn);
  const onEvent            = usePlayerStore((s) => s.onEvent);
  const storeSetVolume     = usePlayerStore((s) => s.setVolume);
  const storeSetMuted      = usePlayerStore((s) => s.setMuted);
  const invalidateLibrary  = useInvalidateLibrary();
  const queryClient        = useQueryClient();
  const currentTrack       = usePlayerStore((s) => s.currentTrack);
  const upcoming           = useQueueStore((s) => s.queue);
  const lastTrackId        = useRef<string | null>(null);
  // tracks we already auto-retried on a fresh session after an `unavailable`
  // event, so a track that's genuinely dead can't loop forever. cleared per
  // track once it actually starts playing.


 // TO DO- on close cache the last song and on next startup play directly from the timestamp isntead of starting from 0
  const retriedUnavailable = useRef<Set<string>>(new Set());
  // last fm scrobble bookeeping for whatever's playing right now
  const scrobbleRef        = useRef<{
    artist: string; track: string; album: string;
    startedAt: number; durationMs: number; scrobbled: boolean;
  } | null>(null);

  // os notif permission
  useEffect(() => {
    requestNotificationPermission();
  }, []);

  // push the saved discord presence pref down to the rust media thread on launch because it defalts to enabled (so if sum1 turned it off last time, it respects
  useEffect(() => {
    setDiscordEnabled(usePrefsStore.getState().discordPresence);
  }, []);

  /* re assert the saved window material on mount (noddefault) a webview reload (HMR / dev ctrl+s) keeps the 
native backdrop but blows away react state, so sync hereto keep the os material matching the stored choice - otherwise it flashes
   back to a bright default. */
  useEffect(() => {
    const effect = useUIStore.getState().windowEffect;
    setWindowEffect(effect).catch(() => {});
  }, []);

  /*  TODO- kill the default webview rightclick menu (dev artifact). our own context
   menus go through react onContextMenu which still fires fine */

/* DONE */
  useEffect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", block);
    return () => document.removeEventListener("contextmenu", block);
  }, []);

  // startup: sync creds, auth status, volume
  useEffect(() => {
    /* load the creds, and then auto validate (helps unfetched data) and then cache */
    getCredentials().then((c) => {
      setFromCredentials(c);
      if (c && c.has_secret) {
        setCredStatus("validating");
        validateCredentials()
          .then((r) => setCredStatus(r.valid ? "valid" : "invalid"))
          .catch(() => setCredStatus("invalid"));
      }
    }).catch(() => {});
    getAuthStatus().then(setFromStatus).catch(() => {});

    const { volume, muted } = usePlayerStore.getState();
    setVolume(volume).catch(() => {});
    setMuted(muted).catch(() => {});
    getVolume().then((vs) => {
      storeSetVolume(vs.level);
      storeSetMuted(vs.muted);
    }).catch(() => {});
  }, [setFromCredentials, setCredStatus, setFromStatus, storeSetVolume, storeSetMuted]);

  /* pewarm the librespot session after login so the first play is instant, and prefetch 
Home recs so they're cached before the user gets there
   (fixes the recs being slow to show up sometimes) */

/* other use of caching  playlsits finish by 28 */
  useEffect(() => {
    if (isLoggedIn) {
      warmupPlayback().catch(() => {});
      queryClient.prefetchQuery({
        queryKey: ["recommendations", "home"],
        queryFn:  () => getRecommendations(undefined, 12),
        staleTime: 30 * 60_000,
      }).catch(() => {});
    }
  }, [isLoggedIn, queryClient]);

  /* load (warm) lyrics for the now playing track plus the next few queued ones, so
  opening the panel (or the song changing) paints INSTTANLy with no spinner
  reruns on track/queue change; prefetchLyrics (dedupes) */
  useEffect(() => {
    if (currentTrack) prefetchLyrics(queryClient, currentTrack);
    upcoming.slice(0, 3).forEach((t) => prefetchLyrics(queryClient, t));
  }, [currentTrack, upcoming, queryClient]);

  
  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onCloseRequested((e) => {
      const { promptOnClose } = usePrefsStore.getState();
      const { isPlaying }     = usePlayerStore.getState();
      if (promptOnClose && isPlaying) {
        e.preventDefault();
        useUIStore.getState().setQuitConfirmOpen(true);
      }
    });
    return () => { unlisten.then((u) => u()).catch(() => {}); };
  }, []);

  
  const maybeScrobble = useCallback(() => {
    const s = scrobbleRef.current;
    if (!s || s.scrobbled || s.durationMs < 30_000) return;
    const elapsed = Math.floor(Date.now() / 1000) - s.startedAt;
    const threshold = Math.min(s.durationMs / 2, 240_000) / 1000;
    if (elapsed >= threshold) {
      lastfmScrobble(s.artist, s.track, s.album, s.startedAt);
      s.scrobbled = true;
    }
  }, []);

  // player events: update os control when track changed -+ fire notification
  const handlePlayerEvent = useCallback((payload: unknown) => {
    onEvent(payload);
    const msg = payload as { type: string; track_id?: string | null };

    if (msg.type === "playing") {
      const { currentTrack } = usePlayerStore.getState();
      // it played so let it be autoretried again if it ever dies later
      if (msg.track_id) retriedUnavailable.current.delete(msg.track_id);
      if (currentTrack && currentTrack.id !== lastTrackId.current) {
        // new track started = scrobble the previous one if it earned it
        maybeScrobble();
        lastTrackId.current = currentTrack.id;
        updateNowPlaying(currentTrack, 0);
        // notification disabled can be from settings general (os fucks it up too, TODO fix)
        if (usePrefsStore.getState().notifyOnTrack) showTrackNotification(currentTrack);

        // last.fm: set nowplaying + scrobblekeeping for this track
        const artist = currentTrack.artists.map((a) => a.name).join(", ");
        const album  = currentTrack.album?.name ?? "";
        lastfmNowPlaying(artist, currentTrack.name, album);
        scrobbleRef.current = {
          artist, track: currentTrack.name, album,
          startedAt: Math.floor(Date.now() / 1000),
          durationMs: currentTrack.duration_ms,
          scrobbled: false,
        };
      }
    }

    if (msg.type === "unavailable") {
      /* librespot connected but the track didn't load. usually transient: a
       session when audio key has requests time out (in-session retries
       cant dig out, a fresh session can, try re BUILD) so if it's the CURRENT track and
       we haven't retried yet, reBUILD the session and try once more. only a
       real availability block (or a failure that survives the fresh session
       retry) actually shows the toast. */
      const { currentTrack } = usePlayerStore.getState();
      const tid = msg.track_id ?? currentTrack?.id ?? null;
      const isCurrent = !!tid && !!currentTrack && currentTrack.id === tid;
      if (isCurrent && tid && !retriedUnavailable.current.has(tid)) {
        retriedUnavailable.current.add(tid);
        retryPlayTrack(tid).catch(() => {
          toast("Can't play this track. Try another.");
        });
      } else if (isCurrent) {
        // already did the freshsession retry and it still died = really dead
        toast("Can't play this track. Try another.");
      }
      /* not the current track (failed preload, say) = ignore. never tear the
       session down while something else is playing. */
    }

    if (msg.type === "time_to_preload_next_track") {
      // preload the next queued track for (gapless) transitions TODO- DONE
      const { currentTrack } = usePlayerStore.getState();
      const next = useQueueStore.getState().peek(currentTrack);
      if (next) {
        preloadTrack(next.id).catch(() => {});
      }
    }

    if (msg.type === "end_of_track") {
      /* TODO review 233-246 ai code */
      maybeScrobble();
      const { currentTrack, setCurrentTrack } = usePlayerStore.getState();
      const next = useQueueStore.getState().advance(currentTrack);
      if (next) {
        setCurrentTrack(next);
        playTrack(next.id).catch(() => {});
      } else {
        // queue's empty (end of playlist / a single) -> kick off a rec radio
        // seeded from the track that just ended
        startRadio(currentTrack);
      }
    }

  /* DONE, FIX limiter fetching TODO - DONE */
  }, [onEvent, maybeScrobble]);

  // librespot player events
  useEffect(() => {
    const p = listen("player:event", (e) => handlePlayerEvent(e.payload));
    return () => { p.then((u) => u()).catch(() => {}); };
  }, [handlePlayerEvent]);

  // library sync event
  useEffect(() => {
    const p = listen("library:synced", () => invalidateLibrary());
    return () => { p.then((u) => u()).catch(() => {}); };
  }, [invalidateLibrary]);

  // os media control events (tray menu for now)
// nope
  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const reg = async () => {
      unlisteners.push(
        await listen("media:play",   () => resumePlayback().catch(() => {})),
        await listen("media:pause",  () => pausePlayback().catch(() => {})),
        await listen("media:stop",   () => stopPlayback().catch(() => {})),
        await listen("media:toggle", () => {
          const { isPlaying } = usePlayerStore.getState();
          if (isPlaying) pausePlayback().catch(() => {});
          else           resumePlayback().catch(() => {});
        }),
        await listen("media:next", () => {
          const { currentTrack, setCurrentTrack } = usePlayerStore.getState();
          const next = useQueueStore.getState().advance(currentTrack);
          if (next) { setCurrentTrack(next); playTrack(next.id).catch(() => {}); }
        }),
        await listen("media:prev", () => {
          const { currentTrack, positionMs, setCurrentTrack } = usePlayerStore.getState();
          if (positionMs > 3000) {
            seekPlayback(0).catch(() => {});
          } else {
            const prev = useQueueStore.getState().previous(currentTrack);
            if (prev) { setCurrentTrack(prev); playTrack(prev.id).catch(() => {}); }
          }
        }),
      );
    };

    reg().catch(() => {});
    return () => unlisteners.forEach((u) => u());
  }, []);

  return null;
}

export default function App() {
  return (
    <BrowserRouter>
      <AppInit />
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Home />} />
          {/* suspense fallback covers the quick lazy chunk fetch on first
              visit to a route --- later visits are instant (chunks cached) */}
          <Route path="settings"     element={<Suspense fallback={<Loader />}><Settings /></Suspense>} />
          <Route path="search"       element={<Suspense fallback={<Loader />}><Search /></Suspense>} />
          <Route path="radio"        element={<Suspense fallback={<Loader />}><Search /></Suspense>} />
          <Route path="library"      element={<Suspense fallback={<Loader />}><Library /></Suspense>} />
          <Route path="profile"      element={<Suspense fallback={<Loader />}><Profile /></Suspense>} />
          <Route path="playlist/:id" element={<Suspense fallback={<Loader />}><PlaylistPage /></Suspense>} />
          <Route path="playlists"    element={<Suspense fallback={<Loader />}><Playlists /></Suspense>} />
          <Route path="artist/:id"   element={<Suspense fallback={<Loader />}><ArtistPage /></Suspense>} />
          <Route path="album/:id"    element={<Suspense fallback={<Loader />}><AlbumPage /></Suspense>} />
          <Route path="*"            element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
