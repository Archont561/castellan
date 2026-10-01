import type { PlaywrightTestConfig } from "@playwright/test";

/**
 * The shared half of every e2e config in the repo, next to `libPreset` for
 * the same reason: the choices that must not drift between faces live in
 * one place, and each app's `playwright.config.ts` only says what makes it
 * different (its port, its server, its fixtures).
 *
 * The browser itself is not installed by `playwright install` — the root
 * `@playwright/browser-chromium` devDependency downloads it at
 * `bun install` time (bun runs its postinstall because the package is in
 * `trustedDependencies`), so a fresh clone is testable with no manual step.
 * Every playwright package is pinned to one version (1.58.2 — the last
 * stable line of `@playwright/experimental-ct-svelte`, which the ui
 * package's component tests ride); a mismatch between `@playwright/test`
 * and the browser package refuses to launch, so the pins move together.
 */
export function e2ePreset(overrides: Partial<PlaywrightTestConfig> = {}): PlaywrightTestConfig {
  const base: PlaywrightTestConfig = {
    // A test that only fails locally is a test nobody runs; one that only
    // fails in CI is a flake factory. CI gets retries, locally it doesn't.
    forbidOnly: Boolean(process.env.CI),
    retries: process.env.CI ? 2 : 0,
    reporter: [["list"]],
    use: {
      trace: "on-first-retry",
      screenshot: "only-on-failure",
      video: "retain-on-failure"
    }
  };
  return { ...base, ...overrides, use: { ...base.use, ...overrides.use } };
}
