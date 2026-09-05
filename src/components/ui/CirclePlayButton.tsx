import { motion } from "framer-motion";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";
import { useUIStore } from "../../store/ui.store";

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
  const fastMode = useUIStore((s) => s.fastMode);

  return (
    <motion.button
      aria-label={ariaLabel}
      initial={fastMode ? false : { opacity: 0, scale: 0.8 }}
      animate={visible ? { opacity: 1, scale: 1 } : { opacity: 0, scale: fastMode ? 1 : 0.8 }}
      whileHover={fastMode ? undefined : { scale: 1.08 }}
      whileTap={fastMode ? undefined : { scale: 0.92 }}
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
