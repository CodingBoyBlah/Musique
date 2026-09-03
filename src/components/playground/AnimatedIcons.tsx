import { motion, AnimatePresence } from "framer-motion";
import { Play, Pause, Volume2, Volume1, VolumeX, Heart } from "lucide-react";

export function AnimatedPlayPause({
  isPlaying,
  size = 16,
  strokeWidth = 2.4,
  fill = "currentColor",
  style,
}: {
  isPlaying: boolean;
  size?: number;
  strokeWidth?: number;
  fill?: string;
  style?: React.CSSProperties;
}) {
  const blurVal = size <= 16 ? "2.5px" : "4.5px";
  const scaleVal = size <= 16 ? 0.72 : 0.6;
  const duration = 0.25;

  return (
    <span
      style={{
        position: "relative",
        width: size,
        height: size,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0,
        ...style,
      }}
    >
      <AnimatePresence initial={false}>
        <motion.span
          key={isPlaying ? "pause" : "play"}
          initial={{ opacity: 0.1, scale: scaleVal, filter: `blur(${blurVal})` }}
          animate={{ opacity: 1,   scale: 1,        filter: "blur(0px)" }}
          exit={{    opacity: 0,   scale: scaleVal, filter: `blur(${blurVal})` }}
          transition={{ duration, ease: [0.22, 1, 0.36, 1] }}
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            willChange: "transform, filter, opacity",
            backfaceVisibility: "hidden",
          }}
        >
          {isPlaying ? (
            <Pause size={size} strokeWidth={strokeWidth} fill={fill} />
          ) : (
            <Play size={size} strokeWidth={strokeWidth} fill={fill} style={{ marginLeft: 1.5 }} />
          )}
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

export function AnimatedVolumeIcon({
  muted,
  volume,
  size = 14,
}: {
  muted: boolean;
  volume: number;
  size?: number;
}) {
  const Icon = (muted || volume === 0) ? VolumeX : volume < 50 ? Volume1 : Volume2;
  const key = muted || volume === 0 ? "muted" : volume < 50 ? "low" : "high";
  const blurVal = size <= 16 ? "2.5px" : "4px";
  const scaleVal = size <= 16 ? 0.72 : 0.6;

  return (
    <span style={{ position: "relative", width: size, height: size, display: "inline-flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
      <AnimatePresence initial={false}>
        <motion.span
          key={key}
          initial={{ opacity: 0.1, scale: scaleVal, filter: `blur(${blurVal})` }}
          animate={{ opacity: 1,   scale: 1,        filter: "blur(0px)" }}
          exit={{    opacity: 0,   scale: scaleVal, filter: `blur(${blurVal})` }}
          transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            willChange: "transform, filter, opacity",
            backfaceVisibility: "hidden",
          }}
        >
          <Icon size={size} strokeWidth={2} fill="currentColor" />
        </motion.span>
      </AnimatePresence>
    </span>
  );
}

export function AnimatedHeart({
  liked,
  size = 14,
}: {
  liked: boolean;
  size?: number;
}) {
  const blurVal = size <= 16 ? "2.5px" : "4px";
  const scaleVal = size <= 16 ? 0.72 : 0.6;

  return (
    <span style={{ position: "relative", width: size, height: size, display: "inline-flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
      <AnimatePresence initial={false}>
        <motion.span
          key={liked ? "liked" : "unliked"}
          initial={{ opacity: 0.1, scale: scaleVal, filter: `blur(${blurVal})` }}
          animate={{ opacity: 1,   scale: 1,        filter: "blur(0px)" }}
          exit={{    opacity: 0,   scale: scaleVal, filter: `blur(${blurVal})` }}
          transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: liked ? "var(--color-accent)" : "inherit",
            willChange: "transform, filter, opacity",
            backfaceVisibility: "hidden",
          }}
        >
          <Heart size={size} strokeWidth={liked ? 0 : 2} fill={liked ? "currentColor" : "none"} />
        </motion.span>
      </AnimatePresence>
    </span>
  );
}
