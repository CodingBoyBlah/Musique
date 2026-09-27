import { AnimatePresence, motion } from "framer-motion";
import { AlertTriangle, Check, Info } from "@/lib/icons";
import { useToastStore, type ToastKind } from "../../store/toast.store";

const KIND_ICON: Record<ToastKind, { Icon: typeof Check; color: string }> = {
  success: { Icon: Check,         color: "var(--color-accent)" },
  error:   { Icon: AlertTriangle, color: "var(--color-danger)" },
  info:    { Icon: Info,          color: "var(--color-text-dim)" },
};

// bottom-centre toast stack. mounted once in Layout.
export function Toaster() {
  const toasts = useToastStore((s) => s.toasts);
  const remove = useToastStore((s) => s.remove);

  return (
    <div
      style={{
        position: "fixed", left: 0, right: 0, bottom: 96, zIndex: 10000,
        display: "flex", flexDirection: "column", alignItems: "center", gap: 8,
        pointerEvents: "none",
      }}
    >
      <AnimatePresence initial={false}>
        {toasts.map((t) => {
          const { Icon, color } = KIND_ICON[t.kind];
          return (
            <motion.div
              key={t.id}
              layout
              role={t.kind === "error" ? "alert" : "status"}
              className="toast glass-solid-fallback"
              initial={{ opacity: 0, y: 14, scale: 0.96 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              // leaves the way it came, and faster than it arrived
              exit={{ opacity: 0, y: 8, scale: 0.97, transition: { duration: 0.15, ease: [0.23, 1, 0.32, 1] } }}
              transition={{ type: "spring", bounce: 0, duration: 0.35 }}
              style={{
                display: "flex", alignItems: "center", gap: 9,
                padding: t.action ? "6px 6px 6px 16px" : "10px 18px", borderRadius: 99,
                background: "var(--color-popover)",
                backdropFilter: "blur(20px)", WebkitBackdropFilter: "blur(20px)",
                border: "1px solid rgba(255,255,255,0.14)",
                boxShadow: "0 12px 36px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.05)",
                color: "var(--color-text-hi)", fontSize: 13, fontWeight: 600,
                letterSpacing: "-0.006em",
                pointerEvents: t.action ? "auto" : "none",
              }}
            >
              <Icon size={15} strokeWidth={2.6} style={{ color, flexShrink: 0 }} />
              {t.text}
              {t.action && (
                <button
                  type="button"
                  className="btn-pill"
                  onClick={() => {
                    // the toast stays on screen for its exit animation with this
                    // handler still attached; a double-click must not undo twice
                    if (!useToastStore.getState().toasts.some((x) => x.id === t.id)) return;
                    remove(t.id);
                    t.action!.onClick();
                  }}
                  style={{ marginLeft: 6, height: 28, padding: "0 12px", fontSize: 12.5 }}
                >
                  {t.action.label}
                </button>
              )}
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
}
