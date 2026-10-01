---
id: TASK-35
title: "Emergency kit and optional Shamir splitting"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - security
  - recovery
dependencies: [TASK-22]
references:
  - backlog/docs/specifications/vault-format-policy/doc-3
priority: medium
ordinal: 3500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The no-account recovery story made physical: a printable emergency kit (vault path, KDF parameters, keyfile as QR, and a write-your-password-here prompt — the password cannot be recovered, by design, and the kit says so), plus optional 3-of-5 Shamir splitting of the master password for households.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Kit renders deterministically to PDF/printable HTML from unlock-time data
- [ ] #2 Keyfile QR round-trips (scan → keyfile works to unlock)
- [ ] #3 Shamir mode splits a secret 3-of-5, recombines, and refuses 2 shares (tested)
- [ ] #4 The kit's copy states plainly what is and is not recoverable
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Shamir via a vetted crate (ssss-style or vss-rs equivalent); PDF from a template engine, no network fonts.
<!-- SECTION:PLAN:END -->
