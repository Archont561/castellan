---
id: TASK-10
title: "Native-messaging manifests: detect, write all, repair"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - extension
  - desktop
dependencies: [TASK-9]
references:
  - apps/extension/native-hosts
  - backlog/docs/specifications/native-channel/doc-2
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
One button in settings: detect installed browsers, write every host manifest (HKCU registry on Windows, per-browser NativeMessagingHosts dirs on macOS/Linux) listing all our extension IDs, plus a Repair button that rewrites them all — deleting an entire category of KeePass forum threads.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Chrome, Edge, Brave, Vivaldi, Firefox manifests written from one pass on Windows/macOS/Linux
- [ ] #2 Extension IDs pinned at first store submission appear in allowed_origins; dev ID alongside
- [ ] #3 Repair rewrites all manifests idempotently and reports what it fixed
- [ ] #4 A stale manifest is detected (path moved, ID missing) and surfaced in the panel from task-9
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Detection per platform (registry + well-known dirs), manifest templates from the extension's native-hosts/ dir, e2e test spawning the host from a generated manifest.
<!-- SECTION:PLAN:END -->
