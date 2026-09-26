import { useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUIStore } from "../../store/ui.store";
import { Modal } from "./Modal";


export function QuitConfirm() {
  const open    = useUIStore((s) => s.quitConfirmOpen);
  const setOpen = useUIStore((s) => s.setQuitConfirmOpen);
  // the safe choice is the default: Enter / Escape both keep the music going
  const keepRef = useRef<HTMLButtonElement>(null);

  const quit = () => getCurrentWindow().destroy().catch(() => {});

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      labelledBy="quit-confirm-title"
      initialFocus={keepRef}
      zIndex={1000}
      panelStyle={{ width: "min(360px, calc(100vw - 32px))", padding: "22px 22px 18px" }}
    >
      <h2 id="quit-confirm-title" style={{ margin: 0, fontSize: 16, fontWeight: 700, color: "var(--color-text-hi)" }}>
        Quit Musique?
      </h2>
      <p style={{ margin: "8px 0 20px", fontSize: 13, lineHeight: 1.55, color: "var(--color-text-dim)" }}>
        Music is still playing. Quitting will stop playback.
      </p>
      <div style={{ display: "flex", gap: 10, justifyContent: "flex-end" }}>
        {/* stopping playback is the destructive choice, so it doesn't get the
            accent - the primary button is the one that keeps things as they are */}
        <button
          onClick={quit}
          className="btn-pill"
          style={{ height: 38, padding: "0 18px" }}
        >
          Quit
        </button>
        <button
          ref={keepRef}
          onClick={() => setOpen(false)}
          className="btn-primary"
          style={{ height: 38, padding: "0 18px" }}
        >
          Keep playing
        </button>
      </div>
    </Modal>
  );
}
