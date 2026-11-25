import { useEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { widevineInstall, widevineLicenseText, widevineStatus, type WidevineProgress } from "../../api/widevine";
import { useIsApple } from "../../hooks/useProvider";
import { useAuthStore } from "../../store/auth.store";
import { useWidevineStore } from "../../store/widevine.store";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";
import { ChevronDown, Loader2 } from "../../lib/icons";
import { Modal } from "./Modal";

export const WIDEVINE_STATUS_KEY = ["widevine-status"] as const;

const formatMb = (bytes: number) => `${(bytes / 1_048_576).toFixed(1)} MB`;

/**
 * Asks before downloading Google's Widevine module, which Apple Music needs to
 * play its protected tracks. Opens by itself once after an Apple Music sign-in
 * when the module is missing; playback opens it again when a track needs it.
 */
export function WidevineSetup() {
  const isAppleAccount = useIsApple();
  const loggedIn = useAuthStore((s) => s.loggedIn);
  const open = useWidevineStore((s) => s.open);
  const dismissed = useWidevineStore((s) => s.dismissed);
  const show = useWidevineStore((s) => s.show);
  const close = useWidevineStore((s) => s.close);
  const qc = useQueryClient();
  const installRef = useRef<HTMLButtonElement>(null);
  const [termsOpen, setTermsOpen] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<WidevineProgress | null>(null);

  const { data: status } = useQuery({
    queryKey: WIDEVINE_STATUS_KEY,
    queryFn: widevineStatus,
    enabled: isAppleAccount && loggedIn,
    staleTime: 5 * 60_000,
  });

  const { data: terms, isLoading: termsLoading, error: termsError } = useQuery({
    queryKey: ["widevine-terms"],
    queryFn: widevineLicenseText,
    enabled: open && termsOpen,
    staleTime: Infinity,
  });

  // the first time an Apple Music account is here without the module, ask once
  useEffect(() => {
    if (isAppleAccount && loggedIn && status?.supported && !status.installed && !dismissed) show();
  }, [isAppleAccount, loggedIn, status, dismissed, show]);

  const install = async () => {
    setInstalling(true);
    setProgress(null);
    try {
      const next = await widevineInstall(setProgress);
      qc.setQueryData(WIDEVINE_STATUS_KEY, next);
      toast("Widevine is installed. Apple Music is ready to play.");
      close();
    } catch (e) {
      toast.error(`Could not install Widevine: ${errMsg(e)}`);
    } finally {
      setInstalling(false);
    }
  };

  const unsupported = status !== undefined && !status.supported;
  const percent = progress?.total ? Math.min(100, Math.round((progress.done / progress.total) * 100)) : null;

  return (
    <Modal
      open={open}
      onClose={() => { if (!installing) close(); }}
      labelledBy="widevine-setup-title"
      initialFocus={installRef}
      panelStyle={{ width: "min(440px, calc(100vw - 32px))", padding: "22px 22px 18px" }}
    >
      <h2 id="widevine-setup-title" style={{ margin: 0, fontSize: 16, fontWeight: 700, color: "var(--color-text-hi)" }}>
        One more step to play Apple Music
      </h2>
      <p style={{ margin: "8px 0 0", fontSize: 13, lineHeight: 1.55, color: "var(--color-text-dim)" }}>
        {unsupported
          ? "Apple Music protects its tracks with Widevine, which isn't available for this system. You can still browse and edit your library, but tracks won't play."
          : "Apple Music protects its tracks with Widevine, the same module browsers use. Musique downloads it once from Google and keeps it on this computer."}
      </p>

      {!unsupported && (
        <>
          <button
            type="button"
            className="focus-ring"
            onClick={() => setTermsOpen((v) => !v)}
            aria-expanded={termsOpen}
            style={{
              display: "inline-flex", alignItems: "center", gap: 4, marginTop: 12, padding: "2px 0",
              border: 0, background: "none", color: "var(--color-text-dim)", fontSize: 12, cursor: "pointer",
            }}
          >
            Google's terms
            <ChevronDown
              size={12}
              aria-hidden="true"
              style={{ transform: termsOpen ? "rotate(180deg)" : undefined, transition: "transform 160ms" }}
            />
          </button>
          {termsOpen && (
            <pre
              style={{
                margin: "8px 0 0", maxHeight: 180, overflow: "auto", padding: "10px 12px", borderRadius: 10,
                background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.08)",
                color: "var(--color-text-dim)", fontSize: 11, lineHeight: 1.5, whiteSpace: "pre-wrap", fontFamily: "inherit",
              }}
            >
              {termsLoading ? "Loading..." : termsError ? `Could not load the terms: ${errMsg(termsError)}` : terms}
            </pre>
          )}
          <p style={{ margin: "10px 0 0", fontSize: 11.5, lineHeight: 1.5, color: "var(--color-text-dim)" }}>
            By installing, you agree to Google's terms for Widevine.
          </p>
        </>
      )}

      {installing && (
        <div role="status" aria-live="polite" style={{ marginTop: 16 }}>
          <div style={{ height: 4, borderRadius: 2, background: "rgba(255,255,255,0.08)", overflow: "hidden" }}>
            <div
              style={{
                height: "100%", borderRadius: 2, background: "var(--color-accent)",
                width: percent === null ? "35%" : `${percent}%`, transition: "width 200ms linear",
              }}
            />
          </div>
          <p style={{ margin: "6px 0 0", fontSize: 11.5, color: "var(--color-text-dim)", fontVariantNumeric: "tabular-nums" }}>
            {progress
              ? `${formatMb(progress.done)}${progress.total ? ` of ${formatMb(progress.total)}` : ""}`
              : "Asking Google for the module..."}
          </p>
        </div>
      )}

      <div style={{ display: "flex", gap: 10, justifyContent: "flex-end", marginTop: 20 }}>
        <button
          type="button"
          onClick={close}
          disabled={installing}
          className="btn-pill"
          style={{ height: 38, padding: "0 18px" }}
        >
          {unsupported ? "OK" : "Not now"}
        </button>
        {!unsupported && (
          <button
            ref={installRef}
            type="button"
            onClick={() => void install()}
            disabled={installing}
            aria-busy={installing || undefined}
            className="btn-primary"
            style={{ height: 38, padding: "0 18px", display: "inline-flex", alignItems: "center", gap: 8 }}
          >
            {installing && <Loader2 size={15} aria-hidden="true" />}
            {installing ? "Installing..." : "Install Widevine"}
          </button>
        )}
      </div>
    </Modal>
  );
}
