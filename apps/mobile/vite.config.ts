import { sveltekit } from "@sveltejs/kit/vite";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

import { DEV_PORT } from "./dev.config";

export default defineConfig({
  // See the desktop config: UnoCSS runs first, and reads ./uno.config.ts.
  plugins: [UnoCSS(), sveltekit()],
  // Tauri's devUrl is the source; dev.config.ts validates and extracts it.
  server: {
    port: DEV_PORT,
    strictPort: true
  }
});
