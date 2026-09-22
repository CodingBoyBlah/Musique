import { useEffect, type ReactNode } from "react";
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
        gap: "clamp(16px, 2.4vw, 24px)",
        flexWrap: "wrap",
        minWidth: 0,
        padding: "12px 0 20px",
      }}
    >
      {/* Fluid responsive album cover */}
      <div
        style={{
          width: isCompact ? "clamp(130px, 17vw, 170px)" : "clamp(140px, 20vw, 220px)",
          height: isCompact ? "clamp(130px, 17vw, 170px)" : "clamp(140px, 20vw, 220px)",
          flexShrink: 0,
          borderRadius: 14,
          overflow: "hidden",
          boxShadow: isCompact
            ? "0 14px 36px rgba(0, 0, 0, 0.5)"
            : "0 24px 64px rgba(0, 0, 0, 0.65)",
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
      </div>

      <div
        className="flex flex-col gap-2 min-w-0"
        style={{ flex: "1 1 240px", paddingBottom: 4 }}
      >
        <p
          className="text-[10.5px] font-bold uppercase tracking-widest"
          style={{ color: "var(--color-text-dim)", margin: 0 }}
        >
          {eyebrow}
        </p>

        <h1
          className="font-black line-clamp-2 break-words"
          title={title}
          style={{
            fontSize: isCompact ? "clamp(22px, 3.4vw, 32px)" : "clamp(26px, 4.2vw, 44px)",
            lineHeight: 1.06,
            letterSpacing: "-0.025em",
            color: "#ffffff",
            margin: 0,
          }}
        >
          {title}
        </h1>

        {children}
      </div>
    </div>
  );
}
