/**
 * The reusable bunup preset: the house build choices for the TS library
 * packages, expressed once so five packages cannot drift apart.
 *
 * A package's `bunup.config.ts` is two lines — `defineConfig(libPreset({…}))`
 * — and names its entries; everything else (format, declarations, output
 * shape) is decided here, on purpose: the repo is ESM-only and every
 * consumer (bun, vite, WXT, SvelteKit) speaks ESM, so a CJS output would
 * be a second artifact nobody imports and nobody tests.
 *
 * The build's job today is proof, not delivery: workspace consumers import
 * `src/` directly (HMR and the turbo graph stay simple), so `dist/` is a
 * gitignored artifact that proves each package bundles and emits types
 * cleanly — the day one publishes, flipping its `exports` to `./dist/` is
 * the only change. That is also why `exports` auto-sync stays OFF: bunup
 * would happily rewrite `exports` to dist and quietly change what every
 * workspace member consumes.
 */
import type { DefineConfigItem } from "bunup";

/**
 * The library preset. `entry` is deliberately NOT defaulted — each package
 * names its public entries explicitly, because an accidental glob would
 * turn internals into public artifacts.
 */
export function libPreset(overrides: Partial<DefineConfigItem> = {}): DefineConfigItem {
  return {
    // ESM only; see the file header for why CJS is not a second output.
    format: "esm",
    // Declarations ship with the bundle: a publishable artifact types
    // itself, and the d.ts is what a consumer's editor reads first.
    dts: true,
    // The repo's runtime is bun (tests, scripts); a package whose artifact
    // targets the browser says so in its own override.
    target: "bun",
    outDir: "dist",
    clean: true,
    sourcemap: true,
    ...overrides
  };
}
