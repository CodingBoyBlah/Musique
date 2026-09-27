import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion } from "framer-motion";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Users, Link2, ArrowRight } from "@/lib/icons";
import { getJam, joinJam, leaveJam, startJam, type JamSession } from "../../api/social";
import { CoverArt } from "../ui/CoverArt";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import { EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    toast("Invite link copied");
  } catch {
    toast.error("Couldn't copy the link");
  }
}

// overlapping avatars, capped with a "+n"
function Faces({ members }: { members: JamSession["members"] }) {
  const shown = members.slice(0, 4);
  return (
    <div style={{ display: "flex", alignItems: "center" }}>
      {shown.map((m, i) => (
        <span
          key={m.id}
          title={m.name}
          style={{
            width: 28,
            height: 28,
            borderRadius: "50%",
            marginLeft: i === 0 ? 0 : -8,
            boxShadow: "0 0 0 2px var(--color-popover, #1c1c22)",
            overflow: "hidden",
            background: "var(--color-surface-2)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 11,
            fontWeight: 700,
            color: "var(--color-text)",
            position: "relative",
            zIndex: shown.length - i,
          }}
        >
          {m.image_url ? <CoverArt url={m.image_url} alt="" size={28} rounded style={{ width: 28, height: 28 }} /> : m.name.slice(0, 1).toUpperCase()}
        </span>
      ))}
      {members.length > shown.length && (
        <span className="tnum" style={{ marginLeft: 6, fontSize: 11.5, color: "var(--color-text-dim)" }}>+{members.length - shown.length}</span>
      )}
    </div>
  );
}

/* spotify jam: listen together. idle it's a quiet invitation; in a jam it
shows who's there, the invite link and the way out. */
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

  const card: React.CSSProperties = {
    margin: "6px 10px 4px",
    padding: 12,
    borderRadius: 14,
    display: "flex",
    flexDirection: "column",
    gap: 10,
    background: jam
      ? "linear-gradient(135deg, color-mix(in srgb, var(--color-accent) 26%, transparent), color-mix(in srgb, var(--color-accent) 8%, transparent))"
      : "var(--color-glass)",
    border: "1px solid",
    borderColor: jam ? "color-mix(in srgb, var(--color-accent) 35%, transparent)" : "var(--color-glass-border)",
  };

  if (jam) {
    const others = jam.members.length - 1;
    return (
      <div style={card}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <Faces members={jam.members} />
          <div style={{ minWidth: 0, flex: 1 }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)" }}>{jam.is_host ? "Your Jam" : "In a Jam"}</div>
            <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text)" }}>
              {others <= 0 ? "Waiting for friends" : `You + ${others} ${others === 1 ? "friend" : "friends"}`}
            </div>
          </div>
        </div>
        <div style={{ display: "flex", gap: 6 }}>
          {jam.join_url && (
            <motion.button
              type="button"
              className="btn-primary"
              whileTap={PRESS}
              transition={PRESS_TRANSITION}
              onClick={() => copy(jam.join_url!)}
              style={{ flex: 1, height: 30, display: "inline-flex", alignItems: "center", justifyContent: "center", gap: 6, fontSize: 12.5 }}
            >
              <Link2 size={13} /> Invite
            </motion.button>
          )}
          <button
            type="button"
            className="btn-pill"
            disabled={busy}
            onClick={() => run(() => leaveJam(jam.session_id, jam.is_host).then(() => null), "Couldn't end the Jam")}
            style={{ height: 30, fontSize: 12.5 }}
          >
            {jam.is_host ? "End" : "Leave"}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div style={card}>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <span style={{ width: 32, height: 32, borderRadius: "50%", flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "color-mix(in srgb, var(--color-accent) 22%, transparent)", color: "var(--color-accent)" }}>
          <Users size={15} />
        </span>
        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)" }}>Start a Jam</div>
          <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>Listen together, everyone adds to the queue</div>
        </div>
      </div>
      <AnimatePresence initial={false} mode="wait">
        {joining ? (
          <motion.form
            key="join"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.16, ease: EASE_OUT }}
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
              onKeyDown={(e) => e.key === "Escape" && setJoining(false)}
              placeholder="Paste invite link"
              aria-label="Jam invite link"
              autoFocus
              className="focus-ring"
              style={{ flex: 1, minWidth: 0, height: 30, borderRadius: 99, border: "1px solid var(--color-border)", background: "rgba(0,0,0,0.25)", color: "var(--color-text-hi)", padding: "0 12px", fontSize: 12.5 }}
            />
            <button type="submit" className="btn-primary" aria-label="Join" disabled={busy || !link.trim()} style={{ height: 30, width: 30, padding: 0, display: "flex", alignItems: "center", justifyContent: "center" }}>
              <ArrowRight size={14} />
            </button>
          </motion.form>
        ) : (
          <motion.div
            key="actions"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.16, ease: EASE_OUT }}
            style={{ display: "flex", gap: 6 }}
          >
            <motion.button
              type="button"
              className="btn-primary"
              disabled={busy}
              whileTap={PRESS}
              transition={PRESS_TRANSITION}
              onClick={() =>
                run(startJam, "Couldn't start a Jam").then((j) => {
                  if (j && j.join_url) copy(j.join_url);
                })
              }
              style={{ flex: 1, height: 30, fontSize: 12.5 }}
            >
              {busy ? "Starting..." : "Start"}
            </motion.button>
            <button type="button" className="btn-pill" onClick={() => setJoining(true)} style={{ height: 30, fontSize: 12.5 }}>
              Join with link
            </button>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
