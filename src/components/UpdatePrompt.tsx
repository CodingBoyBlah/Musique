import { useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Sparkles, RotateCw, AlertTriangle } from "lucide-react";
import { useUpdaterStore } from "../store/updater.store";
import { runUpdateCheck, startDownload, restartApp } from "../lib/updater";

// Polished in-app updater dialog (Cursor / Arc / Obsidian style). Mounted once at
// the app root. Runs exactly ONE silent update check on startup; only ever appears
// on screen when an update is actually available.
export function UpdatePrompt() {
  const stage = useUpdaterStore((s) => s.stage);
  const open = useUpdaterStore((s) => s.open);
  const version = useUpdaterStore((s) => s.version);
  const notes = useUpdaterStore((s) => s.notes);
  const progress = useUpdaterStore((s) => s.progress);
  const error = useUpdaterStore((s) => s.error);
  const dismiss = useUpdaterStore((s) => s.dismiss);

  // one silent check per launch, no polling afterwards
  useEffect(() => {
    runUpdateCheck();
  }, []);

  const busy = stage === "downloading";

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.18 }}
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 1000,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "rgba(0,0,0,0.5)",
            backdropFilter: "blur(4px)",
            WebkitBackdropFilter: "blur(4px)",
            padding: 24,
          }}
          // clicking the backdrop = "Later", but not while downloading/installing
          onClick={() => {
            if (stage === "available") dismiss();
          }}
        >
          <motion.div
            initial={{ opacity: 0, y: 16, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 12, scale: 0.98 }}
            transition={{ duration: 0.22, ease: [0.23, 1, 0.32, 1] }}
            onClick={(e) => e.stopPropagation()}
            style={{
              width: "100%",
              maxWidth: 420,
              background: "var(--color-surface-elevated, rgba(24,24,28,0.98))",
              backgroundColor: "#141418",
              border: "1px solid var(--color-border)",
              borderRadius: 16,
              boxShadow: "0 24px 70px rgba(0,0,0,0.55)",
              padding: 22,
              color: "var(--color-text-hi)",
            }}
          >
            {/* header icon */}
            <div
              style={{
                width: 40,
                height: 40,
                borderRadius: 11,
                marginBottom: 14,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                background: "var(--color-accent-dim)",
                color: "var(--color-accent)",
              }}
            >
              {stage === "error" ? (
                <AlertTriangle size={20} strokeWidth={2.2} />
              ) : stage === "installed" ? (
                <RotateCw size={20} strokeWidth={2.2} />
              ) : (
                <Sparkles size={20} strokeWidth={2.2} />
              )}
            </div>

            {stage === "available" && (
              <>
                <h2 style={titleStyle}>Musique {version} is available</h2>
                <p style={subStyle}>A new version is ready to install.</p>
                {notes && (
                  <div style={notesBox} data-selectable>
                    <p
                      style={{
                        margin: 0,
                        whiteSpace: "pre-wrap",
                        fontSize: 12.5,
                        lineHeight: 1.5,
                        color: "var(--color-text)",
                      }}
                    >
                      {notes.trim()}
                    </p>
                  </div>
                )}
                <div style={btnRow}>
                  <button style={ghostBtn} onClick={dismiss}>
                    Later
                  </button>
                  <button style={primaryBtn} onClick={startDownload}>
                    Update
                  </button>
                </div>
              </>
            )}

            {stage === "downloading" && (
              <>
                <h2 style={titleStyle}>Downloading update…</h2>
                <p style={subStyle}>
                  Musique {version} — please keep the app open.
                </p>
                <div style={{ marginTop: 18 }}>
                  <div
                    style={{
                      height: 8,
                      borderRadius: 99,
                      background: "rgba(255,255,255,0.10)",
                      overflow: "hidden",
                    }}
                  >
                    <motion.div
                      animate={{ width: `${progress}%` }}
                      transition={{ ease: "easeOut", duration: 0.25 }}
                      style={{
                        height: "100%",
                        borderRadius: 99,
                        background: "var(--color-accent)",
                      }}
                    />
                  </div>
                  <p
                    style={{
                      margin: "8px 0 0",
                      fontSize: 12,
                      fontVariantNumeric: "tabular-nums",
                      color: "var(--color-text-dim)",
                      textAlign: "right",
                    }}
                  >
                    {progress}%
                  </p>
                </div>
              </>
            )}

            {stage === "installed" && (
              <>
                <h2 style={titleStyle}>Update installed</h2>
                <p style={subStyle}>
                  Restart to finish updating to Musique {version}.
                </p>
                <div style={btnRow}>
                  <button
                    style={{ ...primaryBtn, width: "100%" }}
                    onClick={restartApp}
                  >
                    Restart now
                  </button>
                </div>
              </>
            )}

            {stage === "error" && (
              <>
                <h2 style={titleStyle}>Update failed</h2>
                <p style={subStyle}>
                  {error ?? "Something went wrong while updating."}
                </p>
                <div style={btnRow}>
                  <button
                    style={{ ...ghostBtn, width: "100%" }}
                    onClick={dismiss}
                  >
                    Close
                  </button>
                </div>
              </>
            )}

            {/* keep buttons disabled visually while busy is handled by not rendering
                actionable controls in the downloading stage */}
            {busy && null}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

const titleStyle: React.CSSProperties = {
  margin: 0,
  fontSize: 17,
  fontWeight: 700,
  letterSpacing: "-0.01em",
};
const subStyle: React.CSSProperties = {
  margin: "6px 0 0",
  fontSize: 13,
  color: "var(--color-text-dim)",
  lineHeight: 1.45,
};
const notesBox: React.CSSProperties = {
  marginTop: 14,
  maxHeight: 168,
  overflowY: "auto",
  padding: "12px 14px",
  borderRadius: 10,
  background: "var(--color-surface)",
  border: "1px solid var(--color-border)",
};
const btnRow: React.CSSProperties = {
  display: "flex",
  gap: 10,
  marginTop: 20,
  justifyContent: "flex-end",
};
const baseBtn: React.CSSProperties = {
  height: 36,
  padding: "0 18px",
  borderRadius: 9,
  cursor: "pointer",
  fontSize: 13.5,
  fontWeight: 600,
  fontFamily: "inherit",
  border: "none",
};
const ghostBtn: React.CSSProperties = {
  ...baseBtn,
  background: "var(--color-surface-2)",
  color: "var(--color-text)",
};
const primaryBtn: React.CSSProperties = {
  ...baseBtn,
  background: "var(--color-accent)",
  color: "var(--color-accent-text)",
};
