import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQueryClient } from "@tanstack/react-query";
import { ListMusic, Plus, Search, X } from "lucide-react";
import { useAddToPlaylistStore } from "../../store/addToPlaylist.store";
import { useMyPlaylists, LIBRARY_KEYS } from "../../hooks/useLibrary";
import { addTrackToPlaylist, createPlaylist } from "../../api/library";
import { toast } from "../../store/toast.store";

/* global "add to playlist" picker. opened from any tracks context menu via
 useAddToPlaylistStore. lists the users playlists -- clicking one writes the
track through to spotify. can also spin up a new playlist on demand. */
export function AddToPlaylistModal() {
  const track   = useAddToPlaylistStore((s) => s.track);
  const close   = useAddToPlaylistStore((s) => s.close);
  const { data: playlists = [], isLoading } = useMyPlaylists();
  const qc = useQueryClient();

  const [filter, setFilter] = useState("");
  const [busy, setBusy]     = useState(false);

  const open = !!track;
  const shown = playlists.filter((p) => p.name.toLowerCase().includes(filter.trim().toLowerCase()));

  async function add(playlistId: string, playlistName: string) {
    if (!track || busy) return;
    setBusy(true);
    try {
      await addTrackToPlaylist(playlistId, track.id);
      toast(`Added to ${playlistName}`);
      qc.invalidateQueries({ queryKey: ["playlist", playlistId] });
      qc.invalidateQueries({ queryKey: LIBRARY_KEYS.playlists });
      close();
    } catch {
      toast("Couldn't add to playlist");
    } finally {
      setBusy(false);
    }
  }

  async function createAndAdd() {
    if (!track || busy) return;
    const name = filter.trim() || `${track.name} mix`;
    setBusy(true);
    try {
      const id = await createPlaylist(name, null, false);
      await addTrackToPlaylist(id, track.id);
      toast(`Created “${name}” and added`);
      qc.invalidateQueries({ queryKey: LIBRARY_KEYS.playlists });
      close();
    } catch {
      toast("Couldn't create playlist");
    } finally {
      setBusy(false);
    }
  }

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.14 }}
          onClick={close}
          style={{
            position: "fixed", inset: 0, zIndex: 1050,
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
              width: 400, maxHeight: "70vh", display: "flex", flexDirection: "column",
              borderRadius: 16, background: "rgba(20,20,26,0.97)",
              border: "1px solid rgba(255,255,255,0.12)", boxShadow: "0 24px 64px rgba(0,0,0,0.6)",
              overflow: "hidden",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "16px 18px 12px" }}>
              <h2 style={{ flex: 1, margin: 0, fontSize: 15, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                Add “{track?.name}” to…
              </h2>
              <button onClick={close} title="Close" style={iconBtn}><X size={16} /></button>
            </div>

            <div style={{ padding: "0 18px 12px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: 8, height: 36, padding: "0 12px", borderRadius: 9, background: "rgba(0,0,0,0.25)", border: "1px solid var(--color-border)" }}>
                <Search size={14} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
                <input
                  autoFocus
                  value={filter}
                  onChange={(e) => setFilter(e.target.value)}
                  placeholder="Find or name a new playlist"
                  spellCheck={false}
                  style={{ flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none", background: "transparent", color: "var(--color-text-hi)", fontSize: 13.5, fontFamily: "inherit" }}
                />
              </div>
            </div>

            <div style={{ flex: 1, overflowY: "auto", padding: "0 10px 10px" }}>
              <button onClick={createAndAdd} disabled={busy} style={rowBtn} onMouseEnter={hoverOn} onMouseLeave={hoverOff}>
                <span style={{ ...thumb, background: "var(--color-accent-dim)", color: "var(--color-accent)" }}><Plus size={18} /></span>
                <span style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)" }}>
                  New playlist{filter.trim() ? ` “${filter.trim()}”` : ""}
                </span>
              </button>

              {isLoading ? (
                <p style={hint}>Loading playlists…</p>
              ) : shown.length === 0 ? (
                <p style={hint}>No matching playlists.</p>
              ) : (
                shown.map((p) => (
                  <button key={p.id} onClick={() => add(p.id, p.name)} disabled={busy} style={rowBtn} onMouseEnter={hoverOn} onMouseLeave={hoverOff}>
                    {p.image_url
                      ? <img src={p.image_url} alt="" style={{ ...thumb, objectFit: "cover" }} />
                      : <span style={{ ...thumb, background: "var(--color-surface-2)" }}><ListMusic size={16} style={{ color: "var(--color-text-dim)" }} /></span>}
                    <span style={{ minWidth: 0, flex: 1, textAlign: "left" }}>
                      <span style={{ display: "block", fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
                      <span style={{ display: "block", fontSize: 11.5, color: "var(--color-text-dim)" }}>{p.total_tracks} tracks</span>
                    </span>
                  </button>
                ))
              )}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

const iconBtn: React.CSSProperties = {
  display: "flex", alignItems: "center", justifyContent: "center", width: 28, height: 28,
  borderRadius: 6, border: "none", background: "transparent", color: "var(--color-text)", cursor: "pointer", flexShrink: 0,
};

const rowBtn: React.CSSProperties = {
  display: "flex", alignItems: "center", gap: 11, width: "100%", padding: "8px 10px",
  border: "none", background: "transparent", borderRadius: 10, cursor: "pointer",
  transition: "background 0.1s",
};
const hoverOn  = (e: React.MouseEvent<HTMLButtonElement>) => { e.currentTarget.style.background = "var(--color-hover)"; };
const hoverOff = (e: React.MouseEvent<HTMLButtonElement>) => { e.currentTarget.style.background = "transparent"; };

const thumb: React.CSSProperties = {
  width: 40, height: 40, borderRadius: 6, flexShrink: 0,
  display: "flex", alignItems: "center", justifyContent: "center",
};

const hint: React.CSSProperties = { margin: 0, padding: "12px 10px", fontSize: 12.5, color: "var(--color-text-dim)" };
