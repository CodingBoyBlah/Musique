import { useEffect, type ReactNode } from "react";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { useUIStore } from "../../store/ui.store";
import { usePlayerStore } from "../../store/player.store";

interface Props {
  imageUrl: string | null | undefined;
  eyebrow: string;
  title: string;
  children: ReactNode; // meta lines + PlayActions
}

/* Album/playlist header with coordinated spring scale when lyrics or queue rail opens */
export function PageHeader({ imageUrl, eyebrow, title, children }: Props) {
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
        gap: isCompact ? 16 : 24,
        minWidth: 0,
        padding: isCompact ? "8px 0 16px" : "12px 0 22px",
        transition: "gap 0.35s cubic-bezier(0.23, 1, 0.32, 1), padding 0.35s cubic-bezier(0.23, 1, 0.32, 1)",
      }}
    >
      {/* Fluid spring scaling album cover */}
      <motion.div
        initial={false}
        animate={{
          width: isCompact ? 160 : 240,
          height: isCompact ? 160 : 240,
        }}
        transition={{ type: "spring", stiffness: 340, damping: 34 }}
        style={{
          width: isCompact ? 160 : 240,
          height: isCompact ? 160 : 240,
          flexShrink: 0,
          borderRadius: isCompact ? 10 : 14,
          overflow: "hidden",
          boxShadow: isCompact
            ? "0 10px 24px rgba(0, 0, 0, 0.42)"
            : "0 20px 48px rgba(0, 0, 0, 0.58)",
          transition: "box-shadow 0.35s ease, border-radius 0.35s ease",
        }}
      >
        <div style={{ width: "100%", height: "100%" }}>
          <CoverArt
            url={imageUrl}
            alt={title}
            size={240}
            className="border border-[#FFFFFF14]"
            style={{ width: "100%", height: "100%", borderRadius: "inherit" }}
          />
        </div>
      </motion.div>

      <div
        className="flex flex-col gap-2 min-w-0"
        style={{ flex: 1, paddingBottom: 4 }}
      >
        <p
          className="text-[10.5px] font-bold uppercase tracking-widest"
          style={{ color: "var(--color-text-dim)" }}
        >
          {eyebrow}
        </p>

        <motion.h1
          initial={false}
          animate={{
            fontSize: isCompact ? 26 : 40,
          }}
          transition={{ type: "spring", stiffness: 340, damping: 34 }}
          className="font-black line-clamp-2 break-words"
          title={title}
          style={{
            fontSize: isCompact ? 26 : 40,
            lineHeight: 1.06,
            letterSpacing: "-0.025em",
            color: "#ffffff",
          }}
        >
          {title}
        </motion.h1>

        {children}
      </div>
    </div>
  );
}
