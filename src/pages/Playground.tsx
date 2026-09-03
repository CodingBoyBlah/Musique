import { useState } from "react";
import { Sparkles } from "lucide-react";
import { PlaygroundThemeBar } from "../components/playground/PlaygroundThemeBar";
import {
  PrimaryPlayButton,
  PulsePlayButton,
  SegmentedControl,
} from "../components/playground/PlaygroundControls";
import { MusiqueAlbumCard } from "../components/playground/PlaygroundCards";
import { AlbumTrackRowsSection } from "../components/playground/PlaygroundTracks";
import {
  SpringToggle,
  PlayerBarVolumeSlider,
} from "../components/playground/PlaygroundDials";
export default function Playground() {
  const [segmentedMode, setSegmentedMode] = useState("Hi-Res Studio");
  const [pulsePlaying, setPulsePlaying] = useState(true);
  const [primaryPlaying, setPrimaryPlaying] = useState(false);

  return (
    <div style={{ maxWidth: 1040, margin: "0 auto", paddingBottom: 80 }}>
      {/* Hero Header */}
      <div style={{ marginBottom: 24 }}>
        <div style={{ display: "inline-flex", alignItems: "center", gap: 8, padding: "4px 12px", borderRadius: 99, background: "var(--color-accent-dim)", color: "var(--color-accent)", fontSize: 12, fontWeight: 700, marginBottom: 10 }}>
          <Sparkles size={13} />
          <span>Musique Design System v2.0</span>
        </div>
        <h1 style={{ fontSize: 30, fontWeight: 800, color: "var(--color-text-hi)", letterSpacing: "-0.03em", margin: "0 0 6px" }}>
          Component Playground
        </h1>
        <p style={{ fontSize: 13.5, color: "var(--color-text-dim)", margin: 0 }}>
          Interactive catalog showcasing bespoke Musique components, dynamic theme accent adaptation, and tactile animations.
        </p>
      </div>

      {/* Dynamic Theme Engine Swatcher */}
      <PlaygroundThemeBar />

      <div style={{ display: "flex", flexDirection: "column", gap: 40 }}>
        {/* SECTION 1: BUTTONS & CONTROLS */}
        <section>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: 17, fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px" }}>
              1. Buttons & Mode Selectors
            </h2>
            <p style={{ fontSize: 13, color: "var(--color-text-dim)", margin: 0 }}>
              Primary pill play button (matching album pages), circular pause/play pulse capsule, and draggable snapping mode switcher.
            </p>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 18, padding: 20, borderRadius: 14, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
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
        </section>

        {/* SECTION 2: ALBUM CARD */}
        <section>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: 17, fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px" }}>
              2. Album Card
            </h2>
            <p style={{ fontSize: 13, color: "var(--color-text-dim)", margin: 0 }}>
              Random Access Memories album card with hover elevation and floating play button with blur/scale icon animation.
            </p>
          </div>

          <div style={{ padding: 20, borderRadius: 14, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
            <MusiqueAlbumCard />
          </div>
        </section>

        {/* SECTION 3: ALBUM TRACK ROWS */}
        <section>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: 17, fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px" }}>
              3. Song & Track Rows
            </h2>
            <p style={{ fontSize: 13, color: "var(--color-text-dim)", margin: 0 }}>
              Album track rows with alternating colors, hover play button, active title recoloring, animated 3-bar equalizer, and dynamic accent pause glyph.
            </p>
          </div>

          <div style={{ padding: 16, borderRadius: 14, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
            <AlbumTrackRowsSection />
          </div>
        </section>

        {/* SECTION 4: TOGGLES & VOLUME SLIDER */}
        <section>
          <div style={{ marginBottom: 14 }}>
            <h2 style={{ fontSize: 17, fontWeight: 700, color: "var(--color-text-hi)", margin: "0 0 4px" }}>
              4. Toggles & Player Bar Volume Slider
            </h2>
            <p style={{ fontSize: 13, color: "var(--color-text-dim)", margin: 0 }}>
              Lossless streaming & spatial atmos spring toggles, and the exact volume slider from the player bar with mute blur/scale animation.
            </p>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 18, padding: 20, borderRadius: 14, background: "var(--color-surface)", border: "1px solid var(--color-border)" }}>
            <SpringToggle label="Lossless Audio Streaming" initial={true} />
            <SpringToggle label="Spatial Atmos Virtualizer" initial={false} />
            <PlayerBarVolumeSlider initial={75} />
          </div>
        </section>
      </div>
    </div>
  );
}
