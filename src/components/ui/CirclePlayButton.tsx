import { motion } from "framer-motion";
import { AnimatedPlayPause } from "../playground/AnimatedIcons";
import { EASE_OUT, PRESS } from "@/lib/motion";

export function CirclePlayButton({
  isPlaying,
  visible = true,
  pending = false,
  onClick,
  size = 42,
  iconSize = 17,
  style,
  ariaLabel = "Play",
}: {
  isPlaying: boolean;
  visible?: boolean;
  // the click was heard and playback is being fetched. shown from the press
  // itself, so a slow network never reads as a dead button.
  pending?: boolean;
  onClick: (e: React.MouseEvent) => void;
  size?: number;
  iconSize?: number;
  style?: React.CSSProperties;
  ariaLabel?: string;
}) {
  return (
    <motion.button
      aria-label={ariaLabel}
      aria-busy={pending || undefined}
      className="focus-ring"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={visible ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.9 }}
      whileHover={{ scale: 1.03 }}
      whileTap={PRESS}
      transition={{ duration: 0.16, ease: EASE_OUT }}
      onClick={onClick}
      style={{
        position: "relative",
        width: size,
        height: size,
        borderRadius: 999,
        border: "none",
        background: "var(--color-accent)",
        color: "var(--color-accent-text, #ffffff)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        cursor: pending ? "progress" : "pointer",
        boxShadow: "0 4px 14px rgba(0,0,0,0.5)",
        pointerEvents: visible ? "auto" : "none",
        flexShrink: 0,
        ...style,
      }}
    >
      {pending && (
        <svg
          aria-hidden
          className="loader-arc"
          viewBox="0 0 50 50"
          style={{ position: "absolute", inset: -3, width: size + 6, height: size + 6, pointerEvents: "none" }}
        >
          <circle
            cx="25" cy="25" r="23.5" fill="none"
            stroke="rgba(255,255,255,0.85)" strokeWidth="2.2" strokeLinecap="round"
            strokeDasharray="36 200"
          />
        </svg>
      )}
      <AnimatedPlayPause isPlaying={isPlaying} size={iconSize} strokeWidth={2.4} />
    </motion.button>
  );
}
