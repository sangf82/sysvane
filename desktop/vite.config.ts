import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  // Keeps Rust errors visible while `tauri dev` runs Vite.
  clearScreen: false,
  server: {
    // `devUrl` in src-tauri/tauri.conf.json expects this port.
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
