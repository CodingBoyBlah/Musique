import { motion } from "framer-motion";
import { SPRING } from "../../lib/motion";

// the on/off switch used across settings and dialogs
export function Switch({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className="switch"
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
      }}
    >
      {/* critically damped: a toggle flip has no momentum to overshoot with */}
      <motion.div
        layout
        transition={{ ...SPRING, duration: 0.25 }}
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
