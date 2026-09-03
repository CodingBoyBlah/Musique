import { useState } from "react";
import { motion } from "framer-motion";
import { RefreshCw, Check, Sparkles } from "lucide-react";
import { applyAccent } from "../../lib/color";

export const PRESET_ACCENTS = [
  { name: "Emerald Green", hex: "#10B981", desc: "Vibrant Spotify Green" },
  { name: "Musique Royal", hex: "#5873D8", desc: "Default Musique Blue" },
  { name: "Electric Violet", hex: "#8B5CF6", desc: "Neon Ultraviolet" },
  { name: "Cyber Cyan", hex: "#06B6D4", desc: "Atmospheric Teal" },
  { name: "Amber Gold", hex: "#F59E0B", desc: "Warm Analog Glow" },
  { name: "Neon Rose", hex: "#F43F5E", desc: "Cyberpunk Rose" },
];

export function PlaygroundThemeBar() {
  const [activeHex, setActiveHex] = useState<string>("#5873D8");

  const pickAccent = (hex: string) => {
    setActiveHex(hex);
    applyAccent(hex);
  };

  const resetToDefault = () => {
    setActiveHex("#5873D8");
    applyAccent("#5873D8");
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        flexWrap: "wrap",
        gap: 16,
        padding: "16px 20px",
        borderRadius: 16,
        background: "rgba(255, 255, 255, 0.03)",
        border: "1px solid rgba(255, 255, 255, 0.08)",
        backdropFilter: "blur(20px)",
        marginBottom: 32,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <div
          style={{
            width: 40,
            height: 40,
            borderRadius: 12,
            background: "var(--color-accent-dim)",
            color: "var(--color-accent)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "0 0 20px -4px var(--color-accent-dim)",
          }}
        >
          <Sparkles size={20} />
        </div>
        <div>
          <div style={{ fontSize: 14, fontWeight: 700, color: "var(--color-text-hi)", letterSpacing: "-0.01em" }}>
            Dynamic Color Accent Engine
          </div>
          <div style={{ fontSize: 12, color: "var(--color-text-dim)" }}>
            Click any swatch to test how every component dynamically adapts to album art / wallpaper colors
          </div>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
        {PRESET_ACCENTS.map((preset) => {
          const isSelected = activeHex.toLowerCase() === preset.hex.toLowerCase();
          return (
            <motion.button
              key={preset.hex}
              whileHover={{ scale: 1.04 }}
              whileTap={{ scale: 0.96 }}
              onClick={() => pickAccent(preset.hex)}
              title={`${preset.name} (${preset.hex})`}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 7,
                padding: "7px 13px 7px 8px",
                borderRadius: 99,
                border: isSelected ? `1.5px solid ${preset.hex}` : "1px solid rgba(255, 255, 255, 0.1)",
                background: isSelected ? "rgba(255, 255, 255, 0.08)" : "rgba(255, 255, 255, 0.02)",
                cursor: "pointer",
                transition: "all 0.15s ease",
              }}
            >
              <span
                style={{
                  width: 14,
                  height: 14,
                  borderRadius: 99,
                  background: preset.hex,
                  boxShadow: isSelected ? `0 0 12px ${preset.hex}` : "none",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                {isSelected && <Check size={10} color="#fff" strokeWidth={3} />}
              </span>
              <span
                style={{
                  fontSize: 12,
                  fontWeight: isSelected ? 600 : 500,
                  color: isSelected ? "var(--color-text-hi)" : "var(--color-text-dim)",
                }}
              >
                {preset.name}
              </span>
            </motion.button>
          );
        })}

        <button
          onClick={resetToDefault}
          title="Reset to Musique Royal Blue"
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "7px 14px",
            borderRadius: 99,
            border: "1px solid rgba(255, 255, 255, 0.08)",
            background: "transparent",
            color: "var(--color-text-dim)",
            cursor: "pointer",
            fontSize: 12,
            transition: "color 0.15s",
          }}
        >
          <RefreshCw size={12} />
          <span>Reset</span>
        </button>
      </div>
    </div>
  );
}
