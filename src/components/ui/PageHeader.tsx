import { useEffect, type ReactNode } from "react";
import { motion } from "framer-motion";
import { CoverArt } from "./CoverArt";
import { useUIStore } from "../../store/ui.store";
import { useReflowPulse } from "../../hooks/useReflowPulse";

interface Props {
  imageUrl: string | null | undefined;
  eyebrow: string;
  title: string;
  children: ReactNode; 


export function PageHeader({ imageUrl, eyebrow, title, children }: Props) {
  const setPageTint = useUIStore((s) => s.setPageTint);
  useReflowPulse(); 

  
  useEffect(() => {
    setPageTint(imageUrl ?? null);
    return () => setPageTint(null);
  }, [imageUrl, setPageTint]);

  return (
    <motion.div
      layout
      style={{
       
        containerType: "inline-size",
        display: "flex",
        alignItems: "flex-end",
        gap: "clamp(14px, 2.4cqi, 24px)",
        minWidth: 0,
        padding: "clamp(8px, 1.4vw, 12px) 0 clamp(14px, 2vw, 20px)",
        
        flexWrap: "wrap",
        rowGap: "clamp(12px, 1.6vw, 16px)",
      }}
    >
      
      <motion.div
        layout
        initial={{ opacity: 0, scale: 0.94 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.45, ease: [0.23, 1, 0.32, 1] }}
        style={{ flex: "0 1 clamp(178px, 34cqi, 300px)", minWidth: 178, maxWidth: 300 }}
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

      <motion.div layout className="flex flex-col gap-2 min-w-0" style={{ flex: "1 1 220px", paddingBottom: 4 }}>
        <p
          className="text-[10px] font-bold uppercase tracking-widest"
          style={{ color: "var(--color-text-dim)" }}
        >
          {eyebrow}
        </p>
        

        {/* TODO experimenet with ellepses limit */}
        <h1
          className="font-black line-clamp-2 break-words"
          title={title}
          style={{ fontSize: "clamp(22px, 7cqi, 44px)", lineHeight: 1.05, letterSpacing: "-0.02em" }}
        >
          {title}
        </h1>
        {children}
      </div>
    </motion.div>
  );
}
