import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, Music, RefreshCw, X } from "@/lib/icons";
import { useYtMatchStore } from "../../store/ytMatch.store";
import {
  forgetYtMatch,
  getYtMatch,
  pinYtMatch,
  searchYtCandidates,
} from "../../api/playback";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import { fmtMs } from "../../utils/fmt";
import { Modal } from "./Modal";

/**
 * "Which YouTube upload is this?" picker.
 *
 * Exists because the matcher deliberately refuses rather than guessing: when it
 * finds nothing acceptable, nothing plays. That's the right default, but it
 * would be a dead end without a way for the user to look at what was rejected
 * and decide for themselves.
 *
 * The candidate list here is intentionally **ungated** - it's the raw search
 * result, including the instrumentals and live cuts the matcher threw out. The
 * whole point is to show what was rejected. Automatic playback never touches
 * this list; picking from it writes a pinned override that survives
 * re-resolution.
 */
export function YtMatchModal() {
  const trackId = useYtMatchStore((s) => s.trackId);
  const label   = useYtMatchStore((s) => s.label);
  const close   = useYtMatchStore((s) => s.close);
  const qc      = useQueryClient();

  const [busy, setBusy] = useState(false);
  const open = !!trackId;

  const { data: match } = useQuery({
    queryKey: ["yt-match", trackId],
    queryFn:  () => getYtMatch(trackId!),
    enabled:  open,
  });

  const { data: candidates = [], isLoading, isError, error } = useQuery({
    queryKey: ["yt-candidates", trackId],
    queryFn:  () => searchYtCandidates(trackId!),
    enabled:  open,
    staleTime: 60_000,
  });

  async function pick(videoId: string) {
    if (!trackId || busy) return;
    setBusy(true);
    try {
      await pinYtMatch(trackId, videoId);
      qc.invalidateQueries({ queryKey: ["yt-match", trackId] });
      toast("Match pinned - this track will always use that upload");
      close();
    } catch (e) {
      toast.error(errMsg(e));
    } finally {
      setBusy(false);
    }
  }

  async function reset() {
    if (!trackId || busy) return;
    setBusy(true);
    try {
      await forgetYtMatch(trackId);
      qc.invalidateQueries({ queryKey: ["yt-match", trackId] });
      toast("Match cleared - it'll be resolved again on next play");
      close();
    } catch (e) {
      toast.error(errMsg(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      open={open}
      onClose={close}
      labelledBy="yt-match-title"
      panelStyle={{
        width: "min(520px, calc(100vw - 32px))", maxHeight: "74vh",
        display: "flex", flexDirection: "column", overflow: "hidden",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "16px 18px 10px" }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <h2 id="yt-match-title" style={{ margin: 0, fontSize: 15, fontWeight: 700, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            YouTube Music source
          </h2>
          <p className="t-caption" style={{ margin: "2px 0 0", fontSize: 12, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {label ?? "this track"}
          </p>
        </div>
        <button onClick={close} aria-label="Close" title="Close" className="btn-icon" style={iconBtn}><X size={16} /></button>
      </div>

      {/* Current state: matched, refused, or not yet resolved. */}
      <div style={{ padding: "0 18px 12px" }}>
        <div style={{
          padding: "9px 12px", borderRadius: 9,
          background: "rgba(0,0,0,0.25)", border: "1px solid var(--color-border)",
          fontSize: 12, color: "var(--color-text-dim)", lineHeight: 1.45,
        }}>
          {match?.video_id ? (
            <>
              Currently using <strong style={{ color: "var(--color-text-hi)", fontWeight: 600 }}>{match.video_id}</strong>
              {match.pinned && <span style={{ color: "var(--color-accent)" }}> (pinned by you)</span>}
              {match.reason && <><br />{match.reason}</>}
            </>
          ) : match ? (
            <>
              No acceptable match was found, so this track won't play.
              {match.reason && <><br />{match.reason}</>}
            </>
          ) : (
            "Not resolved yet - it'll be matched the first time you play it."
          )}
        </div>
      </div>

      <div style={{ padding: "0 18px 8px", fontSize: 11.5, color: "var(--color-text-dim)" }}>
        All search results, including ones automatic matching rejected.
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "0 10px 10px" }}>
        {isLoading ? (
          <p style={hint}>Searching YouTube Music…</p>
        ) : isError ? (
          <p style={hint}>{errMsg(error)}</p>
        ) : candidates.length === 0 ? (
          <p style={hint}>No results.</p>
        ) : (
          candidates.map((c) => {
            const active = c.video_id === match?.video_id;
            return (
              <button
                key={c.video_id}
                onClick={() => pick(c.video_id)}
                disabled={busy}
                className="row-btn"
                data-active={active}
                aria-current={active || undefined}
                style={rowBtn}
              >
                <span style={{ ...thumb, background: "var(--color-surface-2)" }}>
                  {active
                    ? <Check size={16} style={{ color: "var(--color-accent)" }} />
                    : <Music size={15} style={{ color: "var(--color-text-dim)" }} />}
                </span>
                <span style={{ minWidth: 0, flex: 1, textAlign: "left" }}>
                  <span style={{ display: "block", fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {c.title}
                    {c.explicit && <span style={explicitTag}>E</span>}
                  </span>
                  <span className="t-caption tnum" style={{ display: "block", fontSize: 11.5, color: "var(--color-text-dim)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {[c.artists.join(", "), c.album, c.duration_ms != null ? fmtMs(c.duration_ms) : null]
                      .filter(Boolean)
                      .join("  ·  ")}
                  </span>
                </span>
              </button>
            );
          })
        )}
      </div>

      <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, padding: "10px 18px 14px", borderTop: "1px solid var(--color-border)" }}>
        <button onClick={reset} disabled={busy || !match} className="btn-pill" style={secondaryBtn}>
          <RefreshCw size={13} strokeWidth={2.2} />
          Clear &amp; re-match
        </button>
      </div>
    </Modal>
  );
}

const iconBtn: React.CSSProperties = {
  width: 28, height: 28, borderRadius: 6, color: "var(--color-text)",
};

const rowBtn: React.CSSProperties = {
  gap: 11, padding: "8px 10px", borderRadius: 10,
};

const thumb: React.CSSProperties = {
  width: 36, height: 36, borderRadius: 6, flexShrink: 0,
  display: "flex", alignItems: "center", justifyContent: "center",
};

const explicitTag: React.CSSProperties = {
  display: "inline-flex", alignItems: "center", justifyContent: "center",
  marginLeft: 6, padding: "0 4px", height: 13, borderRadius: 3,
  background: "var(--color-surface-2)", color: "var(--color-text-dim)",
  fontSize: 9, fontWeight: 700, verticalAlign: "middle",
};

const secondaryBtn: React.CSSProperties = {
  height: 32, padding: "0 13px", borderRadius: 8, fontSize: 12.5,
};

const hint: React.CSSProperties = { margin: 0, padding: "12px 10px", fontSize: 12.5, color: "var(--color-text-dim)" };
