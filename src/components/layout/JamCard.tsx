import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Users, Link2, X } from "@/lib/icons";
import { getJam, joinJam, leaveJam, startJam, type JamSession } from "../../api/social";
import { CoverArt } from "../ui/CoverArt";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    toast("Jam link copied");
  } catch {
    toast.error("Couldn't copy the link");
  }
}

/* a spotify jam: listen together with friends. start one here and share the
link, join one from a link, see who's in, end or leave it. */
export function JamCard() {
  const qc = useQueryClient();
  const { data: jam } = useQuery({
    queryKey: ["jam"],
    queryFn: getJam,
    refetchInterval: 60_000,
    retry: false,
  });
  // members joining/leaving arrive as dealer pushes
  useEffect(() => {
    let off: (() => void) | null = null;
    let gone = false;
    listen("social:jam-updated", () => qc.invalidateQueries({ queryKey: ["jam"] }))
      .then((u) => (gone ? u() : (off = u)))
      .catch(() => {});
    return () => {
      gone = true;
      off?.();
    };
  }, [qc]);
  const [busy, setBusy] = useState(false);
  const [joining, setJoining] = useState(false);
  const [link, setLink] = useState("");

  async function run(fn: () => Promise<JamSession | null | void>, fail: string) {
    setBusy(true);
    try {
      const next = await fn();
      qc.setQueryData(["jam"], next ?? null);
      qc.invalidateQueries({ queryKey: ["jam"] });
      return next;
    } catch (e) {
      toast.error(`${fail}: ${errMsg(e)}`);
    } finally {
      setBusy(false);
    }
  }

  const box: React.CSSProperties = {
    margin: "4px 12px 10px",
    padding: "12px 12px",
    borderRadius: 12,
    background: "var(--color-glass)",
    border: "1px solid var(--color-glass-border)",
    display: "flex",
    flexDirection: "column",
    gap: 8,
  };

  if (!jam) {
    return (
      <div style={box}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)" }}>
          <Users size={15} /> Jam
        </div>
        <span className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
          Listen together. Everyone in a Jam can add to the queue.
        </span>
        {joining ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (!link.trim()) return;
              run(() => joinJam(link), "Couldn't join the Jam").then((j) => {
                if (j !== undefined) {
                  setJoining(false);
                  setLink("");
                }
              });
            }}
            style={{ display: "flex", gap: 6 }}
          >
            <input
              value={link}
              onChange={(e) => setLink(e.target.value)}
              placeholder="Paste a Jam link"
              aria-label="Jam link"
              autoFocus
              className="focus-ring"
              style={{ flex: 1, minWidth: 0, height: 30, borderRadius: 8, border: "1px solid var(--color-border)", background: "rgba(0,0,0,0.25)", color: "var(--color-text-hi)", padding: "0 8px", fontSize: 12.5 }}
            />
            <button type="submit" className="btn-primary" disabled={busy} style={{ height: 30 }}>Join</button>
          </form>
        ) : (
          <div style={{ display: "flex", gap: 6 }}>
            <button
              type="button"
              className="btn-primary"
              disabled={busy}
              onClick={() =>
                run(startJam, "Couldn't start a Jam").then((j) => {
                  if (j && j.join_url) copy(j.join_url);
                })
              }
            >
              Start a Jam
            </button>
            <button type="button" className="btn-pill" onClick={() => setJoining(true)}>Join</button>
          </div>
        )}
      </div>
    );
  }

  return (
    <div style={box}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 8 }}>
        <span style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)" }}>
          <Users size={15} active /> {jam.is_host ? "Your Jam" : "In a Jam"}
        </span>
        <button
          type="button"
          className="btn-icon"
          aria-label={jam.is_host ? "End Jam" : "Leave Jam"}
          title={jam.is_host ? "End Jam" : "Leave Jam"}
          disabled={busy}
          onClick={() => run(() => leaveJam(jam.session_id, jam.is_host).then(() => null), "Couldn't end the Jam")}
          style={{ width: 24, height: 24, borderRadius: 6 }}
        >
          <X size={13} />
        </button>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {jam.members.map((m) => (
          <div key={m.id} style={{ display: "flex", alignItems: "center", gap: 8 }}>
            {m.image_url ? (
              <CoverArt url={m.image_url} alt="" size={24} rounded style={{ width: 24, height: 24 }} />
            ) : (
              <span style={{ width: 24, height: 24, borderRadius: "50%", background: "var(--color-surface-2)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 11, fontWeight: 700 }}>
                {m.name.slice(0, 1).toUpperCase()}
              </span>
            )}
            <span style={{ flex: 1, minWidth: 0, fontSize: 12.5, color: "var(--color-text)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {m.name}
            </span>
            {m.is_host && <span className="t-caption" style={{ fontSize: 11, color: "var(--color-accent)" }}>Host</span>}
          </div>
        ))}
      </div>
      {jam.join_url && (
        <button type="button" className="btn-pill" onClick={() => copy(jam.join_url!)} style={{ display: "inline-flex", alignItems: "center", gap: 6, alignSelf: "flex-start" }}>
          <Link2 size={13} /> Copy invite link
        </button>
      )}
    </div>
  );
}
