import { motion } from "framer-motion";
import { Heart } from "@/lib/icons";
import { Tooltip } from "./Tooltip";
import { useIsAlbumSaved, useToggleAlbumSave } from "../../hooks/useLibrary";
import { PRESS, PRESS_TRANSITION } from "../../lib/motion";
import { toast } from "../../store/toast.store";
import { errMsg } from "../../lib/err";

// the album page's "add to library" pill, same shape as the artist Follow pill
export function SaveAlbumButton({ id }: { id: string }) {
  const { data: saved = false } = useIsAlbumSaved(id);
  const toggle = useToggleAlbumSave();

  return (
    <Tooltip label={saved ? "Remove from your library" : "Save to your library"} side="top">
      <motion.button
        type="button"
        onClick={() =>
          toggle.mutate(
            { id, saved },
            { onError: (e) => toast.error(`Couldn't update your library: ${errMsg(e)}`) },
          )
        }
        aria-pressed={saved}
        className="ghost-pill focus-ring"
        data-on={saved}
        whileTap={PRESS}
        transition={PRESS_TRANSITION}
        style={{
          height: 36,
          padding: "0 16px",
          borderRadius: 99,
          color: "#ffffff",
          fontSize: 13,
          fontWeight: 600,
          display: "flex",
          alignItems: "center",
          gap: 6,
          cursor: "pointer",
          flexShrink: 0,
        }}
      >
        <Heart size={14} strokeWidth={2.2} active={saved} />
        <span>{saved ? "Saved" : "Save"}</span>
      </motion.button>
    </Tooltip>
  );
}
