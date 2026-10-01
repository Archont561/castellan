import { e2ePreset } from "@castellan/utils/playwright";

/**
 * E2E for Fob loads the *built* extension (`.output/chrome-mv3`, produced by
 * the `wxt build` the e2e script chains before the runner) into a real
 * Chromium.
 *
 * The default headless mode is `chrome-headless-shell`, which cannot load
 * extensions at all; `channel: "chromium"` is the full Chrome-for-Testing
 * binary in new-headless mode, which can. The specs pass it again on the
 * `launchPersistentContext` call, because MV3 needs a persistent profile.
 */
export default e2ePreset({
  testDir: "./e2e",
  timeout: 60_000,
  use: { channel: "chromium" }
});
