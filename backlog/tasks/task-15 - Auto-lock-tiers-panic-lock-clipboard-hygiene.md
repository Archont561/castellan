---
id: TASK-15
title: "Auto-lock tiers, panic lock, clipboard hygiene"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - security
  - ux
dependencies: [TASK-7]
references:
  - crates/vault
  - apps/desktop/src-tauri
priority: high
ordinal: 1500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The locking discipline from the design: soft lock on blur/sleep/OS lock with biometric quick-unlock later, hard lock after reboot/update/N-days, panic-lock hotkey, clipboard auto-clear with a configurable timeout.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Blur/sleep/OS-lock triggers soft lock within the configured window (tested with a fake clock)
- [ ] #2 Panic hotkey locks instantly from any app state, including mid-fill
- [ ] #3 Clipboard copies clear after the timeout; the clear is verified, not fire-and-forget
- [ ] #4 Hard-lock triggers (reboot, update, enrollment change) each have a test
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Timer plumbing in the app shell; lock policy table-driven and testable; clipboard clear with OS verification where the platform allows (Windows ClipboardContentOptions history exclusion too).
<!-- SECTION:PLAN:END -->
