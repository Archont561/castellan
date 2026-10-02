import { e2ePreset } from "@castellan/utils/playwright";

import { DEV_PORT, DEV_URL } from "./dev.config";

/**
 * E2E for the desktop face drives the SvelteKit layer in a plain browser:
 * the Tauri shell is a webview around exactly this app, and the Tauri IPC
 * is one seam (see `@castellan/tauri`) — what these tests pin down is
 * the shell every user sees first and the honest-degradation contract when
 * the native side is absent.
 */
export default e2ePreset({
  testDir: "./e2e",
  use: { baseURL: DEV_URL },
  webServer: {
    // --strictPort: a silent fallback would turn every test into a navigation
    // failure. The port and URL both derive from Tauri's build.devUrl.
    command: `bun run dev --port ${DEV_PORT} --strictPort`,
    url: DEV_URL,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
});
