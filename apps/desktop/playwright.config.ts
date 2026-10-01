import { e2ePreset } from "@castellan/utils/playwright";

/**
 * E2E for the desktop face drives the SvelteKit layer in a plain browser:
 * the Tauri shell is a webview around exactly this app, and the Tauri IPC
 * is one seam (see `src/lib/transport.ts`) — what these tests pin down is
 * the shell every user sees first and the honest-degradation contract when
 * the native side is absent.
 */
export default e2ePreset({
  testDir: "./e2e",
  use: { baseURL: "http://localhost:5173" },
  webServer: {
    // --strictPort: a silent drift to 5175 would turn every test into a
    // navigation failure against the wrong server, which is worse than a
    // loud port conflict. Mobile deliberately takes 5174 so the two faces
    // can run in parallel under turbo.
    command: "bun run dev --port 5173 --strictPort",
    url: "http://localhost:5173",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
});
