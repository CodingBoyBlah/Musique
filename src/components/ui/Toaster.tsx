import { AnimatePresence, motion } from "framer-motion";
import { Check } from "@/lib/icons";
import { useToastStore } from "../../store/toast.store";

// bottom-centre toast stack. mounted once in Layout.
export function Toaster() {
  const toasts = useToastStore((s) => s.toasts);

  return (
    <div
      style={{
        position: "fixed", left: 0, right: 0, bottom: 96, zIndex: 10000,
        display: "flex", flexDirection: "column", alignItems: "center", gap: 8,
        pointerEvents: "none",
      }}
    >
      <AnimatePresence initial={false}>
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            layout
            initial={{ opacity: 0, y: 14, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 8, scale: 0.97 }}
            transition={{ type: "spring", stiffness: 450, damping: 28 }}
            style={{
              display: "flex", alignItems: "center", gap: 9,
              padding: "10px 18px", borderRadius: 99,
              background: "rgba(24,24,28,0.94)",
              backdropFilter: "blur(30px)", WebkitBackdropFilter: "blur(30px)",
              border: "1px solid rgba(255,255,255,0.14)",
              boxShadow: "0 12px 36px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.05)",
              color: "var(--color-text-hi)", fontSize: 13, fontWeight: 600,
              letterSpacing: "-0.01em",
            }}
          >
            <Check size={15} strokeWidth={2.6} style={{ color: "var(--color-accent)" }} />
            {t.text}
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
