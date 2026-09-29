import { useRef, useState } from "react";
import { coverUrl } from "../../lib/coverUrl";
import { useQueryClient } from "@tanstack/react-query";
import { ListMusic, Plus, Search, X } from "@/lib/icons";
import { useAddToPlaylistStore } from "../../store/addToPlaylist.store";
import { useMyPlaylists, LIBRARY_KEYS } from "../../hooks/useLibrary";
import { addTrackToPlaylist, createPlaylist } from "../../api/library";
import { toast } from "../../store/toast.store";
import { Modal } from "./Modal";

/* global "add to playlist" picker. opened from any tracks context menu via
 useAddToPlaylistStore. lists the users playlists -- clicking one writes the
track through to spotify. can also spin up a new playlist on demand. */
export function AddToPlaylistModal() {
  const track   = useAddToPlaylistStore((s) => s.track);
  const close   = useAddToPlaylistStore((s) => s.close);
  const { data: playlists = [], isLoading } = useMyPlaylists();
  const qc = useQueryClient();
  const inputRef = useRef<HTMLInputElement>(null);

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
      toast.error("Couldn't add to playlist");
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
      toast.error("Couldn't create playlist");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      open={open}
      onClose={close}
      labelledBy="add-to-playlist-title"
      initialFocus={inputRef}
      panelStyle={{
        width: "min(400px, calc(100vw - 32px))", maxHeight: "70vh", display: "flex", flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "16px 18px 12px" }}>
        <h2 id="add-to-playlist-title" style={{ flex: 1, margin: 0, fontSize: 15, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          Add “{track?.name}” to…
        </h2>
        <button onClick={close} aria-label="Close" title="Close" className="btn-icon" style={{ width: 28, height: 28, borderRadius: 6, color: "var(--color-text)" }}>
          <X size={16} />
        </button>
      </div>

      <div style={{ padding: "0 18px 12px" }}>
        <div className="focus-within-ring" style={{ display: "flex", alignItems: "center", gap: 8, height: 36, padding: "0 12px", borderRadius: 9, background: "rgba(0,0,0,0.25)", border: "1px solid var(--color-border)", transition: "border-color 0.14s ease, box-shadow 0.14s ease" }}>
          <Search size={14} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} />
          <input
            ref={inputRef}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            onKeyDown={(e) => {
              // Enter only acts when the intent is unambiguous, since both
              // outcomes write to the user's Spotify account: an exact name
              // (or the single match of a typed filter) adds; a name nothing
              // matches creates. Never before the list has loaded - an empty
              // list would read as "no match" and create a duplicate.
              if (e.key !== "Enter" || busy || isLoading) return;
              e.preventDefault();
              const q = filter.trim().toLowerCase();
              if (!q) return;
              const exact = playlists.find((p) => p.name.toLowerCase() === q);
              const target = exact ?? (shown.length === 1 ? shown[0] : undefined);
              if (target) add(target.id, target.name);
              else if (shown.length === 0) createAndAdd();
            }}
            placeholder="Find or name a new playlist"
            aria-label="Find or name a new playlist"
            spellCheck={false}
            style={{ flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none", background: "transparent", color: "var(--color-text-hi)", fontSize: 13.5, fontFamily: "inherit" }}
          />
        </div>
      </div>

      {/* 8 + the rows' 10px = the 18px edge the title and search box sit on */}
      <div style={{ flex: 1, overflowY: "auto", padding: "0 8px 10px" }}>
        <button onClick={createAndAdd} disabled={busy} className="row-btn" style={rowBtn}>
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
            <button key={p.id} onClick={() => add(p.id, p.name)} disabled={busy} className="row-btn" style={rowBtn}>
              {p.image_url
                ? <img src={coverUrl(p.image_url, 40) ?? p.image_url} alt="" style={{ ...thumb, objectFit: "cover" }} />
                : <span style={{ ...thumb, background: "var(--color-surface-2)" }}><ListMusic size={16} style={{ color: "var(--color-text-dim)" }} /></span>}
              <span style={{ minWidth: 0, flex: 1, textAlign: "left" }}>
                <span style={{ display: "block", fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
                <span className="t-caption tnum" style={{ display: "block", fontSize: 11.5, color: "var(--color-text-dim)" }}>{p.total_tracks} tracks</span>
              </span>
            </button>
          ))
        )}
      </div>
    </Modal>
  );
}

const rowBtn: React.CSSProperties = {
  gap: 11, padding: "8px 10px", borderRadius: 10,
};

const thumb: React.CSSProperties = {
  width: 40, height: 40, borderRadius: 6, flexShrink: 0,
  display: "flex", alignItems: "center", justifyContent: "center",
};

const hint: React.CSSProperties = { margin: 0, padding: "12px 10px", fontSize: 12.5, color: "var(--color-text-dim)" };
