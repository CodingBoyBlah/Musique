import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Smartphone,
  Laptop,
  Speaker,
  Tv,
  Cast,
  Loader2,
  Volume2,
  RefreshCw,
  X,
} from "@/lib/icons";
import { useDevices } from "../../hooks/useDevices";
import type { SpotifyDevice } from "../../api/connect";
import { gpuLayer, zTransform, EASE_OUT } from "../../lib/motion";

function getDeviceIcon(type: string) {
  const t = type.toLowerCase();
  if (t === "smartphone") return Smartphone;
  if (t === "computer") return Laptop;
  if (t === "speaker") return Speaker;
  if (t === "tv") return Tv;
  return Cast;
}

type PopoverPos = { top?: number; bottom?: number; right: number };

// the visible trigger (title bar button, or the player-bar "playing on" banner)
function findTrigger(): HTMLElement | null {
  const triggers = document.querySelectorAll<HTMLElement>("[data-devices-trigger]");
  for (const t of triggers) {
    const r = t.getBoundingClientRect();
    if (r.width > 0 && r.height > 0) return t;
  }
  return null;
}

// below a trigger in the top half of the window, above one in the bottom half
function measurePos(): PopoverPos {
  const trigger = findTrigger();
  if (!trigger) return { top: 48, right: 16 };
  const rect = trigger.getBoundingClientRect();
  const right = Math.max(12, window.innerWidth - rect.right);
  if (rect.top > window.innerHeight / 2) {
    return { bottom: Math.max(16, window.innerHeight - rect.top + 8), right };
  }
  return { top: Math.max(12, rect.bottom + 8), right };
}

/* Outer shell: owns presence, so closing actually plays the exit instead of
   the popover being unmounted out from under its own animation. It is also
   the one always-mounted useDevices() - that hook runs the device/playback
   polling, so the card below takes its data as props rather than starting a
   second set of pollers. */
export function DevicesPopover() {
  const devicesState = useDevices();
  return (
    <AnimatePresence>
      {devicesState.devicesOpen && <DevicesPopoverCard key="devices" {...devicesState} />}
    </AnimatePresence>
  );
}

function DevicesPopoverCard({
  devices,
  activeDevice,
  musiqueDeviceId,
  setDevicesOpen,
  transfer,
  transferringId,
  refreshDevices,
  isRemotePlayback,
}: ReturnType<typeof useDevices>) {

  const popoverRef = useRef<HTMLDivElement>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  // measured on the render that opens it, so the first painted frame is
  // already in place (it used to paint at a default spot, then jump) and the
  // slide direction is known before the enter animation starts
  const [pos, setPos] = useState<PopoverPos>(measurePos);
  // whatever had focus when it opened (normally the trigger) gets it back
  const returnFocus = useRef<HTMLElement | null>(
    document.activeElement instanceof HTMLElement ? document.activeElement : null,
  );

  useLayoutEffect(() => {
    const updatePos = () => setPos(measurePos());
    updatePos();
    window.addEventListener("resize", updatePos);
    return () => window.removeEventListener("resize", updatePos);
  }, []);

  const close = (restoreFocus: boolean) => {
    setDevicesOpen(false);
    if (restoreFocus) (returnFocus.current ?? findTrigger())?.focus?.();
  };

  // Close on outside pointer click (ignoring trigger clicks), Escape closes
  // and hands focus back to the trigger
  useEffect(() => {
    function onPointerDown(e: MouseEvent) {
      const isTrigger = Boolean((e.target as Element)?.closest?.("[data-devices-trigger]"));
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node) && !isTrigger) {
        setDevicesOpen(false);
      }
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.stopPropagation();
        setDevicesOpen(false);
        (returnFocus.current ?? findTrigger())?.focus?.();
      }
    }
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [setDevicesOpen]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshDevices();
    } finally {
      // long enough to register as "it refreshed", no longer
      setTimeout(() => setIsRefreshing(false), 250);
    }
  };

  const below = pos.top !== undefined;

  return (
    <motion.div
      ref={popoverRef}
      role="dialog"
      aria-label="Devices"
      className="glass-solid-fallback"
      // grows out of its trigger: from the top-right corner when it hangs
      // below the title-bar button, bottom-right when it rises over the
      // player bar. no overshoot - nothing was flicked.
      initial={{ opacity: 0, y: below ? -6 : 6, scale: 0.96 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: below ? -4 : 4, scale: 0.97, transition: { duration: 0.12, ease: EASE_OUT } }}
      transition={{ duration: 0.18, ease: EASE_OUT }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
        transformOrigin: below ? "top right" : "bottom right",
        position: "fixed",
        ...(pos.top !== undefined ? { top: pos.top } : {}),
        ...(pos.bottom !== undefined ? { bottom: pos.bottom } : {}),
        right: pos.right,
        width: 290,
        maxHeight: 380,
        display: "flex",
        flexDirection: "column",
        background: "rgba(18, 18, 22, 0.96)",
        backdropFilter: "blur(32px)",
        WebkitBackdropFilter: "blur(32px)",
        border: "1px solid var(--color-glass-border)",
        borderRadius: 16,
        boxShadow: "0 18px 48px rgba(0, 0, 0, 0.65), 0 2px 10px rgba(0, 0, 0, 0.35)",
        zIndex: 9999,
        overflow: "hidden",
        userSelect: "none",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "11px 14px",
          borderBottom: "1px solid var(--color-divider)",
        }}
      >
        <span
          className="t-caption"
          style={{
            fontSize: 12.5,
            fontWeight: 600,
            color: "var(--color-text-hi)",
          }}
        >
          Devices
        </span>
        <div style={{ display: "flex", alignItems: "center", gap: 3 }}>
          <button
            type="button"
            onClick={handleRefresh}
            title="Refresh"
            aria-label="Refresh devices"
            className="btn-icon"
            style={{ width: 24, height: 24, borderRadius: 6 }}
          >
            <RefreshCw
              size={12}
              style={{
                animation: isRefreshing ? "spin 0.7s linear infinite" : "none",
              }}
            />
          </button>
          <button
            type="button"
            onClick={() => close(true)}
            title="Close"
            aria-label="Close devices"
            className="btn-icon"
            style={{ width: 24, height: 24, borderRadius: 6 }}
          >
            <X size={13} />
          </button>
        </div>
      </div>

      {/* Device List */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 2,
          padding: 6,
          maxHeight: 280,
          overflowY: "auto",
        }}
      >
        {devices.length === 0 ? (
          <div
            className="t-caption"
            style={{
              padding: "24px 12px",
              textAlign: "center",
              fontSize: 12,
              color: "var(--color-text-dim)",
            }}
          >
            No devices found. Open Spotify on another device, then refresh.
          </div>
        ) : (
          devices.map((device: SpotifyDevice) => {
            const Icon = getDeviceIcon(device.type);
            const isMusiqueDevice =
              Boolean(musiqueDeviceId && device.id === musiqueDeviceId) ||
              device.name.toLowerCase() === "musique";
            const isActive = isRemotePlayback
              ? Boolean(activeDevice?.id ? activeDevice.id === device.id : device.is_active)
              : isMusiqueDevice;
            const isTransferring = transferringId === device.id;

            return (
              <button
                key={device.id ?? device.name}
                type="button"
                className="dev-row"
                data-active={isActive || undefined}
                aria-current={isActive || undefined}
                aria-busy={isTransferring || undefined}
                onClick={() => {
                  if (device.id && !isActive && !isTransferring) {
                    transfer(device.id);
                  }
                }}
                disabled={isTransferring || isActive}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: 10,
                  width: "100%",
                  padding: "8px 10px",
                  border: "none",
                  background: isActive
                    ? "var(--color-surface-2)"
                    : "transparent",
                  cursor: isActive ? "default" : "pointer",
                  textAlign: "left",
                  font: "inherit",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: 10, minWidth: 0 }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      width: 28,
                      height: 28,
                      borderRadius: 6,
                      background: isActive
                        ? "rgba(255, 255, 255, 0.1)"
                        : "var(--color-glass)",
                      color: isActive ? "var(--color-text-hi)" : "var(--color-text-dim)",
                      flexShrink: 0,
                    }}
                  >
                    <Icon size={14} active={isActive} />
                  </div>
                  <div style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
                    <span
                      className="t-caption"
                      style={{
                        fontSize: 12.5,
                        fontWeight: isActive ? 600 : 500,
                        color: "var(--color-text-hi)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {device.name}
                      {isMusiqueDevice && " (This Computer)"}
                    </span>
                    {isActive && (
                      <span
                        className="t-caption"
                        style={{
                          fontSize: 10.5,
                          color: "var(--color-text-dim)",
                          display: "flex",
                          alignItems: "center",
                          gap: 4,
                        }}
                      >
                        <Volume2 size={10} /> Active
                      </span>
                    )}
                  </div>
                </div>

                <div style={{ flexShrink: 0, marginLeft: 8 }}>
                  {isTransferring ? (
                    <Loader2
                      size={13}
                      style={{
                        color: "var(--color-text-dim)",
                        animation: "spin 1s linear infinite",
                      }}
                    />
                  ) : isActive ? (
                    <span
                      style={{
                        display: "inline-block",
                        width: 6,
                        height: 6,
                        borderRadius: "50%",
                        background: "var(--color-text-hi)",
                        boxShadow: "0 0 6px rgba(255, 255, 255, 0.5)",
                      }}
                    />
                  ) : null}
                </div>
              </button>
            );
          })
        )}
      </div>
    </motion.div>
  );
}
