import { useEffect, type ReactNode } from "react";
import { CoverArt } from "./CoverArt";
import { useUIStore } from "../../store/ui.store";

// 220px on a wide page, easing down with it to 130px; about a fifth of the
// header, which is where the old panel-open "compact" size landed as well
const COVER = "clamp(130px, 20cqw, 220px)";

interface Props {
  imageUrl: string | null | undefined;
  eyebrow: string;
  title: string;
  // artist pages: a circular portrait instead of square artwork
  round?: boolean;
  children: ReactNode; // meta lines + PlayActions
}

/* Album/playlist header.

   The cover is sized off the header's own width (container query units), not
   off whether a side panel is open. So it shrinks and grows continuously as
   the page does - under a right-panel slide, a window drag, the sidebar
   folding - with no render and no layout animation. It used to step down a
   size when a panel opened, as a framer layout animation: a React render on
   the first frame of the slide, and a projection pass that walked every
   `layout` element on the page (every track row on an album) on every frame
   of it. The title keeps one size throughout: type that changes size under a
   reflow reads as a glitch. */
export function PageHeader({ imageUrl, eyebrow, title, round, children }: Props) {
  const setPageTint = useUIStore((s) => s.setPageTint);

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
        // makes cqw below mean "percent of this header's width"
        containerType: "inline-size",
      }}
    >
      {/* flexShrink 0: a long title ellipsises, it never squeezes the artwork */}
      <div
        style={{
          width: COVER,
          height: COVER,
          flexShrink: 0,
          borderRadius: round ? "50%" : 14,
          overflow: "hidden",
          boxShadow: "0 20px 54px rgba(0, 0, 0, 0.6)",
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
      </div>

      <div
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
      </div>
    </div>
  );
}
