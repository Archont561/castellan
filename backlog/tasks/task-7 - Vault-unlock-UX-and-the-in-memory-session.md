---
id: TASK-7
title: "Vault unlock UX and the in-memory session"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - vault
  - ux
  - security
dependencies: []
references:
  - crates/vault/src/lib.rs
  - apps/desktop/src-tauri/src/lib.rs
priority: high
ordinal: 700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Open/lock flows as the user meets them: master password (+ keyfile) prompt, unlock into an in-memory session, lock tiers (blur/sleep/OS lock/timer) with zeroize-on-drop, and the vault status events the protocol already defines. This is where the session state machine lives — one place, shared by desktop and mobile dispatchers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Unlock succeeds against KeePassXC-authored KDBX 4 files (corpus test), fails with a stable protocol error code on wrong password
- [ ] #2 Lock wipes derived keys via drop; a locked vault answers list requests with vault_locked, not an error
- [ ] #3 Soft lock after configurable blur/sleep/timer; hard lock (password required) after reboot, app update, or N days
- [ ] #4 DatabaseUnlocked/DatabaseLocked events fire on the event bus so faces reflect lock state live
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Session state machine in the vault crate (no Tauri types), dispatchers call into it; timer wiring in the apps; event bus test with a scripted clock.
<!-- SECTION:PLAN:END -->
