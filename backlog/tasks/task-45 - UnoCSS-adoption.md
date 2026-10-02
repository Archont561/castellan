---
id: TASK-45
title: UnoCSS adoption
status: In Progress
assignee:
  - '@agent'
created_date: '2026-10-01 22:14'
updated_date: '2026-10-02 11:40'
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
- [ ] #2 Existing components render identically (component tests and Storybook unchanged green)
- [x] #3 The extract pipeline runs through the vite plugin without breaking WXT or SvelteKit builds
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Shared config is `unoPreset()` in `@castellan/utils/uno` (new `./uno` export); each consumer's `uno.config.ts` is one line around it, the only option being `shell` (html/body preflight: on for the two SvelteKit faces, off for the extension popup and the library harnesses, which render into a surface somebody else owns). The look is shortcuts (c-entry-row, c-badge, c-field, c-action, c-section-title, c-ring-track, c-ring-progress) plus custom `*-tint-<n>` rules for the oklab currentColor mixes; metrics stay as utilities in the markup, and no property appears in both (a shortcut radius colliding with a face's own `rounded-*` leaves the winner to source order). Both `app.css` files and every component `<style>` block are gone; the tokens are the preset's preflight.

AC#1: `unocss@66.10.5` devDependency at the root and in the five consumers; one preset, four configs. AC#3: vite plugin wired in apps/desktop, apps/mobile (before `sveltekit()`), packages/ui (Storybook builder + the CT `ctViteConfig`, which names the config file explicitly because its vite root is `playwright/`) and apps/extension via `wxt.config.ts`'s `vite()` hook. All four builds green: desktop and mobile SvelteKit static, `wxt build` (popup CSS 2.7 kB, no html/body shell), Storybook. Each built bundle was grepped for the shared package's classes — `c-entry-row`, `c-badge` and the oklab tints are present in the desktop, mobile and Storybook CSS, which is the thing that proves the app-level extraction reaches a package consumed as source.

AC#2 is NOT ticked: this sandbox cannot download a browser (cdn.playwright.dev and GitHub release assets both blocked), so `packages/ui` component tests and the three e2e suites could not run. Instead parity was proven mechanically: 26 class lists generated through the real config and compared declaration-by-declaration against the deleted CSS. Every value matches except three deliberate ones — badge radius 999px → 9999px (both pills), `background: none` → `background-color: transparent` on the row button, and the popup's `system-ui, sans-serif` folded into the shared `--font-sans`. No test or e2e selector changed, by design: the bare semantic class (`username`, `badge`, `status`, `error`, `progress`) stays first on each element as a hook carrying no CSS. New guard `packages/utils/test/uno.test.ts` (9 tests) pins the extraction patterns, the shortcut declarations and the `shell` split. Remaining to close: `bun run --cwd packages/ui test` and `bun run all:e2e` on a machine with chromium.
<!-- SECTION:NOTES:END -->
