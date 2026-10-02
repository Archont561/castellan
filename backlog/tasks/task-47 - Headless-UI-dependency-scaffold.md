---
id: TASK-47
title: Headless UI dependency scaffold
status: Done
assignee:
  - '@agent'
created_date: '2026-10-02 19:25'
updated_date: '2026-10-02 19:25'
labels:
  - ux
  - infrastructure
dependencies: [TASK-45]
references:
  - packages/ui/package.json
  - apps/extension/package.json
priority: low
ordinal: 13900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Scaffold the unstyled/headless UI dependency layer without adopting a styled component kit: the shared desktop/mobile package gets the Svelte primitives, drawer/touch, positioning, date-peer and icon dependencies it will wrap behind Castellan components; the webextension gets only the positioning primitive needed for page-anchored autofill and passkey overlays.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `@castellan/ui` declares the runtime dependencies for Bits UI, its internationalized-date peer, Floating UI positioning, a Svelte 5 drawer/bottom-sheet primitive, and Lucide icons
- [x] #2 `@castellan/extension` declares `@floating-ui/dom` directly for content-script overlays without pulling the shared app UI package into the extension
- [x] #3 `@castellan/desktop` and `@castellan/mobile` keep `@castellan/ui` as their dependency boundary rather than depending on headless primitives directly
- [x] #4 The Bun lockfile is refreshed from the manifest changes
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The dependency boundary is deliberate: desktop and mobile will consume Castellan-styled wrappers from `packages/ui`, while the extension needs a smaller surface because injected UI must survive arbitrary pages and should use Shadow DOM plus Floating UI rather than the app component stack. The mobile drawer dependency is `@harshmandan/svaul` instead of `vaul-svelte` because it is Svelte 5/runes-first and zero-dependency, avoiding a second legacy Bits UI 0.x tree in the lockfile. Actual wrapper components, stories and interaction tests should land in the follow-up UX task that introduces `packages/ui/src/primitives/*`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Dependency scaffold added: `packages/ui` now has the headless/touch/icon primitives to wrap, the extension has direct overlay positioning, and the app manifests stay behind the shared UI boundary.
<!-- SECTION:FINAL_SUMMARY:END -->
