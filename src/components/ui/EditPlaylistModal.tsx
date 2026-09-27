import { useEffect, useMemo, useRef, useState } from "react";
import { motion } from "framer-motion";
import { X, Globe, EyeOff, Users, Check } from "@/lib/icons";
import { Modal } from "./Modal";
import { CoverArt } from "./CoverArt";
import { updatePlaylistDetails } from "../../api/library";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import { PRESS, PRESS_TRANSITION, SPRING } from "../../lib/motion";
import type { PlaylistDetail } from "../../types/spotify";

type Visibility = "public" | "private" | "collaborative";

/* spotify has three sharing states, not two switches: a collaborative
playlist is always private (the api refuses public + collaborative). one
choice of three says that honestly instead of switches that flip each other. */
const VISIBILITY: { value: Visibility; label: string; hint: string; Icon: typeof Globe }[] = [
  { value: "public", label: "Public", hint: "On your profile and in search", Icon: Globe },
  { value: "private", label: "Private", hint: "Only people with the link", Icon: EyeOff },
  { value: "collaborative", label: "Collaborative", hint: "Friends with the link can add songs", Icon: Users },
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
          <div id="edit-playlist-vis" style={{ fontSize: 12, fontWeight: 600, color: "var(--color-text-dim)", marginBottom: 8 }}>Who can see it</div>
          <div role="radiogroup" aria-labelledby="edit-playlist-vis" style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 8 }}>
            {VISIBILITY.map(({ value, label, hint, Icon }) => {
              const on = visibility === value;
              return (
                <motion.button
                  key={value}
                  type="button"
                  role="radio"
                  aria-checked={on}
                  onClick={() => setVisibility(value)}
                  whileTap={PRESS}
                  transition={PRESS_TRANSITION}
                  className="focus-ring"
                  style={{
                    position: "relative",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "flex-start",
                    gap: 6,
                    padding: "12px 12px 11px",
                    borderRadius: 12,
                    border: "1px solid",
                    borderColor: on ? "color-mix(in srgb, var(--color-accent) 70%, transparent)" : "rgba(255,255,255,0.08)",
                    background: on ? "color-mix(in srgb, var(--color-accent) 16%, transparent)" : "rgba(255,255,255,0.03)",
                    color: "var(--color-text-hi)",
                    textAlign: "left",
                    cursor: "pointer",
                    transition: "background 0.16s ease, border-color 0.16s ease",
                    minWidth: 0,
                  }}
                >
                  <span style={{ display: "flex", width: "100%", alignItems: "center", justifyContent: "space-between" }}>
                    <Icon size={17} strokeWidth={1.9} style={{ color: on ? "var(--color-accent)" : "var(--color-text-dim)" }} />
                    {on && (
                      <motion.span
                        layoutId="edit-playlist-vis-check"
                        transition={SPRING}
                        style={{ width: 18, height: 18, borderRadius: 99, background: "var(--color-accent)", display: "flex", alignItems: "center", justifyContent: "center" }}
                      >
                        <Check size={11} strokeWidth={3} style={{ color: "#fff" }} />
                      </motion.span>
                    )}
                  </span>
                  <span style={{ fontSize: 13.5, fontWeight: 650 }}>{label}</span>
                  <span className="t-caption" style={{ fontSize: 11.5, lineHeight: 1.35, color: "var(--color-text-dim)" }}>{hint}</span>
                </motion.button>
              );
            })}
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
