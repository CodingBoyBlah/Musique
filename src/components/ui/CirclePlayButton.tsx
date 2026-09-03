import { motion } from "framer-motion";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";

export function CirclePlayButton({
  isPlaying,
  visible = true,
  onClick,
  size = 42,
  iconSize = 17,
  style,
  ariaLabel = "Play",
}: {
  isPlaying: boolean;
  visible?: boolean;
  onClick: (e: React.MouseEvent) => void;
  size?: number;
  iconSize?: number;
  style?: React.CSSProperties;
  ariaLabel?: string;
}) {
  return (
    <motion.button
      aria-label={ariaLabel}
      initial={{ opacity: 0, scale: 0.8 }}
      animate={visible ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.8 }}
      whileHover={{ scale: 1.08 }}
      whileTap={{ scale: 0.92 }}
      transition={{ duration: 0.18, ease: "easeOut" }}
      onClick={onClick}
      style={{
        width: size,
        height: size,
        borderRadius: 999,
        border: "none",
        background: "var(--color-accent)",
        color: "var(--color-accent-text, #ffffff)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: "pointer",
        boxShadow: "0 4px 14px rgba(0,0,0,0.5)",
        pointerEvents: visible ? "auto" : "none",
        flexShrink: 0,
        outline: "none",
        ...style,
      }}
    >
      <AnimatedPlayPause isPlaying={isPlaying} size={iconSize} strokeWidth={2.4} />
    </motion.button>
  );
}
