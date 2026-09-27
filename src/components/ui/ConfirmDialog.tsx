import { useRef, type ReactNode } from "react";
import { Modal } from "./Modal";

// a small "are you sure" for actions that can't be undone. the safe choice
// (Cancel) gets the initial focus so Enter never confirms by accident.
export function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel,
  danger,
  busy,
  onConfirm,
  onClose,
}: {
  open: boolean;
  title: string;
  body: ReactNode;
  confirmLabel: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  const cancelRef = useRef<HTMLButtonElement>(null);
  return (
    <Modal
      open={open}
      onClose={onClose}
      labelledBy="confirm-dialog-title"
      initialFocus={cancelRef}
      panelStyle={{ width: "min(380px, calc(100vw - 32px))", padding: "18px 18px 16px" }}
    >
      <h2 id="confirm-dialog-title" className="t-title" style={{ margin: 0, fontSize: 16, fontWeight: 700, color: "var(--color-text-hi)" }}>
        {title}
      </h2>
      <div style={{ marginTop: 8, fontSize: 13.5, lineHeight: 1.5, color: "var(--color-text)" }}>{body}</div>
      <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 18 }}>
        <button ref={cancelRef} type="button" className="btn-pill" onClick={onClose}>
          Cancel
        </button>
        <button
          type="button"
          className={danger ? "btn-danger" : "btn-primary"}
          disabled={busy}
          onClick={onConfirm}
        >
          {confirmLabel}
        </button>
      </div>
    </Modal>
  );
}
