import { Component, type ErrorInfo, type ReactNode } from "react";
import { errMsg } from "../lib/err";

interface Props {
  children: ReactNode;
}
interface State {
  error: Error | null;
}

/*last line of defence. a render throw anywhere in the tree (bad shape over
 IPC (IPC NOTES ARE IN OBSIDIAN, REFER INSTEAD ), surprise null, whatever) otherwise nukes the whole app to a blank
 window so ds catch it, keep the window alive,
 give them a reload button instead of a dead white screen (IF blank codition fix, TODO DONE) */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    // shows up in devtools console (disabled unless devd)
    console.error("[ui] render crash:", error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    return (
      <div
        style={{
          height: "100vh", width: "100vw",
          display: "flex", flexDirection: "column",
          alignItems: "center", justifyContent: "center", gap: 16,
          padding: 32, textAlign: "center",
          background: "var(--color-base, #0a0a0c)", color: "var(--color-text, #e8e8ea)",
        }}
      >
        <h1 style={{ margin: 0, fontSize: 20, fontWeight: 700 }}>Something broke</h1>
        <p style={{ margin: 0, maxWidth: 460, fontSize: 13.5, lineHeight: 1.5, color: "var(--color-text-dim, #9a9aa2)" }}>
          The app hit an unexpected error and stopped this screen from rendering.
          Your account and library are safe.
        </p>
        <code
          style={{
            maxWidth: 460, padding: "8px 12px", borderRadius: 8,
            fontSize: 12, wordBreak: "break-word",
            background: "rgba(255,255,255,0.06)", color: "var(--color-danger, #f87171)",
          }}
        >
          {errMsg(error)}
        </code>
        <button
          onClick={() => { this.setState({ error: null }); window.location.reload(); }}
          style={{
            padding: "9px 22px", borderRadius: 99, border: "none", cursor: "pointer",
            fontSize: 13.5, fontWeight: 600,
            background: "var(--color-text-hi, #fff)", color: "#0a0a0c",
          }}
        >
          Reload
        </button>
      </div>
    );
  }
}
