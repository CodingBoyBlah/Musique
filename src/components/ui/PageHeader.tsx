import { useEffect, type ReactNode } from "react";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { useUIStore } from "../../store/ui.store";
import { usePlayerStore } from "../../store/player.store";
import { REFLOW_SPRING } from "../../lib/motion";

interface Props {
  imageUrl: string | null | undefined;
  eyebrow: string;
  title: string;
  // artist pages: a circular portrait instead of square artwork
  round?: boolean;
  children: ReactNode; // meta lines + PlayActions
}

/* Album/playlist header. When the lyrics or queue rail opens the cover steps
   down a size - as a layout animation, so the size flips once and framer
   glides the difference as a transform on the same spring the grids reflow
   on. It used to switch width/height straight away, and the header snapped
   while everything around it moved. The title keeps one size throughout:
   type that jumps a size mid-reflow reads as a glitch. */
export function PageHeader({ imageUrl, eyebrow, title, round, children }: Props) {
  const setPageTint = useUIStore((s) => s.setPageTint);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const isCompact = lyricsOpen || queueOpen;

  // publish this page's cover to the UI store for live accent tinting
  useEffect(() => {
    setPageTint(imageUrl ?? null);
    return () => setPageTint(null);
  }, [imageUrl, setPageTint]);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-end",
        gap: "clamp(16px, 2.4vw, 24px)",
        flexWrap: "wrap",
        minWidth: 0,
        padding: "12px 0 20px",
      }}
    >
      {/* Fluid responsive album cover. flexShrink 0: a long title ellipsises,
          it never squeezes the artwork. */}
      <motion.div
        layout
        transition={{ layout: REFLOW_SPRING }}
        style={{
          width: isCompact ? "clamp(130px, 17vw, 170px)" : "clamp(140px, 20vw, 220px)",
          height: isCompact ? "clamp(130px, 17vw, 170px)" : "clamp(140px, 20vw, 220px)",
          flexShrink: 0,
          borderRadius: round ? "50%" : 14,
          overflow: "hidden",
          boxShadow: isCompact
            ? "0 14px 36px rgba(0, 0, 0, 0.5)"
            : "0 24px 64px rgba(0, 0, 0, 0.65)",
        }}
      >
        {/* radius passes down through this wrapper so the image is clipped to
            the same shape as the frame. The hairline suits square artwork; on
            a circle, a square 1px border showed as flat edges at 12, 3, 6 and
            9 o'clock, so a portrait fills the circle edge to edge. */}
        <div style={{ width: "100%", height: "100%", borderRadius: "inherit" }}>
          <CoverArt
            url={imageUrl}
            alt={title}
            size={240}
            className={round ? "" : "border border-[#FFFFFF14]"}
            style={{ width: "100%", height: "100%", borderRadius: "inherit" }}
          />
        </div>
      </motion.div>

      <motion.div
        layout="position"
        transition={{ layout: REFLOW_SPRING }}
        className="flex flex-col gap-2 min-w-0"
        style={{ flex: "1 1 240px", paddingBottom: 4 }}
      >
        <p
          className="font-bold uppercase"
          style={{ color: "var(--color-text-dim)", margin: 0, fontSize: 11, letterSpacing: "0.06em" }}
        >
          {eyebrow}
        </p>

        <h1
          className="font-black line-clamp-2 break-words"
          title={title}
          style={{
            fontSize: "clamp(24px, 3.8vw, 40px)",
            lineHeight: 1.06,
            letterSpacing: "-0.028em",
            color: "#ffffff",
            margin: 0,
          }}
        >
          {title}
        </h1>

        {children}
      </motion.div>
    </div>
  );
}
