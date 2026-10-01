---
id: TASK-29.1
title: 'castellan-webauthn: the soft authenticator core'
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - passkeys
  - security
dependencies: []
parent_task_id: TASK-29
priority: high
ordinal: 8800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The riskiest new code of the flagship pulled out of the browser path and into a pure library crate: ES256 credential generation and assertion over COSE/CBOR structures, signature counters, and RP ID hash validation. Testable against W3C vectors with zero browsers involved - the extension interception and the app both sit on top of it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ES256 credential generation and assertion produce W3C WebAuthn L3-shaped COSE/CBOR structures verified against published vectors
- [ ] #2 RP ID hash validation refuses a mismatched origin before any key material is touched
- [ ] #3 Attestation format none plus a self-attestation option and a monotonic per-credential signature counter
- [ ] #4 The crate is a pure library with no dependency on any face or UI crate
<!-- AC:END -->
