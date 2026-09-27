import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import { getEpisode, getEpisodeMedia } from "../../api/podcasts";
import { coverUrl } from "../../lib/coverUrl";
import { EASE_OUT } from "../../lib/motion";
import { usePrefsStore } from "../../store/prefs.store";
import { EPISODE_PREFIX } from "../../utils/episode";
import { releaseLabel } from "../ui/EpisodeRow";
import type { TrackItem } from "../../types/spotify";

function episodeIdOf(track: TrackItem) {
  return track.id.startsWith(EPISODE_PREFIX) ? track.id.slice(EPISODE_PREFIX.length) : track.id;
}

/* the left of the immersive view for a podcast. a video podcast loops its
preview clip (muted - the audio is the episode itself); anything else gets the
artwork, big, with what's playing under it. */
export function PodcastStage({ track, onVideo }: { track: TrackItem; onVideo?: (v: HTMLVideoElement | null) => void }) {
  const id = episodeIdOf(track);
  const animated = usePrefsStore((s) => s.showCanvas);
  const { data: media } = useQuery({
    queryKey: ["episode-media", id],
    queryFn: () => getEpisodeMedia(id),
    staleTime: Infinity,
    retry: false,
    enabled: animated,
  });
  const [cors, setCors] = useState(true);
  const video = animated ? media?.video_preview_url : null;
  const art = coverUrl(track.album?.image_url, 900) ?? track.album?.image_url;

  return (
    <div
      style={{
        position: "absolute",
        left: 0,
        top: 0,
        bottom: 0,
        width: "min(50%, calc(100% - 420px))",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 22,
        padding: "clamp(40px, 6vh, 72px) clamp(24px, 4vw, 64px) 120px",
        boxSizing: "border-box",
        pointerEvents: "none",
      }}
    >
      <AnimatePresence mode="wait" initial={false}>
        {video ? (
          <motion.video
            key={`${video}-${cors}`}
            ref={onVideo}
            src={video}
            crossOrigin={cors ? "anonymous" : undefined}
            onError={() => { if (cors) setCors(false); }}
            autoPlay
            loop
            muted
            playsInline
            poster={media?.thumbnail_url ?? art ?? undefined}
            initial={{ opacity: 0, scale: 0.98 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.4, ease: EASE_OUT }}
            style={{ width: "100%", maxHeight: "56vh", aspectRatio: "16 / 9", objectFit: "cover", borderRadius: 18, boxShadow: "0 30px 80px rgba(0,0,0,0.5)", background: "#000" }}
          />
        ) : (
          <motion.img
            key={art ?? "none"}
            src={art ?? undefined}
            alt=""
            initial={{ opacity: 0, scale: 0.98 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.4, ease: EASE_OUT }}
            style={{ width: "min(100%, 48vh)", aspectRatio: "1 / 1", objectFit: "cover", borderRadius: 18, boxShadow: "0 30px 80px rgba(0,0,0,0.5)" }}
          />
        )}
      </AnimatePresence>
      <div style={{ width: "100%", maxWidth: video ? "100%" : "min(100%, 48vh)", textAlign: video ? "left" : "center", color: "#fff" }}>
        <div style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", opacity: 0.65 }}>
          {track.album?.name ?? "Podcast"}
        </div>
        <div style={{ marginTop: 6, fontSize: "clamp(20px, 2.2vw, 30px)", fontWeight: 800, letterSpacing: "-0.02em", lineHeight: 1.15, display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical", overflow: "hidden" }}>
          {track.name}
        </div>
      </div>
    </div>
  );
}

// the episode's own notes, for the About tab
export function EpisodeAbout({ track, ink }: { track: TrackItem; ink: string }) {
  const id = episodeIdOf(track);
  const { data } = useQuery({ queryKey: ["episode", id], queryFn: () => getEpisode(id), staleTime: 600_000, retry: false });
  const date = releaseLabel(data?.release_date ?? null);
  return (
    <div className="scroll-y" style={{ position: "absolute", inset: 0, overflowY: "auto", paddingRight: 6 }}>
      {date && <div style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: `rgba(${ink}, 0.55)`, marginBottom: 10 }}>{date}</div>}
      <p style={{ margin: 0, fontSize: 15, lineHeight: 1.65, color: `rgba(${ink}, 0.88)`, whiteSpace: "pre-line" }}>
        {data?.description ?? "No notes for this episode."}
      </p>
    </div>
  );
}
