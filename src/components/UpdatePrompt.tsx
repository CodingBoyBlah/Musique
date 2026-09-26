import { useEffect } from "react";
import { motion } from "framer-motion";
import { EASE_OUT } from "@/lib/motion";
import { Modal } from "./ui/Modal";
import { Sparkles, RotateCw, AlertTriangle } from "@/lib/icons";
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

  return (
    <Modal
      open={open}
      // Escape / backdrop = "Later", but not while downloading or installing
      onClose={() => {
        if (stage === "available" || stage === "error") dismiss();
      }}
      labelledBy="update-prompt-title"
      zIndex={1000}
      panelStyle={{
        width: "calc(100% - 48px)",
        maxWidth: 420,
        background: "#141418",
        border: "1px solid var(--color-border)",
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
          <h2 id="update-prompt-title" style={titleStyle}>Musique {version} is available</h2>
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
            <button className="btn-pill" style={ghostBtn} onClick={dismiss}>
              Later
            </button>
            <button className="btn-primary" style={baseBtn} onClick={startDownload}>
              Update
            </button>
          </div>
        </>
      )}

      {stage === "downloading" && (
        <>
          <h2 id="update-prompt-title" style={titleStyle}>Downloading update…</h2>
          <p style={subStyle}>
            Musique {version} - please keep the app open.
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
              {/* scaleX, not width: a transform never relayouts */}
              <motion.div
                initial={false}
                animate={{ scaleX: progress / 100 }}
                transition={{ ease: EASE_OUT, duration: 0.25 }}
                style={{
                  height: "100%",
                  borderRadius: 99,
                  background: "var(--color-accent)",
                  transformOrigin: "left center",
                }}
              />
            </div>
            <p
              style={{
                margin: "8px 0 0",
                fontSize: 12,
                fontVariantNumeric: "tabular-nums",
                letterSpacing: "0.004em",
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
          <h2 id="update-prompt-title" style={titleStyle}>Update installed</h2>
          <p style={subStyle}>
            Restart to finish updating to Musique {version}.
          </p>
          <div style={btnRow}>
            <button
              className="btn-primary"
              style={{ ...baseBtn, width: "100%" }}
              onClick={restartApp}
            >
              Restart now
            </button>
          </div>
        </>
      )}

      {stage === "error" && (
        <>
          <h2 id="update-prompt-title" style={titleStyle}>Update failed</h2>
          <p style={subStyle}>
            {error ?? "Something went wrong while updating."}
          </p>
          <div style={btnRow}>
            <button
              className="btn-pill"
              style={{ ...ghostBtn, width: "100%" }}
              onClick={dismiss}
            >
              Close
            </button>
          </div>
        </>
      )}

    </Modal>
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
  fontSize: 13.5,
};
const ghostBtn: React.CSSProperties = {
  ...baseBtn,
  color: "var(--color-text)",
};
