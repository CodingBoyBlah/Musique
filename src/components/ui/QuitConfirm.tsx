import { AnimatePresence, motion } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUIStore } from "../../store/ui.store";


export function QuitConfirm() {
  const open    = useUIStore((s) => s.quitConfirmOpen);
  const setOpen = useUIStore((s) => s.setQuitConfirmOpen);

  const quit = () => getCurrentWindow().destroy().catch(() => {});

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.14 }}
          onClick={() => setOpen(false)}
          style={{
            position: "fixed", inset: 0, zIndex: 1000,
            display: "flex", alignItems: "center", justifyContent: "center",
            background: "rgba(0,0,0,0.5)", backdropFilter: "blur(2px)",
          }}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: 6 }}
            transition={{ duration: 0.18, ease: [0.23, 1, 0.32, 1] }}
            onClick={(e) => e.stopPropagation()}
            style={{
              width: 360, padding: "22px 22px 18px", borderRadius: 16,
              background: "rgba(20,20,26,0.96)",
              border: "1px solid rgba(255,255,255,0.12)",
              boxShadow: "0 24px 64px rgba(0,0,0,0.6)",
            }}
          >
            <h2 style={{ margin: 0, fontSize: 16, fontWeight: 700, color: "var(--color-text-hi)" }}>
              Quit Musique?
            </h2>
            <p style={{ margin: "8px 0 20px", fontSize: 13, lineHeight: 1.55, color: "var(--color-text-dim)" }}>
              Music is still playing. Quitting will stop playback.
            </p>
            <div style={{ display: "flex", gap: 10, justifyContent: "flex-end" }}>
              <button
                onClick={() => setOpen(false)}
                style={{
                  height: 38, padding: "0 18px", borderRadius: 99,
                  border: "1px solid var(--color-border)", background: "transparent",
                  color: "var(--color-text-hi)", fontSize: 13, fontWeight: 600, cursor: "pointer",
                }}
              >
                Keep playing
              </button>
              <button
                onClick={quit}
                style={{
                  height: 38, padding: "0 18px", borderRadius: 99, border: "none",
                  background: "var(--color-accent)", color: "var(--color-accent-text)",
                  fontSize: 13, fontWeight: 600, cursor: "pointer",
                }}
              >
                Quit
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
