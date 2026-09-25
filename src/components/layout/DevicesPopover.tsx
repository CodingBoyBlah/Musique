import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
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
import { gpuLayer, zTransform } from "../../lib/motion";

function getDeviceIcon(type: string) {
  const t = type.toLowerCase();
  if (t === "smartphone") return Smartphone;
  if (t === "computer") return Laptop;
  if (t === "speaker") return Speaker;
  if (t === "tv") return Tv;
  return Cast;
}

export function DevicesPopover() {
  const {
    devices,
    activeDevice,
    musiqueDeviceId,
    devicesOpen,
    setDevicesOpen,
    transfer,
    transferringId,
    refreshDevices,
    isRemotePlayback,
  } = useDevices();

  const popoverRef = useRef<HTMLDivElement>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [pos, setPos] = useState<{ top?: number; bottom?: number; right: number }>({ top: 48, right: 16 });

  useEffect(() => {
    if (!devicesOpen) return;
    const updatePos = () => {
      const triggers = document.querySelectorAll("[data-devices-trigger]");
      let activeTrigger: Element | null = null;
      for (const t of triggers) {
        const r = t.getBoundingClientRect();
        if (r.width > 0 && r.height > 0) {
          activeTrigger = t;
          break;
        }
      }
      if (activeTrigger) {
        const rect = activeTrigger.getBoundingClientRect();
        const right = Math.max(12, window.innerWidth - rect.right);
        if (rect.top > window.innerHeight / 2) {
          setPos({ bottom: Math.max(16, window.innerHeight - rect.top + 8), right });
        } else {
          setPos({ top: Math.max(12, rect.bottom + 8), right });
        }
      }
    };
    updatePos();
    window.addEventListener("resize", updatePos);
    return () => window.removeEventListener("resize", updatePos);
  }, [devicesOpen]);

  // Close on outside pointer click (ignoring trigger clicks)
  useEffect(() => {
    if (!devicesOpen) return;
    function onPointerDown(e: MouseEvent) {
      const isTrigger = Boolean((e.target as Element)?.closest?.("[data-devices-trigger]"));
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node) && !isTrigger) {
        setDevicesOpen(false);
      }
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        setDevicesOpen(false);
      }
    }
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [devicesOpen, setDevicesOpen]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshDevices();
    } finally {
      setTimeout(() => setIsRefreshing(false), 450);
    }
  };

  if (!devicesOpen) return null;

  return (
    <motion.div
      ref={popoverRef}
      initial={{ opacity: 0, y: pos.top !== undefined ? -8 : 8, scale: 0.96 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: pos.top !== undefined ? -6 : 6, scale: 0.96 }}
      transition={{ type: "spring", stiffness: 440, damping: 28 }}
      transformTemplate={zTransform}
      style={{
        ...gpuLayer,
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
          style={{
            fontSize: 12.5,
            fontWeight: 600,
            color: "var(--color-text-hi)",
            letterSpacing: "-0.01em",
          }}
        >
          Devices
        </span>
        <div style={{ display: "flex", alignItems: "center", gap: 3 }}>
          <button
            type="button"
            onClick={handleRefresh}
            title="Refresh"
            style={{
              background: "transparent",
              border: "none",
              color: "var(--color-text-dim)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: 24,
              height: 24,
              borderRadius: 5,
              transition: "color 0.12s, background 0.12s",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)";
              (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-hover)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)";
              (e.currentTarget as HTMLButtonElement).style.background = "transparent";
            }}
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
            onClick={() => setDevicesOpen(false)}
            title="Close"
            style={{
              background: "transparent",
              border: "none",
              color: "var(--color-text-dim)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: 24,
              height: 24,
              borderRadius: 5,
              transition: "color 0.12s, background 0.12s",
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-hi)";
              (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-hover)";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-dim)";
              (e.currentTarget as HTMLButtonElement).style.background = "transparent";
            }}
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
            style={{
              padding: "24px 12px",
              textAlign: "center",
              fontSize: 12,
              color: "var(--color-text-dim)",
            }}
          >
            No devices found
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
              <motion.button
                key={device.id ?? device.name}
                type="button"
                onClick={() => {
                  if (device.id && !isActive && !isTransferring) {
                    transfer(device.id);
                  }
                }}
                disabled={isTransferring || isActive}
                whileHover={!isActive ? { scale: 1.01 } : {}}
                whileTap={!isActive ? { scale: 0.98 } : {}}
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: 10,
                  width: "100%",
                  padding: "8px 10px",
                  borderRadius: 8,
                  border: "none",
                  background: isActive
                    ? "var(--color-surface-2)"
                    : "transparent",
                  cursor: isActive ? "default" : "pointer",
                  textAlign: "left",
                  transition: "background 0.12s",
                }}
                onMouseEnter={(e) => {
                  if (!isActive) {
                    (e.currentTarget as HTMLButtonElement).style.background =
                      "var(--color-surface-hover)";
                  }
                }}
                onMouseLeave={(e) => {
                  if (!isActive) {
                    (e.currentTarget as HTMLButtonElement).style.background =
                      "transparent";
                  }
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
              </motion.button>
            );
          })
        )}
      </div>
    </motion.div>
  );
}
