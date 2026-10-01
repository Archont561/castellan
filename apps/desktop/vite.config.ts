import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [sveltekit()],
  // The dev port matches devUrl in src-tauri/tauri.conf.json; changing one
  // and not the other is a very confusing ten minutes.
  server: {
    port: 5173,
    strictPort: true
  }
});
