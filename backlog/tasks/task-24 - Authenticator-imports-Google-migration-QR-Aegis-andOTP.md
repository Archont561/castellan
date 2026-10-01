---
id: TASK-24
title: "Authenticator imports: Google migration QR, Aegis, andOTP"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - otp
  - mobile
dependencies: [TASK-13]
references:
  - crates/otp
priority: medium
ordinal: 2400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Complete the authenticator-exodus path: otpauth-migration:// batch import with per-account review, Aegis JSON/vault exports (incl. encrypted), andOTP plain and encrypted — the feature that pulls KeePassDX users over in one afternoon.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Google migration QRs import as a batch with per-account accept/skip
- [ ] #2 Aegis exports import, including password-encrypted ones (its format is documented AES-GCM)
- [ ] #3 andOTP both formats import; Steam guard seeds land with a clear 'display format unsupported, code computation ok' note if applicable
- [ ] #4 Import summary shows per-source counts and any skipped accounts with reasons
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Parsers in castellan-otp (shared with WASM so the extension could import too); decryption keys via the existing crypto deps, no new primitives.
<!-- SECTION:PLAN:END -->
