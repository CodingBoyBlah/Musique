import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Eye, EyeOff, ExternalLink } from "lucide-react";
import {
  getCredentials,
  saveCredentials,
  validateCredentials,
  clearCredentials,
} from "../api/credentials";
import {
  useCredentialsStore,
  type ConnectionStatus,
} from "../store/credentials.store";
import { usePrefsStore } from "../store/prefs.store";
import { useUIStore } from "../store/ui.store";
import { setDiscordEnabled } from "../api/media";
import {
  lastfmStatus, lastfmSaveApi, lastfmStartAuth, lastfmFinishAuth,
  lastfmDisconnect, lastfmClear,
} from "../api/lastfm";
import { setWindowEffect as applyWindowEffect, type WindowEffect } from "../api/window";
import { isWindows } from "../lib/platform";

const STATUS_CONFIG: Record<ConnectionStatus, { dot: string; label: string }> = {
  unconfigured: { dot: "rgba(255,255,255,0.30)", label: "Not configured" },
  configured:   { dot: "#f5a623",                label: "Saved — not tested" },
  validating:   { dot: "#fa2d48",                label: "Validating…" },
  valid:        { dot: "#34d399",                label: "Connected" },
  invalid:      { dot: "#ff453a",                label: "Invalid credentials" },
};

function StatusBadge({ status }: { status: ConnectionStatus }) {
  const { dot, label } = STATUS_CONFIG[status];
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <span style={{ width: 8, height: 8, borderRadius: "50%", background: dot, flexShrink: 0 }} />
      <span style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>{label}</span>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  flex:         1,
  height:       42,
  borderRadius: 10,
  border:       "1px solid var(--color-border)",
  background:   "rgba(0,0,0,0.20)",
  color:        "var(--color-text-hi)",
  fontSize:     14,
  padding:      "0 14px",
  outline:      "none",
  fontFamily:   "inherit",
};

function Field({
  label, children, hint,
}: {
  label: string; children: React.ReactNode; hint?: string;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
      <label style={{ fontSize: 12.5, fontWeight: 600, color: "var(--color-text)" }}>{label}</label>
      <div style={{ display: "flex", gap: 8 }}>{children}</div>
      {hint && <span style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>{hint}</span>}
    </div>
  );
}

function PrimaryBtn({ children, onClick, disabled }: { children: React.ReactNode; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        height: 40, padding: "0 22px", borderRadius: 99, border: "none",
        background: "var(--color-accent)", color: "var(--color-accent-text)",
        fontSize: 13.5, fontWeight: 600, cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1, transition: "background 0.12s, opacity 0.12s",
      }}
      onMouseEnter={(e) => { if (!disabled) (e.currentTarget as HTMLButtonElement).style.background = "var(--color-accent-hover)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = "var(--color-accent)"; }}
    >
      {children}
    </button>
  );
}

function GhostBtn({ children, onClick, disabled, subtle }: { children: React.ReactNode; onClick: () => void; disabled?: boolean; subtle?: boolean }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        height: 40, padding: "0 20px", borderRadius: 99,
        border: subtle ? "none" : "1px solid var(--color-border)",
        background: subtle ? "transparent" : "var(--color-surface)",
        color: subtle ? "var(--color-text-dim)" : "var(--color-text-hi)",
        fontSize: 13.5, fontWeight: 600, cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1, transition: "background 0.12s",
      }}
      onMouseEnter={(e) => { if (!disabled && !subtle) (e.currentTarget as HTMLButtonElement).style.background = "var(--color-surface-2)"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = subtle ? "transparent" : "var(--color-surface)"; }}
    >
      {children}
    </button>
  );
}

// general / visual controls

function SettingRow({ label, hint, control }: { label: string; hint?: string; control: React.ReactNode }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 3 }}>
        <span style={{ fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)" }}>{label}</span>
        {hint && <span style={{ fontSize: 12, lineHeight: 1.5, color: "var(--color-text-dim)" }}>{hint}</span>}
      </div>
      <div style={{ flexShrink: 0 }}>{control}</div>
    </div>
  );
}

function Switch({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      style={{
        width: 44, height: 26, borderRadius: 99, border: "none", cursor: "pointer",
        padding: 3, display: "flex", flexShrink: 0,
        justifyContent: checked ? "flex-end" : "flex-start",
        background: checked ? "var(--color-accent)" : "rgba(255,255,255,0.16)",
        transition: "background 0.18s ease",
      }}
    >
      <span
        style={{
          width: 20, height: 20, borderRadius: "50%", background: "#fff",
          boxShadow: "0 1px 3px rgba(0,0,0,0.4)",
          transition: "transform 0.18s cubic-bezier(0.23,1,0.32,1)",
        }}
      />
    </button>
  );
}

function Segmented<T extends string>({
  value, options, onChange,
}: { value: T; options: { value: T; label: string }[]; onChange: (v: T) => void }) {
  return (
    <div style={{ display: "flex", gap: 2, padding: 2, borderRadius: 99, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
      {options.map((o) => {
        const active = o.value === value;
        return (
          <button
            key={o.value}
            onClick={() => onChange(o.value)}
            style={{
              height: 30, padding: "0 14px", borderRadius: 99, border: "none", cursor: "pointer",
              fontSize: 12.5, fontWeight: 600, fontFamily: "inherit",
              background: active ? "var(--color-accent)" : "transparent",
              color: active ? "var(--color-accent-text)" : "var(--color-text-dim)",
              transition: "background 0.12s, color 0.12s",
            }}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

const VIBRANCY_OPTS: { value: WindowEffect; label: string }[] = [
  { value: "mica",    label: "Mica" },
  { value: "acrylic", label: "Acrylic" },
  { value: "none",    label: "None" },
];

function GeneralCard() {
  const notifyOnTrack    = usePrefsStore((s) => s.notifyOnTrack);
  const setNotifyOnTrack = usePrefsStore((s) => s.setNotifyOnTrack);
  const promptOnClose    = usePrefsStore((s) => s.promptOnClose);
  const setPromptOnClose = usePrefsStore((s) => s.setPromptOnClose);
  const discordPresence    = usePrefsStore((s) => s.discordPresence);
  const setDiscordPresence = usePrefsStore((s) => s.setDiscordPresence);

  function toggleDiscord(v: boolean) {
    setDiscordPresence(v);
    setDiscordEnabled(v); // tell the rust media thread right away
  }

  return (
    <Card title="General">
      <SettingRow
        label="Playback notification"
        hint="Show a desktop notification when a new song starts."
        control={<Switch checked={notifyOnTrack} onChange={setNotifyOnTrack} />}
      />
      <Divider />
      <SettingRow
        label="Confirm before closing"
        hint="Ask before quitting while music is still playing."
        control={<Switch checked={promptOnClose} onChange={setPromptOnClose} />}
      />
      <Divider />
      <SettingRow
        label="Discord Rich Presence"
        hint="Show the song you're playing on your Discord profile."
        control={<Switch checked={discordPresence} onChange={toggleDiscord} />}
      />
    </Card>
  );
}

function VisualCard() {
  const windowEffect    = useUIStore((s) => s.windowEffect);
  const setWindowEffect = useUIStore((s) => s.setWindowEffect);

  function choose(e: WindowEffect) {
    setWindowEffect(e);            // persist + drive the CSS scrim
    applyWindowEffect(e).catch(() => {}); // re-apply / clear native material
  }

  return (
    <Card title="Window">
      <SettingRow
        label="Background material"
        hint={isWindows
          ? "Mica tints with your wallpaper, Acrylic is a darker blur, None is a solid background."
          : "Translucent window materials are a Windows-only feature."}
        control={isWindows
          ? <Segmented value={windowEffect} options={VIBRANCY_OPTS} onChange={choose} />
          : <span style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>Windows only</span>}
      />
    </Card>
  );
}

function LastfmCard() {
  const qc = useQueryClient();
  const { data: status } = useQuery({ queryKey: ["lastfm", "status"], queryFn: lastfmStatus });

  const [apiKey, setApiKey] = useState("");
  const [apiSecret, setApiSecret] = useState("");
  const [phase, setPhase] = useState<"idle" | "saving" | "connecting">("idle");
  const [err, setErr] = useState<string | null>(null);

  const refresh = () => qc.invalidateQueries({ queryKey: ["lastfm", "status"] });

  async function saveAndConnect() {
    setErr(null);
    if (apiKey.trim() && apiSecret.trim()) {
      setPhase("saving");
      try {
        await lastfmSaveApi(apiKey.trim(), apiSecret.trim());
        await refresh();
      } catch (e) {
        setErr(String(e)); setPhase("idle"); return;
      }
    }
    // kick off browser auth, then poll for the session
    setPhase("connecting");
    try {
      const token = await lastfmStartAuth();
      await lastfmFinishAuth(token); // resolves once authorized (or it times out)
      setApiSecret("");
      await refresh();
    } catch (e) {
      setErr(String(e));
    } finally {
      setPhase("idle");
    }
  }

  async function disconnect() {
    await lastfmDisconnect().catch(() => {});
    await refresh();
  }
  async function forget() {
    await lastfmClear().catch(() => {});
    setApiKey(""); setApiSecret("");
    await refresh();
  }

  const busy = phase !== "idle";
  const connected = !!status?.connected;

  return (
    <Card title="Last.fm">
      {connected ? (
        <>
          <SettingRow
            label="Scrobbling enabled"
            hint={status?.username ? `Connected as ${status.username}.` : "Connected."}
            control={
              <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: "#34d399" }} />
                <span style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>Active</span>
              </span>
            }
          />
          <Divider />
          <div style={{ display: "flex", gap: 10 }}>
            <GhostBtn subtle onClick={disconnect}>Disconnect</GhostBtn>
            <div style={{ flex: 1 }} />
            <GhostBtn subtle onClick={forget}>Forget API keys</GhostBtn>
          </div>
        </>
      ) : (
        <>
          <p style={{ margin: 0, fontSize: 13, lineHeight: 1.6, color: "var(--color-text-dim)" }}>
            Scrobble what you play to Last.fm. Create an API account at{" "}
            <a href="https://www.last.fm/api/account/create" target="_blank" rel="noreferrer"
               style={{ color: "var(--color-accent)", textDecoration: "none", fontWeight: 500 }}>
              last.fm/api
            </a>{" "}
            to get a key + secret, paste them below, then connect.
          </p>
          <Field label="API Key">
            <input value={apiKey} onChange={(e) => setApiKey(e.currentTarget.value)}
              placeholder={status?.configured ? "•••••••• (saved)" : "Paste your API key"}
              disabled={busy} autoComplete="off" spellCheck={false} style={inputStyle} />
          </Field>
          <Field label="Shared Secret">
            <input type="password" value={apiSecret} onChange={(e) => setApiSecret(e.currentTarget.value)}
              placeholder={status?.configured ? "•••••••• (saved)" : "Paste your shared secret"}
              disabled={busy} autoComplete="off" spellCheck={false} style={inputStyle} />
          </Field>
          {err && <p style={{ margin: 0, fontSize: 12.5, color: "var(--color-danger)" }}>{err}</p>}
          {phase === "connecting" && (
            <p style={{ margin: 0, fontSize: 12, color: "var(--color-text-dim)" }}>
              Authorize the app in your browser — waiting for confirmation…
            </p>
          )}
          <div style={{ display: "flex", gap: 10, paddingTop: 4 }}>
            <PrimaryBtn onClick={saveAndConnect} disabled={busy || (!status?.configured && !(apiKey.trim() && apiSecret.trim()))}>
              {phase === "saving" ? "Saving…" : phase === "connecting" ? "Waiting…" : "Connect Last.fm"}
            </PrimaryBtn>
          </div>
        </>
      )}
    </Card>
  );
}

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section
      style={{
        borderRadius: 16, background: "var(--color-surface)", border: "1px solid var(--color-border)",
        padding: 28, display: "flex", flexDirection: "column", gap: 18,
      }}
    >
      <h2 style={{ margin: 0, fontSize: 16, fontWeight: 600, color: "var(--color-text-hi)" }}>{title}</h2>
      {children}
    </section>
  );
}

function Divider() {
  return <div style={{ height: 1, background: "var(--color-divider)" }} />;
}

export default function Settings() {
  const qc = useQueryClient();
  const { status, setStatus, setFromCredentials, clear } = useCredentialsStore();

  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [showSecret, setShowSecret] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  function autoValidate() {
    setStatus("validating");
    validateCredentials()
      .then((r) => {
        setStatus(r.valid ? "valid" : "invalid");
        setValidationError(r.valid ? null : (r.error ?? "Invalid credentials"));
      })
      .catch(() => setStatus("invalid"));
  }

  const { isLoading } = useQuery({
    queryKey: ["credentials"],
    queryFn: async () => {
      const creds = await getCredentials();
      setFromCredentials(creds);
      if (creds) {
        setClientId(creds.client_id);
        if (creds.has_secret) autoValidate();
      }
      return creds;
    },
  });

  const { mutate: save, isPending: saving } = useMutation({
    mutationFn: () => saveCredentials(clientId, clientSecret),
    onSuccess: () => {
      setClientSecret("");
      setValidationError(null);
      qc.invalidateQueries({ queryKey: ["credentials"] });
      autoValidate(); // connect now so there's no lingering "saved, not tested"
    },
  });

  const { mutate: disconnect, isPending: disconnecting } = useMutation({
    mutationFn: clearCredentials,
    onSuccess: () => {
      setClientId("");
      setClientSecret("");
      setValidationError(null);
      clear();
      qc.invalidateQueries({ queryKey: ["credentials"] });
    },
  });

  const busy = saving || disconnecting || isLoading;
  const canSave = clientId.trim().length > 0 && clientSecret.trim().length > 0 && !busy;
  const canDisconnect = status !== "unconfigured" && !busy;

  return (
    <div style={{ maxWidth: 620, display: "flex", flexDirection: "column", gap: 24 }}>
      <h1 style={{ margin: 0, fontSize: 28, fontWeight: 700, letterSpacing: "-0.01em", color: "var(--color-text-hi)" }}>
        Settings
      </h1>

      {/* spotify API card */}
      <section
        style={{
          borderRadius: 16,
          background:   "var(--color-surface)",
          border:       "1px solid var(--color-border)",
          padding:      28,
          display:      "flex",
          flexDirection: "column",
          gap:          20,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <h2 style={{ margin: 0, fontSize: 16, fontWeight: 600, color: "var(--color-text-hi)" }}>Spotify API</h2>
          <StatusBadge status={status} />
        </div>

        <p style={{ margin: 0, fontSize: 13, lineHeight: 1.6, color: "var(--color-text-dim)" }}>
          Create an app in the{" "}
          <a
            href="https://developer.spotify.com/dashboard"
            target="_blank"
            rel="noreferrer"
            style={{ color: "var(--color-accent)", textDecoration: "none", fontWeight: 500, display: "inline-flex", alignItems: "center", gap: 3 }}
          >
            Spotify Dashboard <ExternalLink size={11} strokeWidth={2.5} />
          </a>
          , set the redirect URI to{" "}
          <code style={{ fontFamily: "ui-monospace, monospace", fontSize: 12, color: "var(--color-text)", background: "rgba(0,0,0,0.25)", padding: "2px 6px", borderRadius: 5 }}>
            http://127.0.0.1:8888/callback
          </code>
          , then paste the Client ID and Secret below.
        </p>

        <Field label="Client ID">
          <input
            value={clientId}
            onChange={(e) => setClientId(e.currentTarget.value)}
            placeholder={isLoading ? "Loading…" : "Paste your Client ID"}
            disabled={busy}
            autoComplete="off"
            spellCheck={false}
            style={inputStyle}
          />
        </Field>

        <Field label="Client Secret">
          <input
            type={showSecret ? "text" : "password"}
            value={clientSecret}
            onChange={(e) => setClientSecret(e.currentTarget.value)}
            placeholder={
              isLoading ? "Loading…"
              : status !== "unconfigured" ? "•••••••••••••••• (saved)"
              : "Paste your Client Secret"
            }
            disabled={busy}
            autoComplete="off"
            spellCheck={false}
            style={inputStyle}
          />
          <button
            type="button"
            onClick={() => setShowSecret((v) => !v)}
            tabIndex={-1}
            title={showSecret ? "Hide" : "Show"}
            style={{
              width: 42, height: 42, borderRadius: 10, flexShrink: 0,
              border: "1px solid var(--color-border)", background: "var(--color-surface)",
              color: "var(--color-text)", cursor: "pointer",
              display: "flex", alignItems: "center", justifyContent: "center",
            }}
          >
            {showSecret ? <EyeOff size={16} strokeWidth={2} /> : <Eye size={16} strokeWidth={2} />}
          </button>
        </Field>

        {validationError && (
          <p style={{ margin: 0, fontSize: 12.5, color: "var(--color-danger)" }}>{validationError}</p>
        )}

        <div style={{ display: "flex", gap: 10, paddingTop: 4 }}>
          <PrimaryBtn onClick={() => save()} disabled={!canSave}>{saving ? "Saving…" : "Save & Connect"}</PrimaryBtn>
          <div style={{ flex: 1 }} />
          <GhostBtn subtle onClick={() => disconnect()} disabled={!canDisconnect}>{disconnecting ? "Clearing…" : "Disconnect"}</GhostBtn>
        </div>
      </section>

      <GeneralCard />
      <VisualCard />
      <LastfmCard />
    </div>
  );
}
