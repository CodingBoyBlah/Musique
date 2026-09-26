interface Props {
  // optional caption under the spinner
  label?: string;
  // fills the available vertical space and centres itself
  fill?: boolean;
  size?: number;
}

// calm loading indicator - a soft rotating arc.
//
// Pure CSS (see .loader in index.css): framer drove the spin from a JS rAF
// loop, which is exactly the loop that stalls while a lazy route chunk is being
// parsed - the moment this spinner is on screen. The fade-in is held back
// ~300ms, so a chunk that arrives quickly never flashes a spinner at all.
export function Loader({ label, fill = true, size = 26 }: Props) {
  return (
    <div
      className="loader"
      role="status"
      aria-label={label ?? "Loading"}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 12,
        width: "100%",
        minHeight: fill ? "42vh" : undefined,
        padding: fill ? 0 : "18px 0",
      }}
    >
      <svg
        className="loader-arc"
        width={size}
        height={size}
        viewBox="0 0 50 50"
        style={{ display: "block" }}
      >
        <circle
          cx="25" cy="25" r="20" fill="none"
          stroke="rgba(255,255,255,0.10)" strokeWidth="4"
        />
        <circle
          cx="25" cy="25" r="20" fill="none"
          stroke="var(--color-accent)" strokeWidth="4" strokeLinecap="round"
          strokeDasharray="80 200"
        />
      </svg>
      {label && (
        <span
          style={{ fontSize: 12.5, fontWeight: 500, letterSpacing: "0.01em", color: "var(--color-text-dim)", opacity: 0.55 }}
        >
          {label}
        </span>
      )}
    </div>
  );
}
