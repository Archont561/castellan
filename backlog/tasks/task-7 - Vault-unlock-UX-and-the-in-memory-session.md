---
id: TASK-7
title: Vault unlock UX and the in-memory session
status: Done
assignee:
  - '@me'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-03 12:58'
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
- [x] #1 Unlock succeeds against KeePassXC-authored KDBX 4 files (corpus test), fails with a stable protocol error code on wrong password
- [x] #2 Lock wipes derived keys via drop; a locked vault answers list requests with vault_locked, not an error
- [x] #3 Soft lock after configurable blur/sleep/timer; hard lock (password required) after reboot, app update, or N days
- [x] #4 DatabaseUnlocked/DatabaseLocked events fire on the event bus so faces reflect lock state live
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Session state machine in the vault crate (no Tauri types), dispatchers call into it; timer wiring in the apps; event bus test with a scripted clock.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Started 2026-10-03. Design per plan + doc-3: session state machine in castellan-vault (Empty/Unlocked/Locked, no Tauri types), time passed as data (scripted times in tests = the scripted clock), LockPolicy {blur,sleep,idle,hard_after} with soft/hard tiers, event bus on the session emitting protocol Event values; protocol gains RpcErrorCode::BadCredentials for wrong-password unlock; castellan-dispatch gains a session-aware Dispatcher (locked -> vault_locked precedes not_implemented); Tauri shells get unlock/lock/status commands + event forwarding (compiled by CI - this sandbox lacks the GTK stack).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done 2026-10-03. Session state machine landed in castellan-vault (session.rs): Empty/Unlocked/Locked, VaultSession over a SessionClock seam (SystemClock in production, ScriptedClock in tests), LockPolicy {after_blur, after_wake, after_idle, hard_after} with soft/hard tiers, zeroize-on-drop of derived keys, and an event bus emitting protocol Events. Wrong password/keyfile maps to bad_credentials, broken files to vault_unreadable (VaultError::error_code); the corpus unlock matrix (13 fixtures incl. KeePassXC 2.7.12 KDBX 4.1 and keyfile-only) runs in tests/session.rs — 39/39. The dispatcher is session-aware (Dispatcher::new(Arc<VaultSession>), 24/24): a locked session answers GetEntries with vault_locked, LockDatabase locks and fires DatabaseLocked. Tauri shells (desktop + mobile) manage one Dispatcher over one session, expose unlock_vault/lock_vault/vault_status plus the rpc door, forward session events to the webview bus on castellan://event (the @castellan/tauri transport's onEvent now listens; 8/8), and run a 1 s lock-tier tick (idle + hard-lock horizon). Shells compile in CI (this sandbox has no webkit stack; verified per the session skill's exclude-scoped gates).
<!-- SECTION:FINAL_SUMMARY:END -->
