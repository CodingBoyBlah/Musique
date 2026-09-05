import { useState } from "react";
import { motion } from "framer-motion";
import { Sparkles } from "lucide-react";
import { useReflowPulse } from "../hooks/useReflowPulse";
import { PlaygroundThemeBar } from "../components/playground/PlaygroundThemeBar";
import {
  PrimaryPlayButton,
  PulsePlayButton,
  SegmentedControl,
} from "../components/playground/PlaygroundControls";
import {
  MusiqueAlbumCard,
  GenreBucketCard,
  ELECTRONIC_ALBUMS,
} from "../components/playground/PlaygroundCards";
import { AlbumTrackRowsSection } from "../components/playground/PlaygroundTracks";
import {
  SpringToggle,
  PlayerBarVolumeSlider,
} from "../components/playground/PlaygroundDials";

const REFLOW = { type: "spring" as const, stiffness: 340, damping: 38 };

export default function Playground() {
  useReflowPulse();
  const [segmentedMode, setSegmentedMode] = useState("Hi-Res Studio");
  const [pulsePlaying, setPulsePlaying] = useState(true);
  const [primaryPlaying, setPrimaryPlaying] = useState(false);

  return (
    <div
      style={{
        maxWidth: "min(1280px, 100%)",
        margin: "0 auto",
        paddingBottom: 80,
        width: "100%",
        boxSizing: "border-box",
        overflowX: "hidden",
      }}
    >
      {/* Hero Header */}
      <div style={{ marginBottom: 24 }}>
        <div style={{ display: "inline-flex", alignItems: "center", gap: 8, padding: "4px 12px", borderRadius: 99, background: "var(--color-accent-dim)", color: "var(--color-accent)", fontSize: 12, fontWeight: 700, marginBottom: 10 }}>
          <Sparkles size={13} />
          <span>Musique Design System v2.0</span>
        </div>
        <h1 style={{ fontSize: "clamp(24px, 3.2vw, 34px)", fontWeight: 800, color: "var(--color-text-hi)", letterSpacing: "-0.03em", margin: "0 0 6px" }}>
          Component Playground
        </h1>
        <p style={{ fontSize: "clamp(13px, 1.4vw, 14.5px)", lineHeight: 1.6, color: "var(--color-text-dim)", margin: 0, maxWidth: 720 }}>
          Interactive catalog showcasing bespoke Musique components, dynamic theme accent adaptation, and tactile animations.
        </p>
      </div>

      {/* Dynamic Theme Engine Swatcher */}
      <motion.div layout="position" transition={{ layout: REFLOW }}>
        <PlaygroundThemeBar />
      </motion.div>

      <div style={{ display: "flex", flexDirection: "column", gap: "clamp(28px, 3.2vw, 40px)" }}>
        {/* SECTION 1: BUTTONS & CONTROLS */}
        <motion.section layout="position" transition={{ layout: REFLOW }}>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: "clamp(15px, 1.8vw, 18px)", fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px", letterSpacing: "-0.01em" }}>
              1. Buttons & Mode Selectors
            </h2>
            <p style={{ fontSize: "clamp(12px, 1.2vw, 13px)", color: "var(--color-text-dim)", margin: 0 }}>
              Primary pill play button (matching album pages), circular pause/play pulse capsule, and draggable snapping mode switcher.
            </p>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "clamp(12px, 2vw, 18px)", padding: "clamp(16px, 2.5vw, 24px)", borderRadius: 16, background: "var(--color-surface)", border: "1px solid var(--color-border)", boxSizing: "border-box" }}>
            <PrimaryPlayButton
              playing={primaryPlaying}
              onToggle={() => setPrimaryPlaying(!primaryPlaying)}
              label="Play"
            />

            <PulsePlayButton
              playing={pulsePlaying}
              onToggle={() => setPulsePlaying(!pulsePlaying)}
            />

            <SegmentedControl
              options={["Stereo", "Hi-Res Studio", "Spatial Atmos", "Vinyl Warmth"]}
              value={segmentedMode}
              onChange={setSegmentedMode}
            />
          </div>
        </motion.section>

        {/* SECTION 2: ALBUM & GENRE CRATE CARDS */}
        <motion.section layout="position" transition={{ layout: REFLOW }}>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: "clamp(15px, 1.8vw, 18px)", fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px", letterSpacing: "-0.01em" }}>
              2. Album Card & Genre Crate Buckets
            </h2>
            <p style={{ fontSize: "clamp(12px, 1.2vw, 13px)", color: "var(--color-text-dim)", margin: 0 }}>
              Standard album card alongside 2D genre crate buckets (holding slotted album covers, interactive digging on click, and hover fan out).
            </p>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "flex-start", gap: "clamp(14px, 2vw, 24px)", padding: "clamp(16px, 2.5vw, 24px)", borderRadius: 16, background: "var(--color-surface)", border: "1px solid var(--color-border)", boxSizing: "border-box" }}>
            <MusiqueAlbumCard />
            <GenreBucketCard genre="Hip-Hop" />
            <GenreBucketCard genre="Electronic" albums={ELECTRONIC_ALBUMS} />
          </div>
        </motion.section>

        {/* SECTION 3: ALBUM TRACK ROWS */}
        <motion.section layout="position" transition={{ layout: REFLOW }}>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: "clamp(15px, 1.8vw, 18px)", fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px", letterSpacing: "-0.01em" }}>
              3. Song & Track Rows
            </h2>
            <p style={{ fontSize: "clamp(12px, 1.2vw, 13px)", color: "var(--color-text-dim)", margin: 0 }}>
              Album track rows with alternating colors, hover play button, active title recoloring, animated 3-bar equalizer, and dynamic accent pause glyph.
            </p>
          </div>

          <div style={{ padding: "clamp(12px, 2vw, 20px)", borderRadius: 16, background: "var(--color-surface)", border: "1px solid var(--color-border)", boxSizing: "border-box" }}>
            <AlbumTrackRowsSection />
          </div>
        </motion.section>

        {/* SECTION 4: TOGGLES & VOLUME SLIDER */}
        <motion.section layout="position" transition={{ layout: REFLOW }}>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: "clamp(15px, 1.8vw, 18px)", fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px", letterSpacing: "-0.01em" }}>
              4. Toggles & Player Bar Volume Slider
            </h2>
            <p style={{ fontSize: "clamp(12px, 1.2vw, 13px)", color: "var(--color-text-dim)", margin: 0 }}>
              Lossless streaming & spatial atmos spring toggles, and the exact volume slider from the player bar with mute blur/scale animation.
            </p>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "clamp(12px, 2vw, 18px)", padding: "clamp(16px, 2.5vw, 24px)", borderRadius: 16, background: "var(--color-surface)", border: "1px solid var(--color-border)", boxSizing: "border-box" }}>
            <SpringToggle label="Lossless Audio Streaming" initial={true} />
            <SpringToggle label="Spatial Atmos Virtualizer" initial={false} />
            <PlayerBarVolumeSlider initial={75} />
          </div>
        </motion.section>
      </div>
    </div>
  );
}
