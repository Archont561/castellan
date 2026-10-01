---
id: TASK-6
title: "Vault crate — KDBX open, entry projection, passphrase generation"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:25'
updated_date: '2026-10-01 20:00'
labels:
  - vault
dependencies:
  - TASK-2
references:
  - crates/vault/src/lib.rs
priority: high
ordinal: 600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Open KDBX via keepass 0.15 (password + keyfile), project entries to EntrySummary with TOTP/passkey detection, generate passphrases from an embedded wordlist with rejection-sampled uniform word choice.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Entry projection maps title/username/url, detects TOTP (otp field or TOTP Seed) and passkeys (castellan.passkey), tested against an in-memory database
- [x] #2 Passphrase words are uniform via rejection sampling; different calls produce different phrases (tripwire test)
- [x] #3 VaultError distinguishes IO from keepass failures with paths attached
- [x] #4 No save path exists yet — by design until task-8
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Verify keepass 0.15's real API from the vendored source before writing against it (Database::open takes a reader; EntryRef Derefs to Entry; errors live at db::DatabaseOpenError).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
keepass 0.15 wraps values in Value<T: Zeroize>, so locking is "drop the handle" — recorded in the crate docs. The wordlist is placeholder-grade until the EFF list lands as a data file.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
3 tests green; the projection is the single place KDBX fields become protocol entries.
<!-- SECTION:FINAL_SUMMARY:END -->
