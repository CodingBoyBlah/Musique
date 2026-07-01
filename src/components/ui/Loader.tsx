import { motion } from "framer-motion";

interface Props {
  // optional caption under the spinner
  label?: string;
  // fills the available vertical space and centres itself
  fill?: boolean;
  size?: number;
}

// calm loading indicator - a soft rotating arc that fades in so it never pops -- replaced the old bare "Loading…" text DONE
export function Loader({ label, fill = true, size = 26 }: Props) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.25, ease: "easeOut" }}
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
      <motion.svg
        width={size}
        height={size}
        viewBox="0 0 50 50"
        animate={{ rotate: 360 }}
        transition={{ duration: 0.85, ease: "linear", repeat: Infinity }}
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
      </motion.svg>
      {label && (
        <motion.span
          initial={{ opacity: 0 }}
          animate={{ opacity: 0.55 }}
          transition={{ delay: 0.1, duration: 0.3 }}
          style={{ fontSize: 12.5, fontWeight: 500, letterSpacing: "0.01em", color: "var(--color-text-dim)" }}
        >
          {label}
        </motion.span>
      )}
    </motion.div>
  );
}
