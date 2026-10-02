import { sveltekit } from "@sveltejs/kit/vite";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  // UnoCSS before sveltekit: the extraction pass has to see the Svelte
  // source before the Kit plugin compiles it away, and the generated
  // `virtual:uno.css` has to exist before Kit's own CSS handling runs.
  // Config comes from ./uno.config.ts, which is two lines around the
  // workspace-shared preset.
  plugins: [UnoCSS(), sveltekit()],
  // The dev port matches devUrl in src-tauri/tauri.conf.json; changing one
  // and not the other is a very confusing ten minutes.
  server: {
    port: 5173,
    strictPort: true
  }
});
