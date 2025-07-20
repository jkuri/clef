import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import checker from "vite-plugin-checker";

export default defineConfig({
  plugins: [react(), tailwindcss(), checker({ typescript: true, biome: { command: "check" } })],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: { open: true },
  preview: { open: true },
  build: { outDir: "dist", emptyOutDir: true, chunkSizeWarningLimit: 3000 },
});
