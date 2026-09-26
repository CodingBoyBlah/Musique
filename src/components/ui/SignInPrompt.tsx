import { useAuth } from "../../hooks/useAuth";
import { EmptyState } from "./EmptyState";
import { MusiqueLogo } from "./MusiqueLogo";

/* the one signed-out state every page shows. used to be pasted four times with
drifting copy ("..." vs "…") and a hint that named the redirect port. */
export function SignInPrompt({
  heading,
  title,
  description,
}: {
  // the page's own h1, kept so the signed-out page still says where you are
  heading?: string;
  title: string;
  description: string;
}) {
  const { login, loggingIn } = useAuth();

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "clamp(20px, 2.5vw, 28px)" }}>
      {heading && (
        <h1 className="t-title" style={{ margin: 0, color: "var(--color-text-hi)" }}>
          {heading}
        </h1>
      )}
      <EmptyState
        icon={
          <MusiqueLogo
            size={56}
            style={{ filter: "drop-shadow(0 8px 24px rgba(88, 115, 216, 0.32))" }}
          />
        }
        iconContainerStyle={{ opacity: 1, marginBottom: 8 }}
        title={title}
        description={description}
        action={
          <button
            type="button"
            className="btn-primary"
            onClick={() => login()}
            aria-disabled={loggingIn || undefined}
            aria-busy={loggingIn || undefined}
            style={{ height: 38, padding: "0 24px", cursor: loggingIn ? "progress" : "pointer" }}
          >
            {loggingIn ? "Waiting for browser…" : "Log in with Spotify"}
          </button>
        }
        hint={loggingIn ? "Finish signing in in your browser." : undefined}
      />
    </div>
  );
}
