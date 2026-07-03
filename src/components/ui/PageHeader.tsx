import { useEffect, type ReactNode } from "react";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { useUIStore } from "../../store/ui.store";
import { useReflowPulse } from "../../hooks/useReflowPulse";
import { useThemeStore } from "../../store/theme.store";
import { loadCoverAccent, applyAccent } from "../../lib/color";


interface Props {
  imageUrl: string | null | undefined;
  eyebrow: string;
  title: string;
  children: ReactNode; // meta lines + PlayActions
}

/*album/playlist header    note: the blurred artwork tint isnt drawn here -- it
 gets pushed to the UI store and Layout paints it as a fixed faint backdrop 
 behind the whole content area, so it stays put while the page scrolls. */
export function PageHeader({ imageUrl, eyebrow, title, children }: Props) {
  const setPageTint = useUIStore((s) => s.setPageTint);
  const albumColors = useThemeStore((s) => s.albumColors);
  useReflowPulse(); // re-render on resize / panel toggle so the header layout glides DONE

  useEffect(() => {
    setPageTint(imageUrl ?? null);
    return () => setPageTint(null);
  }, [imageUrl, setPageTint]);

    useEffect(() => {
    if (!albumColors || !imageUrl) return;
    let cancelled = false;
    loadCoverAccent(imageUrl).then((hex) => {
      if (!cancelled && hex) applyAccent(hex);
    });
    return () => {
      cancelled = true;
      applyAccent(useThemeStore.getState().baseAccent);
    };
  }, [albumColors, imageUrl]);


  return (
    <motion.div
      layout
      style={{
        display: "flex",
        alignItems: "flex-end",
        gap: "clamp(14px, 2.4vw, 24px)",
        minWidth: 0,
        padding: "clamp(8px, 1.4vw, 12px) 0 clamp(14px, 2vw, 20px)",
        /* only wrap (cover on top, text below) at the extreme narrow end. before  that the cover just shrinks instead of crushing the text column. */
        flexWrap: "wrap",
        rowGap: "clamp(12px, 1.6vw, 16px)",
      }}
    >
      {/* cover is fluid and tracks the content box, not the viewport  basis is
          a % of the flex container (clamped in px) so when a side panel
          (lyrics/queue) opens and this column shrinks the cover scales down with
          it, instead of staying viewport-pinned and shoving text onto a new
          line. flex-shrink:1 + the small min keep it shrinking, wrap only kicks
          in at a true extreme. layout animates the resize. */}
      <motion.div
        layout
        initial={{ opacity: 0, scale: 0.94 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.45, ease: [0.23, 1, 0.32, 1] }}
        style={{ flex: "0 1 clamp(150px, 28%, 282px)", minWidth: 150, maxWidth: 282 }}
      >
        <div style={{ width: "100%", aspectRatio: "1 / 1" }}>
          <CoverArt
            url={imageUrl}
            alt={title}
            size={282}
            className="shadow-xl shadow-black/50 border border-[#FFFFFF14]"
            style={{ width: "100%", height: "100%" }}
          />
        </div>
      </motion.div>

      <div className="flex flex-col gap-2 min-w-0" style={{ flex: "1 1 220px", paddingBottom: 4 }}>
        <p
          className="text-[10px] font-bold uppercase tracking-widest"
          style={{ color: "var(--color-text-dim)" }}
        >
          {eyebrow}
        </p>
        {/* clamp to 2 lines + ellepses. a long album/playlist name used to
            balloon this column (and crush the cover) when a side panel opened;
            now it truncates. break-words handles a long unbroken word too.
            title attr shows the full name on hover. */}

        {/* TODO experimenet with ellepses limit */}
        <h1
          className="font-black line-clamp-2 break-words"
          title={title}
          style={{ fontSize: "clamp(26px, 4.2vw, 44px)", lineHeight: 1.05, letterSpacing: "-0.02em" }}
        >
          {title}
        </h1>
        {children}
      </div>
    </motion.div>
  );
}
