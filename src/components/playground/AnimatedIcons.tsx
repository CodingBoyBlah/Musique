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
      <span
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: isPlaying ? 1 : 0,
          transform: isPlaying ? "scale(1)" : "scale(0.68)",
          transition: "opacity 0.2s cubic-bezier(0.22, 1, 0.36, 1), transform 0.2s cubic-bezier(0.22, 1, 0.36, 1)",
          pointerEvents: isPlaying ? "auto" : "none",
        }}
      >
        <Pause size={size} strokeWidth={strokeWidth > 0 ? strokeWidth : 2.4} fill={fill} />
      </span>
      <span
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: !isPlaying ? 1 : 0,
          transform: !isPlaying ? "scale(1)" : "scale(0.68)",
          transition: "opacity 0.2s cubic-bezier(0.22, 1, 0.36, 1), transform 0.2s cubic-bezier(0.22, 1, 0.36, 1)",
          pointerEvents: !isPlaying ? "auto" : "none",
        }}
      >
        <Play size={size} strokeWidth={strokeWidth} fill={fill} style={{ marginLeft: 1.5 }} />
      </span>
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
  const Icon = muted || volume === 0 ? VolumeX : volume < 50 ? Volume1 : Volume2;

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
      }}
    >
      <Icon size={size} strokeWidth={2} fill="currentColor" />
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
        color: liked ? "var(--color-accent)" : "inherit",
        transform: liked ? "scale(1.08)" : "scale(1)",
        transition: "transform 0.2s cubic-bezier(0.22, 1, 0.36, 1), color 0.15s ease",
      }}
    >
      <Heart size={size} strokeWidth={liked ? 0 : 2} fill={liked ? "currentColor" : "none"} />
    </span>
  );
}
