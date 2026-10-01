import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/**
 * Static output with an SPA fallback. There is no SSR inside Tauri — the
 * webview loads local files, all data arrives over `invoke()` IPC, and
 * prerendering load functions would run at *build* time in Node where
 * `invoke` does not exist. `ssr = false` in the root layout finishes the
 * story; dynamic vault routes (`/entry/[id]`) use the fallback shell.
 */
/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // The repo-wide `@` alias (package/app root — see AGENTS.md), taught to
    // both tsc and vite by the framework instead of hand-maintained twin
    // configs. `$lib` stays the SvelteKit-idiomatic alias for src/lib.
    alias: { "@": "./" },
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html"
    })
  }
};

export default config;
