import { useEffect, useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { usePlayerStore } from "../store/player.store";
import { useAuthStore } from "../store/auth.store";
import {
  getDevices,
  getPlaybackState,
  transferPlayback,
  remotePlayTrack,
  getMusiqueDeviceId,
} from "../api/connect";
import { pausePlayback, resumeOrPlay } from "../api/playback";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";

/* the backend forwards spotify's connect-cluster pushes as
"connect:cluster-changed". once one has arrived we know pushes work, so the
polling below backs off to a slow safety net. several components mount this
hook; the shared timestamp and listeners keep pushes and polls coalesced. */
let pushSeen = false;
let lastPushRefresh = 0;

let inflightDevices: Promise<void> | null = null;
let inflightPlayback: Promise<void> | null = null;
let inflightMusiqueId: Promise<void> | null = null;

const clusterListeners = new Set<() => void>();
let clusterUnlisten: (() => void) | null = null;
let clusterPending = false;

let coordinatorRefs = 0;
let coordinatorLastLoggedIn: boolean | null = null;
let coordinatorDevicesTimer: ReturnType<typeof setInterval> | null = null;
let coordinatorDevicesIntervalMs = 0;
let coordinatorPlaybackTimer: ReturnType<typeof setInterval> | null = null;
let coordinatorPlaybackIntervalMs = 0;
let coordinatorStoreUnsub: (() => void) | null = null;

function ensureMusiqueDeviceId() {
  const store = usePlayerStore.getState();
  if (store.musiqueDeviceId || inflightMusiqueId) return;
  inflightMusiqueId = getMusiqueDeviceId()
    .then((id) => {
      if (id) usePlayerStore.getState().setMusiqueDeviceId(id);
    })
    .catch(() => {})
    .finally(() => {
      inflightMusiqueId = null;
    });
}

function fetchDevicesShared(): Promise<void> {
  if (!useAuthStore.getState().loggedIn) return Promise.resolve();
  if (inflightDevices) return inflightDevices;
  const generation = useAuthStore.getState().generation;
  inflightDevices = (async () => {
    try {
      const payload = await getDevices();
      if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return;
      if (payload) {
        const st = usePlayerStore.getState();
        if (payload.musique_device_id && !st.musiqueDeviceId) {
          st.setMusiqueDeviceId(payload.musique_device_id);
        }
        st.setDevices(payload.devices ?? []);
        const currentActive = payload.devices?.find((d) => d.is_active) ?? null;
        if (currentActive) st.setActiveDevice(currentActive);
      }
    } catch {
      // Quietly ignore network/auth errors in background poll
    } finally {
      inflightDevices = null;
    }
  })();
  return inflightDevices;
}

function fetchPlaybackShared(): Promise<void> {
  if (!useAuthStore.getState().loggedIn) return Promise.resolve();
  if (inflightPlayback) return inflightPlayback;
  const generation = useAuthStore.getState().generation;
  inflightPlayback = (async () => {
    try {
      const state = await getPlaybackState();
      if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return;
      usePlayerStore.getState().syncRemotePlayback(state);
    } catch {
      // Quietly ignore
    } finally {
      inflightPlayback = null;
    }
  })();
  return inflightPlayback;
}

function syncPollingTimers() {
  if (coordinatorRefs === 0) {
    if (coordinatorDevicesTimer) {
      clearInterval(coordinatorDevicesTimer);
      coordinatorDevicesTimer = null;
      coordinatorDevicesIntervalMs = 0;
    }
    if (coordinatorPlaybackTimer) {
      clearInterval(coordinatorPlaybackTimer);
      coordinatorPlaybackTimer = null;
      coordinatorPlaybackIntervalMs = 0;
    }
    return;
  }

  const loggedIn = useAuthStore.getState().loggedIn;
  if (coordinatorLastLoggedIn !== loggedIn) {
    coordinatorLastLoggedIn = loggedIn;
    fetchDevicesShared();
    fetchPlaybackShared();
  }

  const { devicesOpen, isRemotePlayback, isPlaying } = usePlayerStore.getState();
  const desiredDevicesMs = loggedIn && devicesOpen ? 3000 : 0;
  if (desiredDevicesMs !== coordinatorDevicesIntervalMs) {
    if (coordinatorDevicesTimer) {
      clearInterval(coordinatorDevicesTimer);
      coordinatorDevicesTimer = null;
    }
    coordinatorDevicesIntervalMs = desiredDevicesMs;
    if (desiredDevicesMs > 0) {
      fetchDevicesShared();
      coordinatorDevicesTimer = setInterval(fetchDevicesShared, desiredDevicesMs);
    }
  }

  const isPlayingLocal = isPlaying && !isRemotePlayback;
  const desiredPlaybackMs = !loggedIn || (isPlayingLocal && !devicesOpen)
    ? 0
    : pushSeen
    ? (devicesOpen ? 8000 : isRemotePlayback ? 5000 : 30000)
    : (devicesOpen ? 2500 : isRemotePlayback ? 3000 : 10000);

  if (desiredPlaybackMs !== coordinatorPlaybackIntervalMs) {
    if (coordinatorPlaybackTimer) {
      clearInterval(coordinatorPlaybackTimer);
      coordinatorPlaybackTimer = null;
    }
    coordinatorPlaybackIntervalMs = desiredPlaybackMs;
    if (desiredPlaybackMs > 0) {
      coordinatorPlaybackTimer = setInterval(() => {
        if (typeof document !== "undefined" && document.visibilityState === "hidden") return;
        fetchPlaybackShared();
      }, desiredPlaybackMs);
    }
  }
}

function setupClusterListener() {
  if (clusterPending || clusterUnlisten) return;
  clusterPending = true;
  listen("connect:cluster-changed", () => {
    pushSeen = true;
    const now = Date.now();
    if (now - lastPushRefresh < 500) return;
    lastPushRefresh = now;
    clusterListeners.forEach((l) => l());
    syncPollingTimers();
  })
    .then((unlisten) => {
      clusterPending = false;
      if (clusterListeners.size === 0) {
        unlisten();
      } else {
        clusterUnlisten = unlisten;
      }
    })
    .catch(() => {
      clusterPending = false;
    });
}

export function useDevices() {
  const loggedIn = useAuthStore((s) => s.loggedIn);
  const devices = usePlayerStore((s) => s.devices);
  const activeDevice = usePlayerStore((s) => s.activeDevice);
  const musiqueDeviceId = usePlayerStore((s) => s.musiqueDeviceId);
  const isRemotePlayback = usePlayerStore((s) => s.isRemotePlayback);
  const devicesOpen = usePlayerStore((s) => s.devicesOpen);
  const toggleDevices = usePlayerStore((s) => s.toggleDevices);
  const setDevicesOpen = usePlayerStore((s) => s.setDevicesOpen);

  const [transferringId, setTransferringId] = useState<string | null>(null);

  const refreshDevices = useCallback(() => fetchDevicesShared(), []);
  const refreshPlayback = useCallback(() => fetchPlaybackShared(), []);

  useEffect(() => {
    coordinatorRefs++;
    ensureMusiqueDeviceId();
    if (!coordinatorStoreUnsub) {
      coordinatorStoreUnsub = usePlayerStore.subscribe((next, prev) => {
        if (
          next.devicesOpen !== prev.devicesOpen ||
          next.isRemotePlayback !== prev.isRemotePlayback ||
          next.isPlaying !== prev.isPlaying
        ) {
          syncPollingTimers();
        }
      });
    }
    const onCluster = () => {
      fetchDevicesShared();
      fetchPlaybackShared();
    };
    clusterListeners.add(onCluster);
    setupClusterListener();
    syncPollingTimers();

    return () => {
      coordinatorRefs--;
      clusterListeners.delete(onCluster);
      if (clusterListeners.size === 0 && clusterUnlisten) {
        clusterUnlisten();
        clusterUnlisten = null;
      }
      if (coordinatorRefs === 0) {
        coordinatorLastLoggedIn = null;
        if (coordinatorStoreUnsub) {
          coordinatorStoreUnsub();
          coordinatorStoreUnsub = null;
        }
      }
      syncPollingTimers();
    };
  }, []);

  useEffect(() => {
    syncPollingTimers();
  }, [loggedIn]);

  const transfer = useCallback(
    async (deviceId: string) => {
      setTransferringId(deviceId);
      try {
        const store = usePlayerStore.getState();
        const targetDevice = store.devices.find((d) => d.id === deviceId);
        const isMusique =
          Boolean(store.musiqueDeviceId && deviceId === store.musiqueDeviceId) ||
          targetDevice?.name.toLowerCase() === "musique";

        if (isMusique) {
          // Transferring to THIS computer
          const track = store.currentTrack;
          const pos = Math.max(0, Math.floor(store.positionMs || 0));
          store.setIsRemotePlayback(false);
          store.setTargetState("playing");
          store.setPlaying(true);
          store.setPosition(pos);
          store.setLastPlayingAt(Date.now());

          if (track) {
            // Instantly start local playback at the exact same position without starting over
            await resumeOrPlay(track.id, pos).catch((e) => {
              console.error("[useDevices] resumeOrPlay error:", e);
            });
          } else {
            // Transfer device if no track was loaded
            await transferPlayback(deviceId, true).catch(() => {});
          }
        } else {
          // Transferring from Musique to a remote device (phone, etc.)
          const track = store.currentTrack;
          const pos = Math.max(0, Math.floor(store.positionMs || 0));
          store.setIsRemotePlayback(true);
          await pausePlayback().catch(() => {});
          await remotePlayTrack(deviceId, track?.id, pos);
        }

        setTimeout(() => {
          refreshPlayback();
          refreshDevices();
        }, 1500);
        setTimeout(() => {
          refreshPlayback();
          refreshDevices();
        }, 3500);
      } catch (e) {
        toast.error(`Could not transfer playback: ${errMsg(e)}`);
      } finally {
        setTransferringId(null);
      }
    },
    [refreshPlayback, refreshDevices]
  );

  return {
    devices,
    activeDevice,
    musiqueDeviceId,
    isRemotePlayback,
    devicesOpen,
    toggleDevices,
    setDevicesOpen,
    transferringId,
    refreshDevices,
    refreshPlayback,
    transfer,
  };
}
