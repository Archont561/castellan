import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

/**
 * Storybook's builder reads the project's vite config and expects to find
 * the Svelte plugin here — the framework package no longer injects it
 * itself. Without this file, `.svelte` files (the stories and the
 * renderer's own components) reach rollup uncompiled and the preview
 * build fails. The component tests are unaffected: they bring their own
 * vite config through `ctViteConfig` in playwright.config.ts.
 */
export default defineConfig({
  plugins: [svelte()]
});
