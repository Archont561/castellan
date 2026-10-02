import { svelte } from "@sveltejs/vite-plugin-svelte";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

/**
 * Storybook's builder reads the project's vite config and expects to find
 * the Svelte plugin here — the framework package no longer injects it
 * itself. Without this file, `.svelte` files (the stories and the
 * renderer's own components) reach rollup uncompiled and the preview
 * build fails. The component tests are unaffected: they bring their own
 * vite config through `ctViteConfig` in playwright.config.ts.
 *
 * UnoCSS is here for the same reason it is in each app: the components
 * carry utility classes now, and the harness that renders them has to
 * generate those utilities — from ./uno.config.ts, the same shared preset
 * the apps use.
 */
export default defineConfig({
  plugins: [UnoCSS(), svelte()]
});
