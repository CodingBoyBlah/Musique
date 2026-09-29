import { useEffect, useMemo, useRef, useState } from "react";
import { X, Info } from "@/lib/icons";
import { Tooltip } from "./Tooltip";
import { SegmentedControl } from "../playground/PlaygroundControls";
import { Modal } from "./Modal";
import { CoverArt } from "./CoverArt";
import { updatePlaylistDetails } from "../../api/library";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import type { PlaylistDetail } from "../../types/spotify";

type Visibility = "public" | "private" | "collaborative";

/* spotify has three sharing states, not two switches: a collaborative
playlist is always private (the api refuses public + collaborative). one
choice of three says that honestly instead of switches that flip each other. */
const VISIBILITY: { value: Visibility; label: string; hint: string }[] = [
  { value: "public", label: "Public", hint: "on your profile and in search" },
  { value: "private", label: "Private", hint: "only people with the link" },
  { value: "collaborative", label: "Collaborative", hint: "friends with the link can add songs (always private on Spotify)" },
];

const NAME_MAX = 100;
const DESC_MAX = 300;

function visibilityOf(p: PlaylistDetail): Visibility {
  if (p.collaborative) return "collaborative";
  if (p.public === false) return "private";
  return "public";
}

function Field({
  label,
  count,
  max,
  grow,
  children,
}: {
  label: string;
  count: number;
  max: number;
  grow?: boolean;
  children: React.ReactNode;
}) {
  const near = count > max * 0.85;
  return (
    <label style={{ display: "flex", flexDirection: "column", gap: 6, minWidth: 0, flex: grow ? 1 : undefined }}>
      <span style={{ display: "flex", justifyContent: "space-between", fontSize: 12, fontWeight: 600, color: "var(--color-text-dim)" }}>
        <span>{label}</span>
        <span className="tnum" style={{ opacity: near ? 1 : 0, transition: "opacity 0.15s ease", color: count >= max ? "var(--color-text-hi)" : undefined }}>
          {count}/{max}
        </span>
      </span>
      {children}
    </label>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  borderRadius: 10,
  background: "rgba(255,255,255,0.05)",
  border: "1px solid rgba(255,255,255,0.08)",
  color: "var(--color-text-hi)",
  fontSize: 14,
  fontFamily: "inherit",
  padding: "10px 12px",
  outline: "none",
  transition: "border-color 0.14s ease, background 0.14s ease",
};

// name / description / sharing for a playlist you own
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
  const [visibility, setVisibility] = useState<Visibility>(visibilityOf(playlist));
  const [busy, setBusy] = useState(false);

  // reopening starts from the playlist as it is now, not a stale draft
  useEffect(() => {
    if (!open) return;
    setName(playlist.name);
    setDescription(playlist.description ?? "");
    setVisibility(visibilityOf(playlist));
  }, [open, playlist]);

  const initialVis = visibilityOf(playlist);
  const dirty = useMemo(
    () =>
      name.trim() !== playlist.name ||
      description.trim() !== (playlist.description ?? "") ||
      visibility !== initialVis,
    [name, description, visibility, playlist, initialVis],
  );
  const canSave = dirty && !!name.trim() && !busy;

  async function save() {
    if (!canSave) return;
    setBusy(true);
    try {
      await updatePlaylistDetails(playlist.id, {
        name: name.trim() !== playlist.name ? name.trim() : undefined,
        description: description.trim() !== (playlist.description ?? "") ? description.trim() : undefined,
        public: visibility !== initialVis ? visibility === "public" : undefined,
        collaborative: visibility !== initialVis ? visibility === "collaborative" : undefined,
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
      panelStyle={{ width: "min(560px, calc(100vw - 32px))", padding: 0, overflow: "hidden" }}
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          save();
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
            e.preventDefault();
            save();
          }
        }}
      >
        <div style={{ display: "flex", alignItems: "center", padding: "18px 20px 4px" }}>
          <h2 id="edit-playlist-title" style={{ flex: 1, margin: 0, fontSize: 17, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>
            Edit details
          </h2>
          <button type="button" onClick={onClose} aria-label="Close" className="btn-icon" style={{ width: 30, height: 30, borderRadius: 99 }}>
            <X size={16} />
          </button>
        </div>

        <div style={{ display: "flex", gap: 18, padding: "14px 20px 0", flexWrap: "wrap" }}>
          <div
            aria-hidden
            style={{
              width: 184,
              height: 184,
              borderRadius: 12,
              overflow: "hidden",
              flexShrink: 0,
              boxShadow: "0 14px 34px rgba(0,0,0,0.45)",
            }}
          >
            <CoverArt url={playlist.image_url} alt="" size={184} style={{ width: "100%", height: "100%" }} />
          </div>

          <div style={{ flex: "1 1 240px", minWidth: 0, minHeight: 184, display: "flex", flexDirection: "column", gap: 12 }}>
            <Field label="Name" count={name.length} max={NAME_MAX}>
              <input
                ref={nameRef}
                className="focus-ring"
                value={name}
                maxLength={NAME_MAX}
                onChange={(e) => setName(e.target.value)}
                placeholder="Give it a name"
                style={{ ...inputStyle, fontSize: 15, fontWeight: 600 }}
              />
            </Field>
            <Field label="Description" count={description.length} max={DESC_MAX} grow>
              <textarea
                className="focus-ring"
                value={description}
                maxLength={DESC_MAX}
                rows={3}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Add an optional description"
                style={{ ...inputStyle, resize: "none", flex: 1, minHeight: 72, lineHeight: 1.45 }}
              />
            </Field>
          </div>
        </div>

        <div style={{ padding: "20px 20px 0" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 10 }}>
            <span id="edit-playlist-vis" style={{ fontSize: 12, fontWeight: 600, color: "var(--color-text-dim)" }}>Who can see it</span>
            <Tooltip
              side="top"
              label={
                <span style={{ display: "flex", flexDirection: "column", gap: 4, maxWidth: 240, textAlign: "left" }}>
                  {VISIBILITY.map((v) => (
                    <span key={v.value}><b>{v.label}</b>: {v.hint}</span>
                  ))}
                </span>
              }
            >
              <span tabIndex={0} aria-label="What these mean" className="focus-ring" style={{ display: "inline-flex", color: "var(--color-text-dim)", cursor: "help", borderRadius: 99 }}>
                <Info size={13} />
              </span>
            </Tooltip>
          </div>
          <div role="radiogroup" aria-labelledby="edit-playlist-vis" style={{ display: "flex" }}>
            <SegmentedControl
              options={VISIBILITY.map((v) => v.label)}
              value={VISIBILITY.find((v) => v.value === visibility)!.label}
              onChange={(label) => setVisibility(VISIBILITY.find((v) => v.label === label)?.value ?? "public")}
              layoutId="edit-playlist-visibility"
            />
          </div>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 12,
            padding: "18px 20px 18px",
          }}
        >
          <span className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>
            {dirty ? "Ctrl + Enter to save" : "No changes yet"}
          </span>
          <div style={{ display: "flex", gap: 8 }}>
            <button type="button" className="btn-pill" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary" disabled={!canSave} style={{ minWidth: 84 }}>
              {busy ? "Saving..." : "Save"}
            </button>
          </div>
        </div>
      </form>
    </Modal>
  );
}
