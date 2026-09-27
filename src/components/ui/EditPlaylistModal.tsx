import { useEffect, useRef, useState } from "react";
import { X } from "@/lib/icons";
import { Modal } from "./Modal";
import { Switch } from "./Switch";
import { updatePlaylistDetails } from "../../api/library";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import type { PlaylistDetail } from "../../types/spotify";

const field: React.CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  borderRadius: 9,
  background: "rgba(0,0,0,0.25)",
  border: "1px solid var(--color-border)",
  color: "var(--color-text-hi)",
  fontSize: 13.5,
  fontFamily: "inherit",
  padding: "9px 12px",
  outline: "none",
};

const label: React.CSSProperties = {
  display: "block",
  fontSize: 11.5,
  fontWeight: 600,
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--color-text-dim)",
  margin: "14px 0 6px",
};

// name / description / public / collaborative for a playlist you own
export function EditPlaylistModal({
  playlist,
  open,
  onClose,
  onSaved,
}: {
  playlist: PlaylistDetail;
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
}) {
  const nameRef = useRef<HTMLInputElement>(null);
  const [name, setName] = useState(playlist.name);
  const [description, setDescription] = useState(playlist.description ?? "");
  const [isPublic, setIsPublic] = useState(playlist.public ?? false);
  const [collaborative, setCollaborative] = useState(playlist.collaborative ?? false);
  const [busy, setBusy] = useState(false);

  // reopening starts from the playlist as it is now, not a stale draft
  useEffect(() => {
    if (!open) return;
    setName(playlist.name);
    setDescription(playlist.description ?? "");
    setIsPublic(playlist.public ?? false);
    setCollaborative(playlist.collaborative ?? false);
  }, [open, playlist]);

  async function save() {
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      await updatePlaylistDetails(playlist.id, {
        name: name.trim() !== playlist.name ? name.trim() : undefined,
        description: description.trim() !== (playlist.description ?? "") ? description.trim() : undefined,
        public: isPublic !== (playlist.public ?? false) ? isPublic : undefined,
        collaborative: collaborative !== (playlist.collaborative ?? false) ? collaborative : undefined,
      });
      toast("Playlist updated");
      onSaved();
      onClose();
    } catch (e) {
      toast.error(`Couldn't update playlist: ${errMsg(e)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      labelledBy="edit-playlist-title"
      initialFocus={nameRef}
      panelStyle={{ width: "min(440px, calc(100vw - 32px))", padding: "16px 18px 16px" }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <h2 id="edit-playlist-title" style={{ flex: 1, margin: 0, fontSize: 15, fontWeight: 700, color: "var(--color-text-hi)" }}>
          Edit details
        </h2>
        <button onClick={onClose} aria-label="Close" className="btn-icon" style={{ width: 28, height: 28, borderRadius: 6 }}>
          <X size={16} />
        </button>
      </div>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          save();
        }}
      >
        <label style={label} htmlFor="edit-playlist-name">Name</label>
        <input
          id="edit-playlist-name"
          ref={nameRef}
          className="focus-ring"
          value={name}
          maxLength={100}
          onChange={(e) => setName(e.target.value)}
          style={field}
        />
        <label style={label} htmlFor="edit-playlist-desc">Description</label>
        <textarea
          id="edit-playlist-desc"
          className="focus-ring"
          value={description}
          maxLength={300}
          rows={3}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Add an optional description"
          style={{ ...field, resize: "vertical", minHeight: 72 }}
        />

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginTop: 16 }}>
          <div>
            <div style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)" }}>Public</div>
            <div className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>Shows on your profile and in search</div>
          </div>
          <Switch
            label="Public"
            checked={isPublic && !collaborative}
            onChange={(v) => {
              setIsPublic(v);
              if (v) setCollaborative(false);
            }}
          />
        </div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginTop: 12 }}>
          <div>
            <div style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)" }}>Collaborative</div>
            <div className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>Anyone you share it with can add songs</div>
          </div>
          <Switch
            label="Collaborative"
            checked={collaborative}
            onChange={(v) => {
              setCollaborative(v);
              if (v) setIsPublic(false);
            }}
          />
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
          <button type="button" className="btn-pill" onClick={onClose}>Cancel</button>
          <button type="submit" className="btn-primary" disabled={busy || !name.trim()}>
            {busy ? "Saving..." : "Save"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
