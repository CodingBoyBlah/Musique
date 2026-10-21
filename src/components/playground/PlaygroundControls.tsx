import { motion, AnimatePresence } from "framer-motion";
import { AnimatedPlayPause } from "../ui/AnimatedIcons";
import "../../styles/ui.css";

// 1. Primary Glow Button (Fully rounded pill, matches Album Page play button, animated play/pause icon)
export function PrimaryPlayButton({
  playing = false,
  onToggle,
  label = "Play",
  size = "md",
}: {
  playing?: boolean;
  onToggle?: () => void;
  label?: string;
  size?: "sm" | "md" | "lg";
}) {
  const sizeStyles = {
    sm: { height: 36, padding: "0 18px", fontSize: 13, iconSize: 14, gap: 6 },
    md: { height: 44, padding: "0 24px", fontSize: 14, iconSize: 16, gap: 8 },
    lg: { height: 50, padding: "0 30px", fontSize: 15, iconSize: 18, gap: 9 },
  }[size];

  return (
    <motion.button
      whileHover={{ scale: 1.04 }}
      whileTap={{ scale: 0.96 }}
      onClick={onToggle}
      title={playing ? "Pause" : "Play"}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        height: sizeStyles.height,
        padding: sizeStyles.padding,
        gap: sizeStyles.gap,
        borderRadius: 99, // fully rounded pill
        border: "none",
        background: "var(--color-accent)",
        color: "var(--color-accent-text)",
        fontSize: sizeStyles.fontSize,
        fontWeight: 700,
        letterSpacing: "-0.01em",
        cursor: "pointer",
        boxShadow: playing
          ? "0 0 0 4px var(--color-accent-dim), 0 4px 18px var(--color-accent-dim)"
          : "0 4px 18px -2px var(--color-accent-dim)",
        transition: "box-shadow 0.2s, background 0.15s",
        outline: "none",
        userSelect: "none",
      }}
    >
      <AnimatedPlayPause isPlaying={playing} size={sizeStyles.iconSize} strokeWidth={0} />
      <span>{playing ? "Pause" : label}</span>
    </motion.button>
  );
}

// 2. Circular Pulse Play Button (KEPT - user loved this, added blur+scale icon animation)
export function PulsePlayButton({
  playing,
  onToggle,
  size = 56,
}: {
  playing: boolean;
  onToggle: () => void;
  size?: number;
}) {
  return (
    <div style={{ position: "relative", width: size, height: size, display: "inline-flex", alignItems: "center", justifyContent: "center" }}>
      <AnimatePresence>
        {playing && (
          <motion.div
            initial={{ scale: 0.85, opacity: 0.7 }}
            animate={{ scale: 1.5, opacity: 0 }}
            exit={{ opacity: 0 }}
            transition={{ repeat: Infinity, duration: 1.8, ease: "easeOut" }}
            style={{
              position: "absolute",
              inset: 0,
              borderRadius: 999,
              border: "2px solid var(--color-accent)",
              pointerEvents: "none",
            }}
          />
        )}
      </AnimatePresence>

      <motion.button
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.94 }}
        onClick={onToggle}
        title={playing ? "Pause" : "Play"}
        style={{
          position: "relative",
          zIndex: 1,
          width: size,
          height: size,
          borderRadius: 999,
          border: "none",
          background: "var(--color-accent)",
          color: "var(--color-accent-text)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          boxShadow: "0 8px 24px -4px var(--color-accent-dim)",
          outline: "none",
        }}
      >
        <AnimatedPlayPause isPlaying={playing} size={Math.round(size * 0.42)} strokeWidth={0} />
      </motion.button>
    </div>
  );
}

export { SegmentedControl } from "../ui/SegmentedControl";
