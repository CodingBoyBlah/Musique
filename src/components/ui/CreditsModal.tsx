import { X } from "@/lib/icons";
import { Modal } from "./Modal";
import { CreditsList } from "./CreditsList";
import { useCreditsStore } from "../../store/credits.store";

export function CreditsModal() {
  const track = useCreditsStore((s) => s.track);
  const close = useCreditsStore((s) => s.close);
  return (
    <Modal
      open={!!track}
      onClose={close}
      labelledBy="credits-title"
      panelStyle={{ width: "min(420px, calc(100vw - 32px))", maxHeight: "75vh", display: "flex", flexDirection: "column", overflow: "hidden" }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "16px 18px 10px" }}>
        <h2 id="credits-title" style={{ flex: 1, margin: 0, fontSize: 15, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          Credits{track ? ` · ${track.name}` : ""}
        </h2>
        <button onClick={close} aria-label="Close" className="btn-icon" style={{ width: 28, height: 28, borderRadius: 6 }}>
          <X size={16} />
        </button>
      </div>
      <div style={{ flex: 1, overflowY: "auto", padding: "4px 18px 18px" }}>
        {track && <CreditsList trackId={track.id} onNavigate={close} />}
      </div>
    </Modal>
  );
}
