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

const clusterListeners = new Set<() => void>();
let clusterUnlisten: (() => void) | null = null;
let clusterPending = false;

function setupClusterListener() {
  if (clusterPending || clusterUnlisten) return;
  clusterPending = true;
  listen("connect:cluster-changed", () => {
    pushSeen = true;
    const now = Date.now();
    if (now - lastPushRefresh < 500) return;
    lastPushRefresh = now;
    clusterListeners.forEach((l) => l());
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
  const setDevices = usePlayerStore((s) => s.setDevices);
  const activeDevice = usePlayerStore((s) => s.activeDevice);
  const setActiveDevice = usePlayerStore((s) => s.setActiveDevice);
  const musiqueDeviceId = usePlayerStore((s) => s.musiqueDeviceId);
  const setMusiqueDeviceId = usePlayerStore((s) => s.setMusiqueDeviceId);
  const isRemotePlayback = usePlayerStore((s) => s.isRemotePlayback);
  const devicesOpen = usePlayerStore((s) => s.devicesOpen);
  const toggleDevices = usePlayerStore((s) => s.toggleDevices);
  const setDevicesOpen = usePlayerStore((s) => s.setDevicesOpen);
  const syncRemotePlayback = usePlayerStore((s) => s.syncRemotePlayback);

  const [transferringId, setTransferringId] = useState<string | null>(null);

  // Initialize Musique device ID once
  useEffect(() => {
    if (!musiqueDeviceId) {
      getMusiqueDeviceId()
        .then((id) => {
          if (id) setMusiqueDeviceId(id);
        })
        .catch(() => {});
    }
  }, [musiqueDeviceId, setMusiqueDeviceId]);

  const refreshDevices = useCallback(async () => {
    if (!useAuthStore.getState().loggedIn) return;
    const generation = useAuthStore.getState().generation;
    if (inflightDevices) return inflightDevices;
    inflightDevices = (async () => {
      try {
        const payload = await getDevices();
        if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return;
        if (payload) {
          if (payload.musique_device_id && !usePlayerStore.getState().musiqueDeviceId) {
            setMusiqueDeviceId(payload.musique_device_id);
          }
          setDevices(payload.devices ?? []);
          const currentActive = payload.devices?.find((d) => d.is_active) ?? null;
          if (currentActive) {
            setActiveDevice(currentActive);
          }
        }
      } catch {
        // Quietly ignore network/auth errors in background poll
      } finally {
        inflightDevices = null;
      }
    })();
    return inflightDevices;
  }, [setDevices, setActiveDevice, setMusiqueDeviceId]);

  const refreshPlayback = useCallback(async () => {
    if (!useAuthStore.getState().loggedIn) return;
    const generation = useAuthStore.getState().generation;
    if (inflightPlayback) return inflightPlayback;
    inflightPlayback = (async () => {
      try {
        const state = await getPlaybackState();
        if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return;
        syncRemotePlayback(state);
      } catch {
        // Quietly ignore
      } finally {
        inflightPlayback = null;
      }
    })();
    return inflightPlayback;
  }, [syncRemotePlayback]);

  // Initial load on mount
  useEffect(() => {
    refreshDevices();
    refreshPlayback();
  }, [loggedIn, refreshDevices, refreshPlayback]);

  // live device/playback changes pushed over the dealer
  const [pushLive, setPushLive] = useState(pushSeen);
  useEffect(() => {
    const onCluster = () => {
      setPushLive(true);
      refreshDevices();
      refreshPlayback();
    };
    clusterListeners.add(onCluster);
    setupClusterListener();

    return () => {
      clusterListeners.delete(onCluster);
      if (clusterListeners.size === 0 && clusterUnlisten) {
        clusterUnlisten();
        clusterUnlisten = null;
      }
    };
  }, [refreshDevices, refreshPlayback]);

  // Devices panel polling: faster when panel is open
  useEffect(() => {
    if (!devicesOpen) return;
    refreshDevices();
    const timer = setInterval(refreshDevices, 3000);
    return () => clearInterval(timer);
  }, [devicesOpen, refreshDevices]);

  // Playback sync polling:
  // - 2500ms when devices panel is open
  // - 3000ms when remote playback is active
  // - 10000ms idle heartbeat when not playing locally, to detect remote playback started elsewhere
  useEffect(() => {
    const store = usePlayerStore.getState();
    const isPlayingLocal = store.isPlaying && !isRemotePlayback;
    if (isPlayingLocal && !devicesOpen) return;

    // with pushes flowing, polls only keep the remote scrubber honest and
    // catch anything a push missed
    const intervalMs = pushLive
      ? devicesOpen ? 8000 : isRemotePlayback ? 5000 : 30000
      : devicesOpen ? 2500 : isRemotePlayback ? 3000 : 10000;
    const timer = setInterval(() => {
      if (typeof document !== "undefined" && document.visibilityState === "hidden") return;
      refreshPlayback();
    }, intervalMs);
    return () => clearInterval(timer);
  }, [isRemotePlayback, devicesOpen, refreshPlayback, pushLive]);

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
