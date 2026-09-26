import { useRef, type CSSProperties, type ReactNode, type RefObject } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { EASE_OUT } from "@/lib/motion";
import { useModalA11y } from "@/hooks/useModalA11y";

interface Props {
  open:     boolean;
  onClose:  () => void;
  // accessible name: id of the heading inside the panel
  labelledBy?: string;
  // element to focus when the dialog opens (the safe default action)
  initialFocus?: RefObject<HTMLElement | null>;
  zIndex?:  number;
  panelStyle?: CSSProperties;
  children: ReactNode;
}

/* Shared shell for every centred modal: dimming scrim, a panel that rises a
   few pixels out of a slight scale (never from nothing), and the keyboard
   contract - Escape closes, Tab is trapped, focus returns to the trigger.

   Centred, so the origin stays centre: a modal isn't anchored to anything.
   It leaves faster than it arrives - the user has decided, get out of the way. */
export function Modal({
  open, onClose, labelledBy, initialFocus, zIndex = 1050, panelStyle, children,
}: Props) {
  const panelRef = useRef<HTMLDivElement>(null);
  useModalA11y(panelRef, open, onClose, initialFocus);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1, transition: { duration: 0.16 } }}
          exit={{ opacity: 0, transition: { duration: 0.12 } }}
          onClick={onClose}
          style={{
            position: "fixed", inset: 0, zIndex,
            display: "flex", alignItems: "center", justifyContent: "center",
            background: "rgba(0,0,0,0.5)", backdropFilter: "blur(2px)",
          }}
        >
          <motion.div
            ref={panelRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby={labelledBy}
            tabIndex={-1}
            className="glass-solid-fallback"
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0, transition: { duration: 0.2, ease: EASE_OUT } }}
            exit={{ opacity: 0, scale: 0.97, y: 6, transition: { duration: 0.12, ease: EASE_OUT } }}
            onClick={(e) => e.stopPropagation()}
            style={{
              outline: "none",
              borderRadius: 16,
              background: "rgba(20,20,26,0.97)",
              border: "1px solid rgba(255,255,255,0.12)",
              boxShadow: "0 24px 64px rgba(0,0,0,0.6)",
              ...panelStyle,
            }}
          >
            {children}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
