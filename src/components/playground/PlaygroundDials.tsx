import { useState } from "react";
import { motion } from "framer-motion";
import { AnimatedVolumeIcon } from "./AnimatedIcons";

// 1. Spring Toggle (KEPT - user explicitly approved!)
export function SpringToggle({
  label = "Lossless Audio Streaming",
  initial = true,
}: {
  label?: string;
  initial?: boolean;
}) {
  const [on, setOn] = useState(initial);

  return (
    <div
      onClick={() => setOn(!on)}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 16,
        padding: "10px 16px",
        borderRadius: 12,
        background: "var(--color-surface)",
        border: "1px solid var(--color-border)",
        cursor: "pointer",
        minWidth: 280,
      }}
    >
      <span style={{ fontSize: 13, fontWeight: 600, color: on ? "var(--color-text-hi)" : "var(--color-text-dim)" }}>
        {label}
      </span>

      <div
        style={{
          width: 44,
          height: 24,
          borderRadius: 99,
          background: on ? "var(--color-accent)" : "rgba(255, 255, 255, 0.12)",
          padding: 2,
          display: "flex",
          alignItems: "center",
          justifyContent: on ? "flex-end" : "flex-start",
          transition: "background 0.2s",
        }}
      >
        <motion.div
          layout
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
          style={{
            width: 20,
            height: 20,
            borderRadius: 99,
            background: "#ffffff",
            boxShadow: "0 2px 5px rgba(0,0,0,0.3)",
          }}
        />
      </div>
    </div>
  );
}

// 2. Exact PlayerBar Volume Slider with Mute Blur+Scale Animation
export function PlayerBarVolumeSlider({
  initial = 75,
}: {
  initial?: number;
}) {
  const [volume, setVolume] = useState(initial);
  const [muted, setMuted] = useState(false);

  const handleMuteToggle = () => {
    setMuted(!muted);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const next = Number(e.target.value);
    setVolume(next);
    if (muted && next > 0) setMuted(false);
  };

  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 8,
        padding: "10px 16px",
        borderRadius: 12,
        background: "var(--color-surface)",
        border: "1px solid var(--color-border)",
      }}
    >
      {/* Mute icon button with exact blur+scale animation */}
      <motion.button
        onClick={handleMuteToggle}
        whileHover={{ scale: 1.10 }}
        whileTap={{ scale: 0.88 }}
        transition={{ type: "spring", stiffness: 450, damping: 22 }}
        title={muted ? "Unmute" : "Mute"}
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          width: 28,
          height: 28,
          borderRadius: "50%",
          border: "none",
          background: "transparent",
          color: muted ? "var(--color-accent)" : "var(--color-text)",
          cursor: "pointer",
          flexShrink: 0,
          padding: 0,
        }}
      >
        <AnimatedVolumeIcon muted={muted} volume={volume} size={14} />
      </motion.button>

      {/* Exact range input from PlayerBar.tsx with class 'vol' and CSS variable '--vol' */}
      <input
        className="vol"
        type="range"
        min={0}
        max={100}
        value={muted ? 0 : volume}
        onChange={handleVolumeChange}
        onMouseDown={() => {
          if (muted) setMuted(false);
        }}
        style={{
          width: 84,
          ["--vol" as string]: `${muted ? 0 : volume}%`,
        } as React.CSSProperties}
      />

      <span style={{ fontSize: 11, color: "var(--color-text-dim)", width: 34, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>
        {muted ? "Muted" : `${volume}%`}
      </span>
    </div>
  );
}
