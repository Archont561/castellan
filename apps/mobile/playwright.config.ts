import { e2ePreset } from "@castellan/utils/playwright";

/**
 * Same SvelteKit layer as the desktop face, so the same e2e shape — only
 * the port differs (5174, so turbo can run both faces in parallel) and the
 * spec emulates a phone viewport, because this is the face that has to
 * work on one.
 */
export default e2ePreset({
  testDir: "./e2e",
  use: { baseURL: "http://localhost:5174" },
  webServer: {
    command: "bun run dev --port 5174 --strictPort",
    url: "http://localhost:5174",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
});
