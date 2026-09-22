import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  optimizeDeps: { include: ["wanakana", "pinyin-pro"] },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-react":  ["react", "react-dom", "react-router-dom"],
          "vendor-motion": ["framer-motion"],
          "vendor-query":  ["@tanstack/react-query"],
          romanize:        ["wanakana", "pinyin-pro"],
        },
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      
      ignored: [
        "**/src-tauri/**",
        "**/drizzle/**",
        "**/db/**",
        "**/.git/**",
        "**/node_modules/**",
        "**/*.md",
        "**/*.txt",
        "**/*.log",
        "**/*.png",
        "**/*.jpg",
        "**/*.jpeg",
        "**/*.ico",
        "**/*.svg",
      ],
    },
  },
}));
