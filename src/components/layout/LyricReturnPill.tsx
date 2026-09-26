import { AnimatePresence, motion } from "framer-motion";
import { ArrowRight } from "@/lib/icons";
import { EASE_OUT, PRESS } from "../../lib/motion";
import "../../styles/lyrics.css";

/* Shown while the reader has scrolled away from the sung line (see
   hooks/useLyricFollow). Following resumes on its own after a few quiet
   seconds; this is the way back now. Hangs off the nearest positioned
   ancestor, bottom centre. */
export function ReturnPill({ show, onClick }: { show: boolean; onClick: () => void }) {
  return (
    <AnimatePresence>
      {show && (
        <motion.button
          type="button"
          className="lyr-return focus-ring glass-solid-fallback"
          onClick={onClick}
          initial={{ opacity: 0, y: 6 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 6, transition: { duration: 0.12, ease: EASE_OUT } }}
          whileTap={PRESS}
          transition={{ duration: 0.18, ease: EASE_OUT }}
          style={{ x: "-50%" }}
        >
          Back to current line
          <ArrowRight size={13} strokeWidth={2.4} style={{ transform: "rotate(90deg)" }} />
        </motion.button>
      )}
    </AnimatePresence>
  );
}
