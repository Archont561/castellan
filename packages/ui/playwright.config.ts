import { e2ePreset } from "@castellan/utils/playwright";
import { defineConfig } from "@playwright/experimental-ct-svelte";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import UnoCSS from "unocss/vite";

/**
 * Component tests for the ui package: each component mounted in a real
 * browser (the same prebundled chromium the app e2e suites use), driven
 * through the Playwright component-testing API. The shared half of the
 * config is the same `e2ePreset`; the CT half is the vite config that
 * compiles the Svelte components under test.
 */
const base = e2ePreset();

export default defineConfig({
  ...base,
  testDir: "./tests",
  use: {
    ...base.use,
    ctViteConfig: {
      // The CT build runs with its own vite root (`playwright/`), so the
      // config file is named explicitly rather than discovered — otherwise
      // the components mount with their class attributes and no CSS.
      plugins: [
        UnoCSS({ configFile: new URL("./uno.config.ts", import.meta.url).pathname }),
        svelte()
      ]
    }
  }
});
