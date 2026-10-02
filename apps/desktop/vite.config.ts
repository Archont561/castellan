import { sveltekit } from "@sveltejs/kit/vite";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

import { DEV_PORT } from "./dev.config";

export default defineConfig({
  // UnoCSS before sveltekit: the extraction pass has to see the Svelte
  // source before the Kit plugin compiles it away, and the generated
  // `virtual:uno.css` has to exist before Kit's own CSS handling runs.
  // Config comes from ./uno.config.ts, which is two lines around the
  // workspace-shared preset.
  plugins: [UnoCSS(), sveltekit()],
  // Tauri's devUrl is the source; dev.config.ts validates and extracts it.
  server: {
    port: DEV_PORT,
    strictPort: true
  }
});
