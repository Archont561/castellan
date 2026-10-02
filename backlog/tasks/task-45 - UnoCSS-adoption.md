---
id: TASK-45
title: UnoCSS adoption
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 22:14'
updated_date: '2026-10-02 13:30'
labels:
  - ux
  - infrastructure
dependencies: []
priority: low
ordinal: 12800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The styling item deferred earlier and never tracked: UnoCSS as a workspace devDependency with one shared preset consumed by packages/ui and the SvelteKit apps - atomic utilities without shipping a component framework's CSS. Component tests and Storybook must stay green through the migration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 UnoCSS is a workspace devDependency with one shared preset consumed by packages/ui and the SvelteKit apps
- [x] #2 Existing components render identically (component tests and Storybook unchanged green)
- [x] #3 The extract pipeline runs through the vite plugin without breaking WXT or SvelteKit builds
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Shared config is `unoPreset()` in `@castellan/utils/uno` (new `./uno` export); each consumer's `uno.config.ts` is one line around it, the only option being `shell` (html/body preflight: on for the two SvelteKit faces, off for the extension popup and the library harnesses, which render into a surface somebody else owns). The look is shortcuts (c-entry-row, c-badge, c-field, c-action, c-section-title, c-ring-track, c-ring-progress) — in `@castellan/ui/uno`, see the last paragraph — plus custom `*-tint-<n>` rules for the oklab currentColor mixes; metrics stay as utilities in the markup, and no property appears in both (a shortcut radius colliding with a face's own `rounded-*` leaves the winner to source order). Both `app.css` files and every component `<style>` block are gone; the tokens are the preset's preflight.

AC#1: `unocss@66.10.5` devDependency at the root and in the five consumers; one preset, four configs. AC#3: vite plugin wired in apps/desktop, apps/mobile (before `sveltekit()`), packages/ui (Storybook builder + the CT `ctViteConfig`, which names the config file explicitly because its vite root is `playwright/`) and apps/extension via `wxt.config.ts`'s `vite()` hook. All four builds green: desktop and mobile SvelteKit static, `wxt build` (popup CSS 2.7 kB, no html/body shell), Storybook. Each built bundle was grepped for the shared package's classes — `c-entry-row`, `c-badge` and the oklab tints are present in the desktop, mobile and Storybook CSS, which is the thing that proves the app-level extraction reaches a package consumed as source.

AC#2: the Playwright layer **did** run in the end. `scripts/offline-browsers.ts` (`bun run browsers:offline`) provisions chromium from npm — `@sparticuz/chromium` ships the binary brotli-compressed inside the tarball, `@ffmpeg-installer/ffmpeg` likewise for the failure videos — into the user cache, shimmed into Playwright's registry layout behind a launcher script that carries `LD_LIBRARY_PATH`. On it: `packages/ui` component tests **10/10** and the desktop and mobile e2e suites green. The extension e2e suite is the one hole: that build is a headless shell with no extensions subsystem (no `extensions::` symbols, no `chrome-extension://` scheme), so `--load-extension` is ignored and the MV3 service worker never registers — it needs full Chrome-for-Testing, i.e. a reachable `cdn.playwright.dev`. It exercises no shortcut, and the popup's built CSS was verified by hand (2.7 kB, tokens and `data-[state=…]` variants, no page shell).

Parity was proven twice over. Mechanically first: 26 class lists generated through the real config and compared declaration-by-declaration against the deleted CSS, every value identical except three deliberate ones (badge radius 999px → 9999px, both pills; `background: none` → `background-color: transparent` on the row button; the popup's `system-ui, sans-serif` folded into the shared `--font-sans`). Then in a browser: the new `packages/ui/tests/styling.test.ts` mounts the components and reads computed styles back — grid columns, the inherited font, the 0.7em badge, `transition-property: stroke-dashoffset`, `stroke-width: 2.5px`. No test or e2e selector changed, by design: the bare semantic class (`username`, `badge`, `status`, `error`, `progress`) stays first on each element as a hook carrying no CSS.

Follow-up applied in the same task: the shortcuts live in **`presetUi()` in `@castellan/ui/uno`** (package root, not `src/`, which the extractor scans), layered onto the foundation through a new `presets` option on `unoPreset()`. `@castellan/utils/uno` keeps what everything needs — tokens, tints, the `shell` preflight, the extraction pipeline — because the extension popup depends on utils and not on the component library. The rebuilt CSS has the same content hashes as before the move, which is the whole claim: a pure relocation.
<!-- SECTION:NOTES:END -->
