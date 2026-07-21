import { build } from "vite";
import react from "@vitejs/plugin-react";

await build({
  configFile: false,
  base: "./",
  plugins: [react()],
  clearScreen: false,
  root: process.cwd(),
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    outDir: "dist",
    emptyOutDir: true
  }
});
