---
id: TASK-36
title: "i18n with Polish first-class, a11y pass"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - ux
dependencies: []
references:
  - apps
  - packages/ui
priority: medium
ordinal: 3600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Castellan is built in Warsaw; Polish is a first-class language, not a translation afterthought: full pl + en message catalogs from v0.1 forward, RTL-safe layout where components are shared, and an accessibility pass (keyboard traps, contrast, screen-reader labels on every interactive element).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Message catalogs en + pl complete; no hardcoded strings in components (lint)
- [ ] #2 Language switch live without restart; date/number formatting locale-aware
- [ ] #3 Axe/pa11y clean on the main flows; focus management in palette and fill prompts
- [ ] #4 Polish review by a native speaker recorded in the log
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Svelte i18n in the ui package so all faces share catalogs; string extraction as a turbo task; a11y checks in the Playwright suite.
<!-- SECTION:PLAN:END -->
