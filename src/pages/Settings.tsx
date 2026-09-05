import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Eye, EyeOff, RotateCcw, ChevronDown, Info } from "lucide-react";
import { SegmentedControl } from "../components/playground/PlaygroundControls";
import { Tooltip } from "../components/ui/Tooltip";
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
import { type AudioQuality } from "../api/playback";
import { useUIStore } from "../store/ui.store";
import { setDiscordEnabled } from "../api/media";
import {
  lastfmStatus,
  lastfmSaveApi,
  lastfmStartAuth,
  lastfmFinishAuth,
  lastfmDisconnect,
  lastfmClear,
} from "../api/lastfm";
import {
  setWindowEffect as applyWindowEffect,
  type WindowEffect,
} from "../api/window";
import { isWindows, isMac } from "../lib/platform";
import { useThemeStore, type ThemeSource } from "../store/theme.store";
import { useReflowPulse } from "../hooks/useReflowPulse";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };

const STATUS_CONFIG: Record<ConnectionStatus, { dot: string; label: string }> =
  {
    unconfigured: { dot: "#34d399", label: "Shared Quota (Default)" },
    configured: { dot: "#f5a623", label: "Custom Key Saved" },
    validating: { dot: "#fa2d48", label: "Validating…" },
    valid: { dot: "#34d399", label: "Connected" },
    invalid: { dot: "#ff453a", label: "Invalid credentials" },
  };

function StatusBadge({ status }: { status: ConnectionStatus }) {
  const { dot, label } = STATUS_CONFIG[status];
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <span
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          background: dot,
          flexShrink: 0,
        }}
      />
      <span style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>
        {label}
      </span>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  flex: 1,
  minWidth: 0,
  boxSizing: "border-box",
  height: 42,
  borderRadius: 10,
  border: "1px solid var(--color-border)",
  background: "rgba(0,0,0,0.20)",
  color: "var(--color-text-hi)",
  fontSize: 14,
  padding: "0 14px",
  outline: "none",
  fontFamily: "inherit",
};

function Field({
  label,
  children,
  hint,
}: {
  label: string;
  children: React.ReactNode;
  hint?: string;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
      <label
        style={{ fontSize: 12.5, fontWeight: 600, color: "var(--color-text)" }}
      >
        {label}
      </label>
      <div style={{ display: "flex", gap: 8 }}>{children}</div>
      {hint && (
        <span style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>
          {hint}
        </span>
      )}
    </div>
  );
}

function PrimaryBtn({
  children,
  onClick,
  disabled,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        height: 40,
        padding: "0 22px",
        borderRadius: 99,
        border: "none",
        background: "var(--color-accent)",
        color: "var(--color-accent-text)",
        fontSize: 13.5,
        fontWeight: 600,
        cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1,
        transition: "background 0.12s, opacity 0.12s",
      }}
      onMouseEnter={(e) => {
        if (!disabled)
          (e.currentTarget as HTMLButtonElement).style.background =
            "var(--color-accent-hover)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background =
          "var(--color-accent)";
      }}
    >
      {children}
    </button>
  );
}

function GhostBtn({
  children,
  onClick,
  disabled,
  subtle,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  subtle?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        height: 40,
        padding: "0 20px",
        borderRadius: 99,
        border: subtle ? "none" : "1px solid var(--color-border)",
        background: subtle ? "transparent" : "var(--color-surface)",
        color: subtle ? "var(--color-text-dim)" : "var(--color-text-hi)",
        fontSize: 13.5,
        fontWeight: 600,
        cursor: disabled ? "default" : "pointer",
        opacity: disabled ? 0.4 : 1,
        transition: "background 0.12s",
      }}
      onMouseEnter={(e) => {
        if (!disabled && !subtle)
          (e.currentTarget as HTMLButtonElement).style.background =
            "var(--color-surface-2)";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLButtonElement).style.background = subtle
          ? "transparent"
          : "var(--color-surface)";
      }}
    >
      {children}
    </button>
  );
}

// general / visual controls

function SettingRow({
  label,
  hint,
  control,
}: {
  label: React.ReactNode;
  hint?: string;
  control: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        flexWrap: "wrap",
        gap: "10px 16px",
        width: "100%",
      }}
    >
      <div
        style={{
          flex: "1 1 200px",
          minWidth: 0,
          display: "flex",
          flexDirection: "column",
          gap: 3,
        }}
      >
        <span
          style={{
            fontSize: "clamp(13px, 1.2vw, 14px)",
            fontWeight: 600,
            color: "var(--color-text-hi)",
          }}
        >
          {label}
        </span>
        {hint && (
          <span
            style={{
              fontSize: 12,
              lineHeight: 1.5,
              color: "var(--color-text-dim)",
            }}
          >
            {hint}
          </span>
        )}
      </div>
      <div
        style={{
          flexShrink: 0,
          maxWidth: "100%",
          display: "flex",
          alignItems: "center",
        }}
      >
        {control}
      </div>
    </div>
  );
}

function Switch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      style={{
        width: 44,
        height: 24,
        borderRadius: 99,
        border: "none",
        cursor: "pointer",
        padding: 2,
        display: "flex",
        alignItems: "center",
        flexShrink: 0,
        justifyContent: checked ? "flex-end" : "flex-start",
        background: checked ? "var(--color-accent)" : "rgba(255, 255, 255, 0.12)",
        transition: "background 0.2s",
        outline: "none",
      }}
    >
      <motion.div
        layout
        transition={{ type: "spring", stiffness: 500, damping: 30 }}
        style={{
          width: 20,
          height: 20,
          borderRadius: 99,
          background: "#ffffff",
          boxShadow: "0 2px 5px rgba(0,0,0,0.3)",
        }}
      />
    </button>
  );
}

function Segmented<T extends string>({
  value,
  options,
  onChange,
  layoutId = "settings-segmented",
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
  layoutId?: string;
}) {
  const labelToValue = new Map(options.map((o) => [o.label, o.value]));
  const currentOption = options.find((o) => o.value === value);
  const currentLabel = currentOption ? currentOption.label : options[0]?.label || "";

  return (
    <SegmentedControl
      options={options.map((o) => o.label)}
      value={currentLabel}
      onChange={(label) => {
        const targetVal = labelToValue.get(label);
        if (targetVal !== undefined) onChange(targetVal);
      }}
      layoutId={layoutId}
    />
  );
}

const VIBRANCY_OPTS: { value: WindowEffect; label: string }[] = [
  { value: "mica", label: "Mica" },
  { value: "acrylic", label: "Acrylic" },
  { value: "none", label: "None" },
];

// macOS: only Vibrancy / No material. "mica" is reused to mean "vibrancy on".
const MAC_MATERIAL_OPTS: { value: WindowEffect; label: string }[] = [
  { value: "mica", label: "Vibrancy" },
  { value: "none", label: "No material" },
];

// the accent-source choice (System colors only exists on macOS)
const THEME_OPTS: { value: ThemeSource; label: string }[] = [
  { value: "default", label: "Default" },
  { value: "wallpaper", label: "Wallpaper colors" },
  ...(isMac
    ? [{ value: "system" as ThemeSource, label: "System colors" }]
    : []),
];

function AppearanceCard() {
  const source = useThemeStore((s) => s.source);
  const setSource = useThemeStore((s) => s.setSource);
  const albumColors = useThemeStore((s) => s.albumColors);
  const setAlbumColors = useThemeStore((s) => s.setAlbumColors);

  return (
    <Card title="Appearance">
      <SettingRow
        label="Accent color"
        hint={
          isMac
            ? "Default blue, sampled from your desktop wallpaper, or your macOS system accent color."
            : "Default blue, or sampled from your desktop wallpaper."
        }
        control={
          <Segmented
            value={source}
            options={THEME_OPTS}
            onChange={setSource}
            layoutId="settings-accent-color"
          />
        }
      />
      <Divider />
      <SettingRow
        label="Album colors"
        hint="Recolor album & playlist pages from their cover art."
        control={<Switch checked={albumColors} onChange={setAlbumColors} />}
      />
    </Card>
  );
}

const QUALITY_OPTIONS: { value: AudioQuality; label: string }[] = [
  { value: "96", label: "Normal · 96 kbps" },
  { value: "160", label: "High · 160 kbps" },
  { value: "320", label: "Very high · 320 kbps" },
];

function PlaybackCard() {
  const audioQuality = usePrefsStore((s) => s.audioQuality);
  const setAudioQuality = usePrefsStore((s) => s.setAudioQuality);

  return (
    <Card title="Playback">
      <SettingRow
        label="Audio quality"
        hint="Higher bitrates use more data and cache space."
        control={
          <Segmented
            value={audioQuality}
            options={QUALITY_OPTIONS}
            onChange={setAudioQuality}
            layoutId="settings-audio-quality"
          />
        }
      />
    </Card>
  );
}

function GeneralCard() {
  const notifyOnTrack = usePrefsStore((s) => s.notifyOnTrack);
  const setNotifyOnTrack = usePrefsStore((s) => s.setNotifyOnTrack);
  const promptOnClose = usePrefsStore((s) => s.promptOnClose);
  const setPromptOnClose = usePrefsStore((s) => s.setPromptOnClose);
  const discordPresence = usePrefsStore((s) => s.discordPresence);
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
  const windowEffect = useUIStore((s) => s.windowEffect);
  const setWindowEffect = useUIStore((s) => s.setWindowEffect);
  const transparency = useUIStore((s) => s.materialTransparency);
  const setTransparency = useUIStore((s) => s.setMaterialTransparency);
  const fastMode = useUIStore((s) => s.fastMode);
  const setFastMode = useUIStore((s) => s.setFastMode);

  function toggleFastMode(enabled: boolean) {
    setFastMode(enabled);
    applyWindowEffect(enabled ? "none" : useUIStore.getState().windowEffect).catch(() => {});
  }

  function choose(e: WindowEffect) {
    if (fastMode) return;
    setWindowEffect(e); // persist + drive the CSS scrim
    applyWindowEffect(e).catch(() => {}); // re-apply / clear native material
  }

  // on mac "acrylic" isn't a thing; coerce so the segmented has a valid selection
  const macValue: WindowEffect = windowEffect === "none" ? "none" : "mica";

  // the transparency slider only applies to adjustable materials:
  // Windows acrylic, or macOS vibrancy (which we model as "mica" on mac).
  const showTransparency =
    !fastMode &&
    ((isWindows && windowEffect === "acrylic") || (isMac && macValue === "mica"));

  // guard: an old/partial persisted store can hand us a non-finite value, which
  // makes `--vol` compute to "NaN%" -> invalid -> track falls back to 0% (all grey).
  const tval = Number.isFinite(transparency) ? transparency : 0.4;
  const tpct = Math.round(tval * 100); // 10..70, shown as label
  const tfill = Math.max(
    0,
    Math.min(100, Math.round(((tpct - 10) / 60) * 100)),
  ); // 0..100 fill

  return (
    <Card title="Performance & Window">
      <SettingRow
        label={
          <div style={{ display: "flex", alignItems: "center", gap: 7 }}>
            <span>Fast mode</span>
            <Tooltip
              label="Maximizes performance: disables window Mica/Acrylic materials, eliminates icon and gradient blurs, turns off animated cover backdrops in Immersive and Lyrics, and disables album page tint blooms."
              side="top"
            >
              <span
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  color: "var(--color-text-dim)",
                  cursor: "help",
                }}
              >
                <Info size={14} strokeWidth={2.2} />
              </span>
            </Tooltip>
          </div>
        }
        hint="Sacrifices transparency, blurs, and transitions for maximum frame rates and lowest CPU/GPU usage."
        control={<Switch checked={fastMode} onChange={toggleFastMode} />}
      />
      <Divider />
      <SettingRow
        label="Background material"
        hint={
          fastMode
            ? "Disabled in Fast mode. Turn off Fast mode to enable translucent window materials."
            : isWindows
              ? "Mica tints with your wallpaper, Acrylic is a darker blur, None is a solid background."
              : isMac
                ? "Vibrancy shows the desktop through the window; No material paints a solid background."
                : "Translucent window materials aren't available on this platform."
        }
        control={
          fastMode ? (
            <span style={{ fontSize: 12.5, color: "var(--color-text-dim)", fontStyle: "italic" }}>
              Locked to None (Fast mode)
            </span>
          ) : isWindows ? (
            <Segmented
              value={windowEffect}
              options={VIBRANCY_OPTS}
              onChange={choose}
              layoutId="settings-bg-material"
            />
          ) : isMac ? (
            <Segmented
              value={macValue}
              options={MAC_MATERIAL_OPTS}
              onChange={choose}
              layoutId="settings-bg-material"
            />
          ) : (
            <span style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>
              Not available
            </span>
          )
        }
      />
      {showTransparency && (
        <>
          <Divider />
          <SettingRow
            label="Transparency"
            hint="How much the desktop shows through the material."
            control={
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  width: "100%",
                  minWidth: "min(196px, 100%)",
                  maxWidth: 260,
                  boxSizing: "border-box",
                }}
              >
                <input
                  className="vol"
                  type="range"
                  // NOT 0-100: the range is capped so the window can never go fully
                  // transparent (invisible) or pointlessly solid. 10% .. 70%.
                  min={10}
                  max={70}
                  step={1}
                  value={tpct}
                  onChange={(e) =>
                    setTransparency(Number(e.target.value) / 100)
                  }
                  aria-label="Material transparency"
                  // --vol drives the same fluid fill the volume slider uses; map the
                  // capped 10..70 value onto a 0..100% track fill.
                  style={
                    {
                      flex: 1,
                      ["--vol" as string]: `${tfill}%`,
                    } as React.CSSProperties
                  }
                />
                <span
                  style={{
                    fontSize: 12,
                    fontVariantNumeric: "tabular-nums",
                    color: "var(--color-text-dim)",
                    width: 34,
                    textAlign: "right",
                  }}
                >
                  {tpct}%
                </span>
                <Tooltip label="Revert to default" side="top">
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={() => setTransparency(0.4)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      width: 24,
                      height: 24,
                      borderRadius: 6,
                      border: "none",
                      background: "transparent",
                      color:
                        Math.abs(tval - 0.4) < 0.005
                          ? "rgba(255,255,255,0.22)"
                          : "var(--color-text-dim)",
                      cursor:
                        Math.abs(tval - 0.4) < 0.005 ? "default" : "pointer",
                      padding: 0,
                      flexShrink: 0,
                      transition: "color 0.12s",
                    }}
                    onMouseEnter={(e) => {
                      if (Math.abs(tval - 0.4) >= 0.005)
                        (e.currentTarget as HTMLButtonElement).style.color =
                          "var(--color-text-hi)";
                    }}
                    onMouseLeave={(e) => {
                      if (Math.abs(tval - 0.4) >= 0.005)
                        (e.currentTarget as HTMLButtonElement).style.color =
                          "var(--color-text-dim)";
                    }}
                  >
                    <RotateCcw size={13} strokeWidth={2.2} />
                  </motion.button>
                </Tooltip>
              </div>
            }
          />
        </>
      )}
    </Card>
  );
}

function LastfmCard() {
  const qc = useQueryClient();
  const { data: status } = useQuery({
    queryKey: ["lastfm", "status"],
    queryFn: lastfmStatus,
  });

  const [apiKey, setApiKey] = useState("");
  const [apiSecret, setApiSecret] = useState("");
  const [phase, setPhase] = useState<"idle" | "saving" | "connecting">("idle");
  const [err, setErr] = useState<string | null>(null);

  const refresh = () =>
    qc.invalidateQueries({ queryKey: ["lastfm", "status"] });

  async function saveAndConnect() {
    setErr(null);
    if (apiKey.trim() && apiSecret.trim()) {
      setPhase("saving");
      try {
        await lastfmSaveApi(apiKey.trim(), apiSecret.trim());
        await refresh();
      } catch (e) {
        setErr(String(e));
        setPhase("idle");
        return;
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
    setApiKey("");
    setApiSecret("");
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
            hint={
              status?.username
                ? `Connected as ${status.username}.`
                : "Connected."
            }
            control={
              <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span
                  style={{
                    width: 8,
                    height: 8,
                    borderRadius: "50%",
                    background: "#34d399",
                  }}
                />
                <span
                  style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}
                >
                  Active
                </span>
              </span>
            }
          />
          <Divider />
          <div style={{ display: "flex", flexWrap: "wrap", gap: 10, alignItems: "center" }}>
            <GhostBtn subtle onClick={disconnect}>
              Disconnect
            </GhostBtn>
            <div style={{ flex: 1 }} />
            <GhostBtn subtle onClick={forget}>
              Forget API keys
            </GhostBtn>
          </div>
        </>
      ) : (
        <>
          <p
            style={{
              margin: 0,
              fontSize: 13,
              lineHeight: 1.6,
              color: "var(--color-text-dim)",
            }}
          >
            Scrobble what you play to Last.fm. Create an API account at{" "}
            <a
              href="https://www.last.fm/api/account/create"
              target="_blank"
              rel="noreferrer"
              style={{
                color: "var(--color-accent)",
                textDecoration: "none",
                fontWeight: 500,
              }}
            >
              last.fm/api
            </a>{" "}
            to get a key + secret, paste them below, then connect.
          </p>
          <Field label="API Key">
            <input
              value={apiKey}
              onChange={(e) => setApiKey(e.currentTarget.value)}
              placeholder={
                status?.configured ? "•••••••• (saved)" : "Paste your API key"
              }
              disabled={busy}
              autoComplete="off"
              spellCheck={false}
              style={inputStyle}
            />
          </Field>
          <Field label="Shared Secret">
            <input
              type="password"
              value={apiSecret}
              onChange={(e) => setApiSecret(e.currentTarget.value)}
              placeholder={
                status?.configured
                  ? "•••••••• (saved)"
                  : "Paste your shared secret"
              }
              disabled={busy}
              autoComplete="off"
              spellCheck={false}
              style={inputStyle}
            />
          </Field>
          {err && (
            <p
              style={{
                margin: 0,
                fontSize: 12.5,
                color: "var(--color-danger)",
              }}
            >
              {err}
            </p>
          )}
          {phase === "connecting" && (
            <p
              style={{
                margin: 0,
                fontSize: 12,
                color: "var(--color-text-dim)",
              }}
            >
              Authorize the app in your browser - waiting for confirmation…
            </p>
          )}
          <div style={{ display: "flex", gap: 10, paddingTop: 4 }}>
            <PrimaryBtn
              onClick={saveAndConnect}
              disabled={
                busy ||
                (!status?.configured && !(apiKey.trim() && apiSecret.trim()))
              }
            >
              {phase === "saving"
                ? "Saving…"
                : phase === "connecting"
                  ? "Waiting…"
                  : "Connect Last.fm"}
            </PrimaryBtn>
          </div>
        </>
      )}
    </Card>
  );
}

function Card({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <motion.section
      layout="position"
      transition={{ layout: REFLOW }}
      style={{
        borderRadius: 16,
        background: "var(--color-surface)",
        border: "1px solid var(--color-border)",
        padding: "clamp(16px, 2.5vw, 24px)",
        display: "flex",
        flexDirection: "column",
        gap: 18,
        width: "100%",
        boxSizing: "border-box",
      }}
    >
      <h2
        style={{
          margin: 0,
          fontSize: "clamp(15px, 1.6vw, 17px)",
          fontWeight: 600,
          letterSpacing: "-0.01em",
          color: "var(--color-text-hi)",
        }}
      >
        {title}
      </h2>
      {children}
    </motion.section>
  );
}

function Divider() {
  return <div style={{ height: 1, background: "var(--color-divider)" }} />;
}

export default function Settings() {
  useReflowPulse();
  const qc = useQueryClient();
  const { status, setStatus, setFromCredentials, clear, isCustom } =
    useCredentialsStore();

  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [showSecret, setShowSecret] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState(true);

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
      if (creds && creds.is_custom) {
        setClientId(creds.client_id);
        if (creds.has_secret) autoValidate();
      } else {
        setClientId("");
      }
      return creds;
    },
  });

  const { mutate: save, isPending: saving } = useMutation({
    mutationFn: () => saveCredentials(clientId, clientSecret || undefined),
    onSuccess: () => {
      setClientSecret("");
      setValidationError(null);
      qc.invalidateQueries({ queryKey: ["credentials"] });
      autoValidate();
    },
  });

  const { mutate: resetToDefault, isPending: resetting } = useMutation({
    mutationFn: clearCredentials,
    onSuccess: () => {
      setClientId("");
      setClientSecret("");
      setValidationError(null);
      clear();
      qc.invalidateQueries({ queryKey: ["credentials"] });
    },
  });

  const busy = saving || resetting || isLoading;
  const canSave = clientId.trim().length > 0 && !busy;
  const canReset = isCustom && !busy;

  return (
    <div
      style={{
        maxWidth: "min(760px, 100%)",
        margin: "0 auto",
        width: "100%",
        boxSizing: "border-box",
        display: "flex",
        flexDirection: "column",
        gap: "clamp(20px, 2.5vw, 26px)",
        paddingBottom: 60,
      }}
    >
      <h1
        style={{
          margin: 0,
          fontSize: "clamp(24px, 3vw, 30px)",
          fontWeight: 700,
          letterSpacing: "-0.02em",
          color: "var(--color-text-hi)",
        }}
      >
        Settings
      </h1>

      <PlaybackCard />
      <GeneralCard />
      <AppearanceCard />
      <VisualCard />
      <LastfmCard />

      {/* spotify API & authentication card (collapsible at the very end) */}
      <motion.section
        layout="position"
        transition={{ layout: REFLOW }}
        style={{
          borderRadius: 16,
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          padding: "clamp(16px, 2.5vw, 24px)",
          display: "flex",
          flexDirection: "column",
          gap: 0,
          width: "100%",
          boxSizing: "border-box",
        }}
      >
        <div
          onClick={() => setCollapsed((v) => !v)}
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: 12,
            cursor: "pointer",
            userSelect: "none",
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", gap: 4, flex: "1 1 240px", minWidth: 0 }}>
            <h2
              style={{
                margin: 0,
                fontSize: "clamp(15px, 1.6vw, 17px)",
                fontWeight: 600,
                letterSpacing: "-0.01em",
                color: "var(--color-text-hi)",
              }}
            >
              Spotify API & Authentication
            </h2>
            <span style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
              {isCustom
                ? "Using your personal developer application (Custom Quota)"
                : "Using public shared multi-grant credentials (Default Quota)"}
            </span>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
            <StatusBadge status={status} />
            <motion.div
              animate={{ rotate: collapsed ? 0 : 180 }}
              transition={{ duration: 0.2, ease: "easeOut" }}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: 28,
                height: 28,
                borderRadius: 8,
                background: "rgba(255,255,255,0.05)",
                color: "var(--color-text-dim)",
              }}
            >
              <ChevronDown size={16} strokeWidth={2.2} />
            </motion.div>
          </div>
        </div>

        <AnimatePresence initial={false}>
          {!collapsed && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.26, ease: [0.23, 1, 0.32, 1] }}
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 20,
                overflow: "hidden",
                paddingTop: 20,
              }}
            >
              <p
                style={{
                  margin: 0,
                  fontSize: 13,
                  lineHeight: 1.6,
                  color: "var(--color-text-dim)",
                }}
              >
                Musique includes built-in authentication with zero configuration needed.
                If you are a power user experiencing rate limits, you can optionally connect your own
                Spotify Developer App. Set the redirect URI in your dashboard to{" "}
                <code
                  style={{
                    fontFamily: "ui-monospace, monospace",
                    fontSize: 12,
                    color: "var(--color-text)",
                    background: "rgba(0,0,0,0.25)",
                    padding: "2px 6px",
                    borderRadius: 5,
                    wordBreak: "break-all",
                  }}
                >
                  http://127.0.0.1:8989/login
                </code>
                , then paste your Client ID below. (Client Secret is optional with PKCE).
              </p>

              <Field label="Personal Client ID (Optional)">
                <input
                  value={clientId}
                  onChange={(e) => setClientId(e.currentTarget.value)}
                  placeholder={
                    isLoading
                      ? "Loading…"
                      : isCustom
                        ? clientId
                        : "Leave blank to use default shared access"
                  }
                  disabled={busy}
                  autoComplete="off"
                  spellCheck={false}
                  style={inputStyle}
                />
              </Field>

              <Field label="Client Secret (Optional)">
                <input
                  type={showSecret ? "text" : "password"}
                  value={clientSecret}
                  onChange={(e) => setClientSecret(e.currentTarget.value)}
                  placeholder={
                    isLoading
                      ? "Loading…"
                      : isCustom && status === "configured"
                        ? "•••••••••••••••• (saved)"
                        : "Optional (not required for PKCE)"
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
                    width: 42,
                    height: 42,
                    borderRadius: 10,
                    flexShrink: 0,
                    border: "1px solid var(--color-border)",
                    background: "var(--color-surface)",
                    color: "var(--color-text)",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  {showSecret ? (
                    <EyeOff size={16} strokeWidth={2} />
                  ) : (
                    <Eye size={16} strokeWidth={2} />
                  )}
                </button>
              </Field>

              {validationError && (
                <p
                  style={{ margin: 0, fontSize: 12.5, color: "var(--color-danger)" }}
                >
                  {validationError}
                </p>
              )}

              <div style={{ display: "flex", flexWrap: "wrap", gap: 10, paddingTop: 4 }}>
                <PrimaryBtn onClick={() => save()} disabled={!canSave}>
                  {saving ? "Saving…" : "Save Custom Client ID"}
                </PrimaryBtn>
                <div style={{ flex: 1 }} />
                <GhostBtn
                  subtle
                  onClick={() => resetToDefault()}
                  disabled={!canReset}
                >
                  {resetting ? "Resetting…" : "Reset to Default Quota"}
                </GhostBtn>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.section>
    </div>
  );
}
