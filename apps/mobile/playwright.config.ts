import { e2ePreset } from "@castellan/utils/playwright";

import { DEV_PORT, DEV_URL } from "./dev.config";

/**
 * Same SvelteKit layer as the desktop face, so the same e2e shape — only
 * the source-derived port differs so turbo can run both faces in parallel,
 * and the spec emulates a phone viewport because this face has to work on
 * one.
 */
export default e2ePreset({
  testDir: "./e2e",
  use: { baseURL: DEV_URL },
  webServer: {
    command: `bun run dev --port ${DEV_PORT} --strictPort`,
    url: DEV_URL,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
});
