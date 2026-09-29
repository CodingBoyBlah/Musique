import { useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Eye, EyeOff, RotateCcw, ChevronDown, Info, Minus, Plus } from "@/lib/icons";
import { stepZoom, resetZoom, zoomLabel, ZOOM_MIN, ZOOM_MAX } from "../lib/zoom";
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
import { usePrefsStore, type SidebarMode } from "../store/prefs.store";
import { usePlayerStore } from "../store/player.store";
import {
  type AudioQuality,
  type PlaybackBackend,
  setPlaybackBackend,
} from "../api/playback";
import { usePlaybackBackend } from "../hooks/usePlaybackBackend";
import { toast } from "../store/toast.store";
import { errMsg } from "../lib/err";
import { useUIStore } from "../store/ui.store";
import { setDiscordEnabled, requestNotificationPermission } from "../api/media";
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
import { EASE_OUT, PRESS_TRANSITION, REFLOW_SPRING } from "../lib/motion";
import "../styles/ui.css";
import { Switch } from "../components/ui/Switch";

const REFLOW = REFLOW_SPRING;

const STATUS_CONFIG: Record<ConnectionStatus, { dot: string; label: string }> =
  {
    unconfigured: { dot: "#34d399", label: "Built-in access" },
    configured: { dot: "#f5a623", label: "Custom app saved" },
    // in progress, not a failure: amber, pulsing, never the error red
    validating: { dot: "var(--color-warning)", label: "Checking…" },
    valid: { dot: "#34d399", label: "Connected" },
    invalid: { dot: "#ff453a", label: "Invalid credentials" },
  };

function StatusBadge({ status }: { status: ConnectionStatus }) {
  const { dot, label } = STATUS_CONFIG[status];
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <span
        className={status === "validating" ? "status-pulse" : undefined}
        style={{
          width: 8,
          height: 8,
          borderRadius: "50%",
          background: dot,
          flexShrink: 0,
        }}
      />
      <span className="t-caption" role="status" style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}>
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
        <span className="t-caption" style={{ fontSize: 11.5, color: "var(--color-text-dim)" }}>
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
      className="btn-primary"
      style={{ height: 40, padding: "0 22px", fontSize: 13.5 }}
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
  danger,
  autoFocus,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  subtle?: boolean;
  danger?: boolean;
  autoFocus?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      autoFocus={autoFocus}
      className={danger ? "btn-danger" : subtle ? "btn-text ghost-subtle" : "btn-pill"}
      style={{
        height: 40,
        padding: "0 20px",
        borderRadius: 99,
        fontSize: 13.5,
        fontWeight: 600,
      }}
    >
      {children}
    </button>
  );
}

/* The revert-to-default button next to a slider. At the default there is
   nothing to revert, so it goes quiet and inert - no hover, no press - rather
   than pretending to act. */
function RevertBtn({
  atDefault,
  onClick,
}: {
  atDefault: boolean;
  onClick: () => void;
}) {
  return (
    <motion.button
      type="button"
      whileHover={atDefault ? undefined : { scale: 1.03 }}
      whileTap={atDefault ? undefined : { scale: 0.97 }}
      transition={PRESS_TRANSITION}
      onClick={() => { if (!atDefault) onClick(); }}
      aria-disabled={atDefault || undefined}
      aria-label="Revert to default"
      className="btn-icon"
      style={{
        width: 24,
        height: 24,
        borderRadius: 6,
        padding: 0,
        color: atDefault ? "rgba(255,255,255,0.22)" : undefined,
        opacity: 1,
      }}
    >
      <RotateCcw size={13} strokeWidth={2.2} />
    </motion.button>
  );
}

// general / visual controls

function SettingRow({
  label,
  hint,
  control,
}: {
  label: string;
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
            fontSize: "1rem",
            fontWeight: 600,
            color: "var(--color-text-hi)",
          }}
        >
          {label}
        </span>
        {hint && (
          <span
            className="t-caption"
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

function Segmented<T extends string>({
  value,
  options,
  onChange,
  layoutId = "settings-segmented",
}: {
  value: T;
  options: {
    value: T;
    label: string;
    /** Renders greyed out and unselectable. */
    disabled?: boolean;
    /** Extra content after the label, e.g. an info icon saying why it's off. */
    info?: React.ReactNode;
  }[];
  onChange: (v: T) => void;
  layoutId?: string;
}) {
  const labelToValue = new Map(options.map((o) => [o.label, o.value]));
  const currentOption = options.find((o) => o.value === value);
  const currentLabel = currentOption ? currentOption.label : options[0]?.label || "";

  const disabled = options.filter((o) => o.disabled).map((o) => o.label);
  const adornments = Object.fromEntries(
    options.filter((o) => o.info).map((o) => [o.label, o.info]),
  );

  return (
    <SegmentedControl
      options={options.map((o) => o.label)}
      value={currentLabel}
      onChange={(label) => {
        const targetVal = labelToValue.get(label);
        if (targetVal !== undefined) onChange(targetVal);
      }}
      layoutId={layoutId}
      disabled={disabled.length ? disabled : undefined}
      adornments={Object.keys(adornments).length ? adornments : undefined}
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

const MOD_KEY = isMac ? "⌘" : "Ctrl";

/* − 100% + , the same steps as the keyboard. The percentage is the reset:
   clicking it goes back to 100%, the way a browser's zoom badge does. */
function ZoomStepper() {
  const zoom = usePrefsStore((s) => s.uiZoom);
  const btn: React.CSSProperties = {
    width: 32,
    height: 32,
    borderRadius: 8,
    border: "1px solid var(--color-border)",
    background: "var(--color-surface)",
  };
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <Tooltip label={`Zoom out (${MOD_KEY} −)`}>
        <button
          type="button"
          className="btn-icon"
          aria-label="Zoom out"
          aria-disabled={zoom <= ZOOM_MIN || undefined}
          onClick={() => zoom > ZOOM_MIN && stepZoom(-1, { quiet: true })}
          style={btn}
        >
          <Minus size={15} strokeWidth={2.2} />
        </button>
      </Tooltip>
      <Tooltip label={zoom === 1 ? "Actual size" : `Reset to 100% (${MOD_KEY} 0)`}>
        <button
          type="button"
          className="btn-text tnum"
          aria-label={`Zoom ${zoomLabel(zoom)}. Reset to 100%`}
          onClick={() => resetZoom({ quiet: true })}
          style={{ minWidth: 56, height: 32, borderRadius: 8, fontSize: 13, fontWeight: 700, color: "var(--color-text-hi)" }}
        >
          {zoomLabel(zoom)}
        </button>
      </Tooltip>
      <Tooltip label={`Zoom in (${MOD_KEY} +)`}>
        <button
          type="button"
          className="btn-icon"
          aria-label="Zoom in"
          aria-disabled={zoom >= ZOOM_MAX || undefined}
          onClick={() => zoom < ZOOM_MAX && stepZoom(1, { quiet: true })}
          style={btn}
        >
          <Plus size={15} strokeWidth={2.2} />
        </button>
      </Tooltip>
    </div>
  );
}

const SIDEBAR_MODE_OPTS: { value: SidebarMode; label: string }[] = [
  { value: "pins", label: "Pins" },
  { value: "playlists", label: "Playlists" },
];

function AppearanceCard() {
  const source = useThemeStore((s) => s.source);
  const setSource = useThemeStore((s) => s.setSource);
  const albumColors = useThemeStore((s) => s.albumColors);
  const setAlbumColors = useThemeStore((s) => s.setAlbumColors);
  const ambientMotion = usePrefsStore((s) => s.ambientMotion);
  const sidebarMode = usePrefsStore((s) => s.sidebarMode);
  const setSidebarMode = usePrefsStore((s) => s.setSidebarMode);
  const setAmbientMotion = usePrefsStore((s) => s.setAmbientMotion);
  const showCanvas = usePrefsStore((s) => s.showCanvas);
  const setShowCanvas = usePrefsStore((s) => s.setShowCanvas);

  return (
    <Card title="Appearance">
      <SettingRow
        label="Zoom"
        hint={`Make everything bigger or smaller, for large or high-resolution screens. ${MOD_KEY} + and ${MOD_KEY} − work anywhere, ${MOD_KEY} 0 resets.`}
        control={<ZoomStepper />}
      />
      <Divider />
      <SettingRow
        label="Sidebar"
        hint="Pins shows only the playlists and albums you pin. Playlists lists every playlist in your library."
        control={
          <Segmented
            value={sidebarMode}
            options={SIDEBAR_MODE_OPTS}
            onChange={setSidebarMode}
            layoutId="settings-sidebar-mode"
          />
        }
      />
      <Divider />
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
      <Divider />
      <SettingRow
        label="Animated background"
        hint="Let the blurred cover art drift slowly behind the full-screen player. Turn off for a still background."
        control={<Switch checked={ambientMotion} onChange={setAmbientMotion} />}
      />
      <Divider />
      <SettingRow
        label="Animated artwork"
        hint="Play the looping video (Spotify Canvas) some artists attach to their tracks in the full-screen player. The background takes its colours from the video."
        control={<Switch checked={showCanvas} onChange={setShowCanvas} />}
      />
    </Card>
  );
}

const QUALITY_OPTIONS: { value: AudioQuality; label: string }[] = [
  { value: "96", label: "Normal · 96 kbps" },
  { value: "160", label: "High · 160 kbps" },
  { value: "320", label: "Very high · 320 kbps" },
];

/* The "Requires Premium" marker on the Spotify option. Rendered even when that
   option is selectable, so the constraint is discoverable before someone loses
   Premium rather than only after. */
const PREMIUM_INFO = (
  <Tooltip label="Requires Premium">
    <span
      style={{ display: "inline-flex", alignItems: "center", cursor: "help" }}
    >
      <Info size={12} strokeWidth={2.2} />
    </span>
  </Tooltip>
);

/* Podcasts on YouTube Music play from the show's own public feed (or its
   YouTube uploads), so the few that exist only on Spotify can't. */
const PODCAST_INFO = (
  <Tooltip label="Podcasts play from each show's public feed. Spotify exclusives won't play">
    <span
      style={{ display: "inline-flex", alignItems: "center", cursor: "help" }}
    >
      <Info size={12} strokeWidth={2.2} />
    </span>
  </Tooltip>
);

function backendOptions(spotifyAvailable: boolean): {
  value: PlaybackBackend;
  label: string;
  disabled?: boolean;
  info?: React.ReactNode;
}[] {
  return [
    {
      value: "spotify",
      label: "Spotify",
      // Spotify only streams audio to Premium, so a free account can't pick it.
      disabled: !spotifyAvailable,
      info: PREMIUM_INFO,
    },
    { value: "youtube", label: "YouTube Music", info: PODCAST_INFO },
  ];
}

function PlaybackCard() {
  const audioQuality = usePrefsStore((s) => s.audioQuality);
  const setAudioQuality = usePrefsStore((s) => s.setAudioQuality);
  const audioCacheLimitMb = usePrefsStore((s) => s.audioCacheLimitMb);
  const setAudioCacheLimitMb = usePrefsStore((s) => s.setAudioCacheLimitMb);

  // Free Spotify accounts can't stream through librespot, so "Automatic"
  // routes them to YouTube Music. The explicit options exist mainly so the
  // YouTube path can be tried on a Premium account without downgrading it.
  const qcBackend = useQueryClient();
  const { data: backend } = usePlaybackBackend();
  /* Optimistic: the pill moves the moment it's chosen, not after the IPC
     round trip and a refetch - otherwise a drag-release snaps back to the old
     value and then jumps forward. Rolled back if the switch fails. */
  const [pendingBackend, setPendingBackend] = useState<PlaybackBackend | null>(null);
  // only the latest pick may settle the pill: an older request finishing
  // after a newer pick would otherwise snap it back to a stale value
  const backendReq = useRef(0);
  const changeBackend = (mode: PlaybackBackend) => {
    const seq = ++backendReq.current;
    setPendingBackend(mode);
    setPlaybackBackend(mode)
      .then(() => qcBackend.invalidateQueries({ queryKey: ["playback-backend"] }))
      .catch((e) => {
        if (seq === backendReq.current) toast.error(errMsg(e));
      })
      .finally(() => {
        if (seq === backendReq.current) setPendingBackend(null);
      });
  };

  const cfill = Math.round(
    ((Math.max(512, Math.min(8192, audioCacheLimitMb)) - 512) / (8192 - 512)) * 100
  );
  const gbLabel = (audioCacheLimitMb / 1024).toFixed(1) + " GB";

  return (
    <Card title="Playback">
      <SettingRow
        label="Audio source"
        hint={
          backend && !backend.spotify_available
            ? "Your Spotify plan can't stream audio, so playback uses YouTube Music. Metadata, artwork and lyrics still come from Spotify."
            : "Where audio is streamed from. Metadata, artwork and lyrics always come from Spotify."
        }
        control={
          <Segmented
            value={pendingBackend ?? backend?.active ?? "youtube"}
            options={backendOptions(backend?.spotify_available ?? false)}
            onChange={changeBackend}
            layoutId="settings-playback-backend"
          />
        }
      />
      <SettingRow
        label="Audio quality"
        hint="Higher bitrates use more data. Applied on next track."
        control={
          <Segmented
            value={audioQuality}
            options={QUALITY_OPTIONS}
            onChange={setAudioQuality}
            layoutId="settings-audio-quality"
          />
        }
      />
      <SettingRow
        label="Disk cache limit"
        hint="Storage reserved for tracks & keys for instant repeat playback."
        control={
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              width: "100%",
              minWidth: "min(196px, 100%)",
              maxWidth: 290,
              boxSizing: "border-box",
            }}
          >
            <input
              className="vol"
              type="range"
              min={512}
              max={8192}
              step={256}
              value={audioCacheLimitMb}
              onChange={(e) => setAudioCacheLimitMb(Number(e.target.value))}
              aria-label="Disk cache limit"
              style={
                {
                  flex: 1,
                  ["--vol" as string]: `${cfill}%`,
                } as React.CSSProperties
              }
            />
            <span
              className="tnum"
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: "var(--color-text-hi)",
                minWidth: 46,
                textAlign: "right",
              }}
            >
              {gbLabel}
            </span>
            <Tooltip label={audioCacheLimitMb === 2048 ? "Already at the default (2 GB)" : "Revert to default (2 GB)"} side="top">
              <RevertBtn atDefault={audioCacheLimitMb === 2048} onClick={() => setAudioCacheLimitMb(2048)} />
            </Tooltip>
          </div>
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
        control={
          <Switch
            checked={notifyOnTrack}
            onChange={(v) => {
              setNotifyOnTrack(v);
              if (v) requestNotificationPermission();
            }}
          />
        }
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

function FeaturesCard() {
  const showStats = usePrefsStore((s) => s.showStats);
  const setShowStats = usePrefsStore((s) => s.setShowStats);
  const showFriends = usePrefsStore((s) => s.showFriends);
  const setShowFriends = usePrefsStore((s) => s.setShowFriends);
  return (
    <Card title="Features">
      <SettingRow
        label="Friend activity"
        hint="The Friends panel in the top bar: what people you follow are playing, and Jams."
        control={
          <Switch
            checked={showFriends}
            onChange={(v) => {
              setShowFriends(v);
              if (!v && usePlayerStore.getState().friendsOpen) usePlayerStore.getState().toggleFriends();
            }}
          />
        }
      />
      <Divider />
      <SettingRow
        label="Listening stats"
        hint="The Stats page in the sidebar: time listened, streaks, top songs and when you listen. Worked out on this device."
        control={<Switch checked={showStats} onChange={setShowStats} />}
      />
    </Card>
  );
}

function VisualCard() {
  const windowEffect = useUIStore((s) => s.windowEffect);
  const setWindowEffect = useUIStore((s) => s.setWindowEffect);
  const transparency = useUIStore((s) => s.materialTransparency);
  const setTransparency = useUIStore((s) => s.setMaterialTransparency);

  function choose(e: WindowEffect) {
    setWindowEffect(e); // persist + drive the CSS scrim
    applyWindowEffect(e).catch(() => {}); // re-apply / clear native material
  }

  // on mac "acrylic" isn't a thing; coerce so the segmented has a valid selection
  const macValue: WindowEffect = windowEffect === "none" ? "none" : "mica";

  // the transparency slider only applies to adjustable materials:
  // Windows acrylic, or macOS vibrancy (which we model as "mica" on mac).
  const showTransparency =
    (isWindows && windowEffect === "acrylic") || (isMac && macValue === "mica");

  // guard: an old/partial persisted store can hand us a non-finite value, which
  // makes `--vol` compute to "NaN%" -> invalid -> track falls back to 0% (all grey).
  const tval = Number.isFinite(transparency) ? transparency : 0.4;
  const tpct = Math.round(tval * 100); // 10..70, shown as label
  const tfill = Math.max(
    0,
    Math.min(100, Math.round(((tpct - 10) / 60) * 100)),
  ); // 0..100 fill

  return (
    <Card title="Window">
      <SettingRow
        label="Background material"
        hint={
          isWindows
            ? "Mica tints with your wallpaper, Acrylic is a darker blur, None is a solid background."
            : isMac
              ? "Vibrancy shows the desktop through the window; No material paints a solid background."
              : "Translucent window materials aren't available on this platform."
        }
        control={
          isWindows ? (
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
                  className="tnum"
                  style={{
                    fontSize: 12,
                    color: "var(--color-text-dim)",
                    width: 34,
                    textAlign: "right",
                  }}
                >
                  {tpct}%
                </span>
                <Tooltip label={Math.abs(tval - 0.4) < 0.005 ? "Already at the default" : "Revert to default"} side="top">
                  <RevertBtn atDefault={Math.abs(tval - 0.4) < 0.005} onClick={() => setTransparency(0.4)} />
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
  // forgetting the keys can't be undone (they have to be pasted again), so it
  // asks once, inline, right where the button was
  const [confirmForget, setConfirmForget] = useState(false);

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
        setErr(errMsg(e));
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
      setErr(errMsg(e));
    } finally {
      setPhase("idle");
    }
  }

  async function disconnect() {
    await lastfmDisconnect().catch(() => {});
    await refresh();
  }
  async function forget() {
    setConfirmForget(false);
    await lastfmClear().catch((e) => toast.error(errMsg(e)));
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
                  className="t-caption"
                  style={{ fontSize: 12.5, color: "var(--color-text-dim)" }}
                >
                  Active
                </span>
              </span>
            }
          />
          <Divider />
          {confirmForget ? (
            <div
              role="alertdialog"
              aria-label="Forget Last.fm API keys"
              onKeyDown={(e) => { if (e.key === "Escape") setConfirmForget(false); }}
              style={{ display: "flex", flexWrap: "wrap", gap: 10, alignItems: "center" }}
            >
              <span style={{ flex: "1 1 220px", fontSize: 13, lineHeight: 1.5, color: "var(--color-text)" }}>
                Forget your Last.fm API key and secret? You'll need to paste them again to reconnect.
              </span>
              <GhostBtn autoFocus onClick={() => setConfirmForget(false)}>
                Cancel
              </GhostBtn>
              <GhostBtn danger onClick={forget}>
                Forget keys
              </GhostBtn>
            </div>
          ) : (
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10, alignItems: "center" }}>
              <GhostBtn subtle onClick={disconnect}>
                Disconnect
              </GhostBtn>
              <div style={{ flex: 1 }} />
              <GhostBtn subtle onClick={() => setConfirmForget(true)}>
                Forget API keys
              </GhostBtn>
            </div>
          )}
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
          <Field label="API key">
            <input
              value={apiKey}
              onChange={(e) => setApiKey(e.currentTarget.value)}
              placeholder={
                status?.configured ? "•••••••• (saved)" : "Paste your API key"
              }
              disabled={busy}
              autoComplete="off"
              spellCheck={false}
              aria-label="Last.fm API key"
              className="settings-input"
              style={inputStyle}
            />
          </Field>
          <Field label="Shared secret">
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
              aria-label="Last.fm shared secret"
              className="settings-input"
              style={inputStyle}
            />
          </Field>
          {err && (
            <p
              role="alert"
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
          fontSize: "1.143rem",
          fontWeight: 600,
          letterSpacing: "-0.014em",
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
        className="t-title"
        style={{
          margin: 0,
          fontWeight: 700,
          color: "var(--color-text-hi)",
        }}
      >
        Settings
      </h1>

      <PlaybackCard />
      <GeneralCard />
      <FeaturesCard />
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
        <button
          type="button"
          className="disclosure"
          onClick={() => setCollapsed((v) => !v)}
          aria-expanded={!collapsed}
          aria-controls="spotify-dev-app-panel"
          style={{
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: 12,
            padding: 6,
            margin: -6,
            width: "calc(100% + 12px)",
            userSelect: "none",
          }}
        >
          <span style={{ display: "flex", flexDirection: "column", gap: 4, flex: "1 1 240px", minWidth: 0 }}>
            <span
              role="heading"
              aria-level={2}
              style={{
                margin: 0,
                fontSize: "1.143rem",
                fontWeight: 600,
                letterSpacing: "-0.014em",
                color: "var(--color-text-hi)",
              }}
            >
              Spotify developer app
            </span>
            <span className="t-caption" style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
              {isCustom
                ? "Using your own Spotify developer app."
                : "Using Musique's built-in access. Nothing to set up."}
            </span>
          </span>

          <span style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
            <StatusBadge status={status} />
            <motion.span
              animate={{ rotate: collapsed ? 0 : 180 }}
              transition={{ duration: 0.2, ease: EASE_OUT }}
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
            </motion.span>
          </span>
        </button>

        <AnimatePresence initial={false}>
          {!collapsed && (
            <motion.div
              id="spotify-dev-app-panel"
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0, transition: { duration: 0.18, ease: EASE_OUT } }}
              transition={{ duration: 0.26, ease: EASE_OUT }}
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
                Musique works out of the box. If you keep hitting Spotify's rate limits,
                you can connect your own Spotify developer app instead. In your app's
                dashboard, set the redirect URI to{" "}
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
                , then paste its client ID below. The client secret is optional.
              </p>

              <Field label="Client ID">
                <input
                  value={clientId}
                  onChange={(e) => setClientId(e.currentTarget.value)}
                  placeholder={
                    isLoading
                      ? "Loading…"
                      : isCustom
                        ? clientId
                        : "Leave blank to use built-in access"
                  }
                  disabled={busy}
                  autoComplete="off"
                  spellCheck={false}
                  aria-label="Spotify client ID"
                  className="settings-input"
                  style={inputStyle}
                />
              </Field>

              <Field label="Client secret (optional)">
                <input
                  type={showSecret ? "text" : "password"}
                  value={clientSecret}
                  onChange={(e) => setClientSecret(e.currentTarget.value)}
                  placeholder={
                    isLoading
                      ? "Loading…"
                      : isCustom && status === "configured"
                        ? "•••••••••••••••• (saved)"
                        : "Not needed for most setups"
                  }
                  disabled={busy}
                  autoComplete="off"
                  spellCheck={false}
                  aria-label="Spotify client secret"
                  className="settings-input"
                  style={inputStyle}
                />
                <button
                  type="button"
                  onClick={() => setShowSecret((v) => !v)}
                  aria-label={showSecret ? "Hide client secret" : "Show client secret"}
                  aria-pressed={showSecret}
                  title={showSecret ? "Hide" : "Show"}
                  className="btn-pill"
                  style={{
                    width: 42,
                    height: 42,
                    padding: 0,
                    borderRadius: 10,
                    color: "var(--color-text)",
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
                  role="alert"
                  style={{ margin: 0, fontSize: 12.5, color: "var(--color-danger)" }}
                >
                  {validationError}
                </p>
              )}

              <div style={{ display: "flex", flexWrap: "wrap", gap: 10, paddingTop: 4 }}>
                <PrimaryBtn onClick={() => save()} disabled={!canSave}>
                  {saving ? "Saving…" : "Save client ID"}
                </PrimaryBtn>
                <div style={{ flex: 1 }} />
                <GhostBtn
                  subtle
                  onClick={() => resetToDefault()}
                  disabled={!canReset}
                >
                  {resetting ? "Switching…" : "Use built-in access"}
                </GhostBtn>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.section>
    </div>
  );
}
