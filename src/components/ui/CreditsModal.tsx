import { X } from "@/lib/icons";
import { Modal } from "./Modal";
import { CreditsList } from "./CreditsList";
import { CoverArt } from "./CoverArt";
import { useCreditsStore } from "../../store/credits.store";

export function CreditsModal() {
  const track = useCreditsStore((s) => s.track);
  const close = useCreditsStore((s) => s.close);
  return (
    <Modal
      open={!!track}
      onClose={close}
      labelledBy="credits-title"
      panelStyle={{ width: "min(440px, calc(100vw - 32px))", maxHeight: "78vh", display: "flex", flexDirection: "column", overflow: "hidden", padding: 0 }}
    >
      {track && (
        <>
          <div style={{ display: "flex", alignItems: "center", gap: 14, padding: "18px 18px 16px", borderBottom: "1px solid var(--color-divider)" }}>
            <div style={{ width: 56, height: 56, borderRadius: 8, overflow: "hidden", flexShrink: 0, boxShadow: "0 8px 22px rgba(0,0,0,0.4)" }}>
              <CoverArt url={track.album?.image_url} alt="" size={56} style={{ width: 56, height: 56 }} />
            </div>
            <div style={{ minWidth: 0, flex: 1 }}>
              <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-dim)" }}>Credits</div>
              <h2 id="credits-title" style={{ margin: "2px 0 0", fontSize: 16, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {track.name}
              </h2>
              <div className="t-caption" style={{ fontSize: 12.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {track.artists.map((a) => a.name).join(", ")}
              </div>
            </div>
            <button onClick={close} aria-label="Close" className="btn-icon" style={{ width: 30, height: 30, borderRadius: 99, alignSelf: "flex-start" }}>
              <X size={16} />
            </button>
          </div>
          <div className="scroll-y" style={{ flex: 1, overflowY: "auto", padding: "18px 18px 20px" }}>
            <CreditsList trackId={track.id} onNavigate={close} />
          </div>
        </>
      )}
    </Modal>
  );
}
