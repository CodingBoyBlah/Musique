import { useEffect, useId, useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ArrowUpRight,
  Check,
  ChevronDown,
  Globe,
  Link2,
  Loader2,
  X,
} from "../../lib/icons";
import { isTopKeyLayer, popKeyLayer, pushKeyLayer } from "../../lib/keyLayers";
import type { LoginLinks } from "../../api/auth";
import { useAuth } from "../../hooks/useAuth";
import { MusiqueLogo } from "./MusiqueLogo";
import "./SignInPrompt.css";

const LOGIN_TABS = [
  { key: "ncspot", label: "ncspot", description: "Your Spotify account" },
  {
    key: "desktop",
    label: "Spotify desktop",
    description: "Your playback connection",
  },
] as const;

function LoginLinksPanel({
  links,
  preparing,
  prepare,
}: {
  links: LoginLinks | null;
  preparing: boolean;
  prepare: () => void;
}) {
  const [copied, setCopied] = useState<string | null>(null);
  const [manual, setManual] = useState(false);
  const [message, setMessage] = useState("");

  async function copy(key: keyof LoginLinks | "both") {
    if (!links) return;
    try {
      await navigator.clipboard.writeText(
        key === "both" ? `${links.ncspot}\n${links.desktop}` : links[key],
      );
      setCopied(key);
      setMessage(
        key === "both"
          ? "Both links copied. Paste each into a separate tab."
          : "Link copied. Paste it into your browser's address bar.",
      );
    } catch {
      setCopied(null);
      setManual(true);
      setMessage(
        "Select and copy the links below. Your browser could not copy them automatically.",
      );
    }
  }

  async function open(key: keyof LoginLinks) {
    if (!links) return;
    try {
      await openUrl(links[key]);
    } catch {
      setManual(true);
      setMessage(
        "Your browser did not open. Copy the link and paste it into a new browser tab.",
      );
    }
  }

  return (
    <>
      <div className="sign-in-dropdown-heading">
        <h3>Choose a sign-in link</h3>
        <p>Use the same Spotify account in both tabs.</p>
      </div>
      <div className="sign-in-dropdown-links">
        {LOGIN_TABS.map(({ key, label, description }) => (
          <div className="sign-in-dropdown-link" key={key}>
            <div className="sign-in-dropdown-row">
              <button
                type="button"
                className="sign-in-copy-link"
                disabled={!links}
                aria-label={`Copy ${label} link`}
                onClick={() => void copy(key)}
              >
                <span className="sign-in-tab-icon" aria-hidden="true">
                  <Globe size={17} />
                </span>
                <span className="sign-in-link-label">
                  <strong>{label}</strong>
                  <span>{description}</span>
                </span>
                <span className="sign-in-copy-label">
                  {copied === key ? (
                    <>
                      <Check size={13} /> Copied
                    </>
                  ) : (
                    "Copy"
                  )}
                </span>
              </button>
              <button
                type="button"
                className="sign-in-open-link"
                disabled={!links}
                aria-label={`Open ${label} tab`}
                title={`Open ${label} tab`}
                onClick={() => void open(key)}
              >
                <ArrowUpRight size={17} aria-hidden="true" />
              </button>
            </div>
            {manual && links && (
              <input
                className="sign-in-manual-link"
                aria-label={`${label} sign-in link`}
                readOnly
                value={links[key]}
                onFocus={(event) => event.currentTarget.select()}
              />
            )}
          </div>
        ))}
      </div>
      <div className="sign-in-dropdown-footer">
        {links ? (
          <button
            type="button"
            className="sign-in-copy-both"
            onClick={() => void copy("both")}
          >
            {copied === "both" ? (
              <Check size={15} aria-hidden="true" />
            ) : (
              <Link2 size={15} aria-hidden="true" />
            )}
            {copied === "both" ? "Both links copied" : "Copy both links"}
          </button>
        ) : preparing ? (
          <p className="sign-in-preparing" role="status">
            <Loader2 size={14} aria-hidden="true" /> Preparing your links...
          </p>
        ) : (
          <button type="button" className="sign-in-copy-both" onClick={prepare}>
            Prepare new links
          </button>
        )}
        <p className="sign-in-copy-message" role="status">
          {message}
        </p>
      </div>
    </>
  );
}

export function SignInPrompt({
  heading,
  title,
  description,
}: {
  heading?: string;
  title: string;
  description: string;
}) {
  const { login, loggingIn, loginLinks, generation } = useAuth();
  const [helpOpen, setHelpOpen] = useState(false);
  const [linksOpen, setLinksOpen] = useState(false);
  const copyRef = useRef<HTMLDivElement>(null);
  const copyButtonRef = useRef<HTMLButtonElement>(null);
  const helpButtonRef = useRef<HTMLButtonElement>(null);
  const focusDropdown = useRef(false);
  const helpId = useId();
  const linksId = useId();

  useEffect(() => {
    if (!linksOpen) return;
    const layer = pushKeyLayer();
    const onPointer = (event: PointerEvent) => {
      if (!copyRef.current?.contains(event.target as Node)) setLinksOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !isTopKeyLayer(layer)) return;
      event.preventDefault();
      event.stopPropagation();
      setLinksOpen(false);
      copyButtonRef.current?.focus();
    };
    document.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey, true);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey, true);
      popKeyLayer(layer);
    };
  }, [linksOpen]);

  useEffect(() => {
    if (!linksOpen || !focusDropdown.current) return;
    const first = copyRef.current?.querySelector<HTMLButtonElement>(
      ".sign-in-copy-link:not(:disabled)",
    );
    if (first) {
      first.focus();
      focusDropdown.current = false;
    }
  }, [linksOpen, loginLinks]);

  function showLinks(keyboard = false) {
    focusDropdown.current = keyboard;
    setLinksOpen(true);
    if (!loginLinks && !loggingIn) login(false);
  }

  return (
    <div className="sign-in-page">
      {heading && <h1 className="t-title sign-in-page-heading">{heading}</h1>}
      <section className="sign-in-content" aria-label="Spotify sign-in">
        <MusiqueLogo size={60} className="sign-in-logo" />
        <h2 className="sign-in-title">{title}</h2>
        <p className="sign-in-description">{description}</p>
        <div className="sign-in-controls">
          <div className="sign-in-actions">
            <button
              type="button"
              className="btn-primary sign-in-login"
              disabled={loggingIn}
              aria-busy={loggingIn || undefined}
              onClick={() => {
                setLinksOpen(false);
                login();
              }}
            >
              {loggingIn ? (
                <>
                  <Loader2 size={16} aria-hidden="true" /> Waiting for
                  sign-in...
                </>
              ) : (
                "Log in with Spotify"
              )}
            </button>
            <div
              ref={copyRef}
              className="sign-in-copy"
              onBlur={(event) => {
                if (
                  event.relatedTarget &&
                  !event.currentTarget.contains(event.relatedTarget as Node)
                )
                  setLinksOpen(false);
              }}
            >
              <button
                ref={copyButtonRef}
                type="button"
                className="btn-pill sign-in-copy-trigger"
                aria-expanded={linksOpen}
                aria-controls={linksOpen ? linksId : undefined}
                onClick={() => (linksOpen ? setLinksOpen(false) : showLinks())}
                onKeyDown={(event) => {
                  if (event.key === "ArrowDown") {
                    event.preventDefault();
                    showLinks(true);
                  }
                }}
              >
                Copy links{" "}
                <ChevronDown
                  size={14}
                  className="sign-in-chevron"
                  aria-hidden="true"
                />
              </button>
              {linksOpen && (
                <div
                  id={linksId}
                  className="sign-in-dropdown"
                  role="region"
                  aria-label="Copy Spotify sign-in links"
                >
                  <LoginLinksPanel
                    key={`${generation}:${loginLinks ? "ready" : "waiting"}`}
                    links={loginLinks}
                    preparing={loggingIn}
                    prepare={() => login(false)}
                  />
                </div>
              )}
            </div>
          </div>
          <div className="sign-in-caption">
            <p>
              {loggingIn
                ? "Finish both browser tabs to continue."
                : "Two tabs will open. Premium is not required."}
            </p>
            <button
              ref={helpButtonRef}
              type="button"
              className="sign-in-help-trigger"
              aria-expanded={helpOpen}
              aria-controls={helpOpen ? helpId : undefined}
              onClick={() => {
                setLinksOpen(false);
                setHelpOpen(!helpOpen);
              }}
            >
              Help <ChevronDown size={12} aria-hidden="true" />
            </button>
          </div>
        </div>
        {helpOpen && (
          <section
            id={helpId}
            className="sign-in-help"
            aria-labelledby={`${helpId}-title`}
          >
            <div className="sign-in-help-heading">
              <h3 id={`${helpId}-title`}>Two tabs, one Spotify account</h3>
              <button
                type="button"
                className="sign-in-help-close"
                aria-label="Close sign-in help"
                onClick={() => {
                  setHelpOpen(false);
                  helpButtonRef.current?.focus();
                }}
              >
                <X size={15} aria-hidden="true" />
              </button>
            </div>
            <ol className="sign-in-steps">
              <li>
                <strong>Use your usual browser</strong>
                <p>
                  Open a browser where you are logged in to Spotify. A free
                  account works too.
                </p>
              </li>
              <li>
                <strong>Approve both tabs</strong>
                <p>
                  Click "Log in with Spotify", then approve{" "}
                  <span className="sign-in-app-name">ncspot</span> and{" "}
                  <span className="sign-in-app-name">Spotify desktop</span> with
                  the same account.
                </p>
              </li>
              <li>
                <strong>Missing a tab?</strong>
                <p>
                  Open "Copy links" above. Copy each link into a separate
                  browser tab, or use the arrow beside it to open it.
                </p>
              </li>
            </ol>
            <p className="sign-in-help-note">
              Keep Musique open. Links expire after 5 minutes. If sign-in times
              out, try again for fresh links.
            </p>
          </section>
        )}
      </section>
    </div>
  );
}
