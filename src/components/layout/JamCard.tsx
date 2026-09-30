import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQueryClient } from "@tanstack/react-query";
import { Users, Link2, ArrowRight, X } from "@/lib/icons";
import { joinJam, kickJamMember, leaveJam, setJamQueueOnly, startJam, type JamMember, type JamSession } from "../../api/social";
import { CoverArt } from "../ui/CoverArt";
import { Tooltip } from "../ui/Tooltip";
import { toast } from "../../store/toast.store";
import { useJamStore } from "../../store/jam.store";
import { errMsg } from "../../lib/err";
import { leftJam, refreshConnectState, seedJamFromLocal } from "../../lib/jam";
import { usePlaybackBackend } from "../../hooks/usePlaybackBackend";
import { EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    toast("Invite link copied");
  } catch {
    toast.error("Couldn't copy the link");
  }
}

function Face({ m, size, ring }: { m: JamMember; size: number; ring?: boolean }) {
  return (
    <span
      title={m.name}
      style={{
        width: size,
        height: size,
        borderRadius: "50%",
        flexShrink: 0,
        boxShadow: ring ? "0 0 0 2px var(--color-popover, #1c1c22)" : undefined,
        overflow: "hidden",
        background: "var(--color-surface-2)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: size * 0.4,
        fontWeight: 700,
        color: "var(--color-text)",
        position: "relative",
      }}
    >
      {m.image_url ? <CoverArt url={m.image_url} alt="" size={size} rounded style={{ width: size, height: size }} /> : m.name.slice(0, 1).toUpperCase()}
    </span>
  );
}

// overlapping avatars, capped with a "+n"
function Faces({ members }: { members: JamMember[] }) {
  const shown = members.slice(0, 4);
  return (
    <div style={{ display: "flex", alignItems: "center" }}>
      {shown.map((m, i) => (
        <span key={m.id} style={{ marginLeft: i === 0 ? 0 : -8, zIndex: shown.length - i, display: "flex" }}>
          <Face m={m} size={28} ring />
        </span>
      ))}
      {members.length > shown.length && (
        <span className="tnum" style={{ marginLeft: 6, fontSize: 11.5, color: "var(--color-text-dim)" }}>+{members.length - shown.length}</span>
      )}
    </div>
  );
}

function hostOf(jam: JamSession) {
  return jam.members.find((m) => m.is_host) ?? null;
}

/* spotify jam: listen together. idle it's a quiet invitation; in a jam it
shows who's there, the invite link, the host's controls and the way out. */
export function JamCard() {
  const qc = useQueryClient();
  const jam = useJamStore((s) => s.session);
  const { data: backend } = usePlaybackBackend();
  // jams stream through spotify's own player. the youtube source can't carry one
  const spotifyAudio = !backend || backend.active === "spotify";

  const [busy, setBusy] = useState(false);
  const [joining, setJoining] = useState(false);
  const [showPeople, setShowPeople] = useState(false);
  const [link, setLink] = useState("");

  async function run<T>(fn: () => Promise<T>, fail: string): Promise<T | undefined> {
    setBusy(true);
    try {
      return await fn();
    } catch (e) {
      toast.error(`${fail}: ${errMsg(e)}`);
    } finally {
      setBusy(false);
    }
  }

  function adopt(next: JamSession | null) {
    useJamStore.getState().setSession(next);
    qc.setQueryData(["jam"], next);
  }

  const card: React.CSSProperties = {
    // the panel's rhythm: box 4px from the edge, content on the 14px line
    margin: "6px 4px 4px",
    padding: 10,
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
    const others = jam.members.filter((m) => !m.is_current_user).length || Math.max(0, jam.members.length - 1);
    const host = hostOf(jam);
    const subtitle = others <= 0
      ? "Waiting for friends"
      : jam.is_host
        ? `You + ${others} ${others === 1 ? "friend" : "friends"}`
        : `${jam.members.length} listening`;
    return (
      <div style={card}>
        <button
          type="button"
          onClick={() => setShowPeople((v) => !v)}
          aria-expanded={showPeople}
          className="focus-ring"
          style={{ display: "flex", alignItems: "center", gap: 10, background: "none", border: 0, padding: 0, textAlign: "left", cursor: "pointer", color: "inherit" }}
        >
          <Faces members={jam.members} />
          <div style={{ minWidth: 0, flex: 1 }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {jam.is_host ? "Your Jam" : `${host?.name ?? "Someone"}'s Jam`}
            </div>
            <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text)" }}>{subtitle}</div>
          </div>
        </button>

        <AnimatePresence initial={false}>
          {showPeople && (
            <motion.div
              key="people"
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.18, ease: EASE_OUT }}
              style={{ overflow: "hidden", display: "flex", flexDirection: "column", gap: 6 }}
            >
              {jam.members.map((m) => (
                <div key={m.id} style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <Face m={m} size={22} />
                  <span style={{ flex: 1, minWidth: 0, fontSize: 12.5, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {m.is_current_user ? "You" : m.name}
                  </span>
                  {m.is_host && <span className="t-caption" style={{ fontSize: 11, color: "var(--color-text-dim)" }}>Host</span>}
                  {jam.is_host && !m.is_host && !m.is_current_user && (
                    <Tooltip label={`Remove ${m.name}`} side="top" align="end">
                      <button
                        type="button"
                        className="btn-icon"
                        aria-label={`Remove ${m.name} from the Jam`}
                        disabled={busy}
                        onClick={() => run(() => kickJamMember(jam.session_id, m.id), `Couldn't remove ${m.name}`)}
                        style={{ width: 22, height: 22, borderRadius: 6 }}
                      >
                        <X size={12} strokeWidth={2.4} />
                      </button>
                    </Tooltip>
                  )}
                </div>
              ))}
              {jam.is_host && (
                <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 12, color: "var(--color-text)", cursor: "pointer", paddingTop: 2 }}>
                  <input
                    type="checkbox"
                    checked={!jam.queue_only_mode}
                    disabled={busy}
                    onChange={(e) =>
                      run(() => setJamQueueOnly(!e.target.checked), "Couldn't change the Jam").then((next) => {
                        if (next) adopt(next);
                      })
                    }
                  />
                  Guests can play and skip songs
                </label>
              )}
              {!jam.is_host && jam.queue_only_mode && (
                <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>
                  You can add songs. {host?.name ?? "The host"} controls what plays.
                </div>
              )}
            </motion.div>
          )}
        </AnimatePresence>

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
            onClick={() =>
              run(async () => {
                await leaveJam(jam.session_id, jam.is_host);
                await leftJam(jam.is_host);
                qc.setQueryData(["jam"], null);
              }, jam.is_host ? "Couldn't end the Jam" : "Couldn't leave the Jam")
            }
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
          <div className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>
            {spotifyAudio ? "Listen together, everyone adds to the queue" : "Jams play through Spotify. Switch the audio source in Settings."}
          </div>
        </div>
      </div>
      {spotifyAudio && (
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
                  if (j === undefined) return;
                  setJoining(false);
                  setLink("");
                  adopt(j);
                  if (j) {
                    toast(`Joined ${hostOf(j)?.name ?? "the"}${hostOf(j) ? "'s" : ""} Jam`);
                    // social-connect hands the jam's playback over right after
                    // the join; pick it up once it has landed
                    setTimeout(() => refreshConnectState(true), 1500);
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
                  run(async () => {
                    // the jam starts with what's playing and what's queued here
                    await seedJamFromLocal().catch(() => {});
                    return startJam();
                  }, "Couldn't start a Jam").then((j) => {
                    if (!j) return;
                    adopt(j);
                    refreshConnectState(true);
                    if (j.join_url) copy(j.join_url);
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
      )}
    </div>
  );
}
