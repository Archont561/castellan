---
id: TASK-11
title: "Extension autofill: get_entries and fill"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - extension
  - ux
dependencies: [TASK-9,TASK-10]
references:
  - apps/extension/entrypoints
  - packages/core
priority: high
ordinal: 1100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The daily-driver feature: page-origin detection in the content script, entries offered through the popup and inline, fill on pick. Matching uses the protocol's origin rule — the extension pre-filters with the WASM build, the app answers authoritatively.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Entries offered for the active tab's origin, pre-filtered by the same WASM origin_matches the app uses
- [ ] #2 Fill works on username+password pairs, single fields, and multi-step logins (username page then password page)
- [ ] #3 No fill happens without an explicit user gesture; nothing auto-submits
- [ ] #4 Works on Chrome and Firefox from the one codebase (per-engine quirks live in adapters)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Content script reports origin to background; background asks the app via the client; fill injection behind a user gesture; Playwright tests against a fixture login page in both engines.
<!-- SECTION:PLAN:END -->
