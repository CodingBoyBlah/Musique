import { motion } from "framer-motion";
import { ChevronDown } from "@/lib/icons";
import { EASE_OUT, PRESS, PRESS_TRANSITION } from "../../lib/motion";
import { Tooltip } from "./Tooltip";

export type SortDir = "asc" | "desc";

/* Sort direction as one button that flips, not a two-item dropdown. There are
   only two states, so a menu was a click to open plus a click to choose for
   what is really a toggle - and the arrow says which way the list runs. */
export function SortDirectionToggle({
  dir,
  onChange,
}: {
  dir: SortDir;
  onChange: (dir: SortDir) => void;
}) {
  const asc = dir === "asc";
  return (
    <Tooltip label={asc ? "Ascending" : "Descending"} side="top">
      <motion.button
        type="button"
        onClick={() => onChange(asc ? "desc" : "asc")}
        aria-label={asc ? "Sorted ascending. Switch to descending" : "Sorted descending. Switch to ascending"}
        className="focus-ring"
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        style={{
          display: "flex", alignItems: "center", justifyContent: "center",
          width: 32, height: 32, borderRadius: 8, cursor: "pointer", flexShrink: 0,
          border: "1px solid var(--color-glass-border)",
          background: "var(--color-glass)",
          color: "var(--color-text-hi)",
        }}
      >
        <motion.span
          initial={false}
          animate={{ rotate: asc ? 180 : 0 }}
          transition={{ duration: 0.2, ease: EASE_OUT }}
          style={{ display: "flex" }}
        >
          <ChevronDown size={14} strokeWidth={2.5} />
        </motion.span>
      </motion.button>
    </Tooltip>
  );
}
