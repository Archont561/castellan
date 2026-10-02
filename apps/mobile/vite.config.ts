import { sveltekit } from "@sveltejs/kit/vite";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  // See the desktop config: UnoCSS runs first, and reads ./uno.config.ts.
  plugins: [UnoCSS(), sveltekit()],
  // The dev port matches devUrl in src-tauri/tauri.conf.json; changing one
  // and not the other is a very confusing ten minutes.
  server: {
    port: 5174,
    strictPort: true
  }
});
