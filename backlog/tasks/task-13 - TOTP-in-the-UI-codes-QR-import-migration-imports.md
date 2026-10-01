---
id: TASK-13
title: "TOTP in the UI: codes, QR import, migration imports"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - otp
  - ux
dependencies: [TASK-3,TASK-7]
references:
  - crates/otp
  - packages/ui/src/components/TotpRing.svelte
priority: high
ordinal: 1300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Live codes with the TotpRing countdown (driven by seconds_remaining from the protocol, never client-computed periods), otpauth QR import from image file and screen capture, and the Google Authenticator migration QR format.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Codes tick in the ring component from protocol answers; no period arithmetic in TS
- [ ] #2 otpauth:// URIs import from pasted text, image files, and screen capture (desktop)
- [ ] #3 Google Authenticator export QRs (multi-account otpauth-migration:// payloads) parse and import as a batch with per-account review
- [ ] #4 Aegis and andOTP exports import (their file formats are JSON/zip with documented layouts)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Migration URI parsing in castellan-otp (it is protocol-shaped logic, shared with the WASM face); QR decode in the app via a Rust barcode crate; batch import UI with review.
<!-- SECTION:PLAN:END -->
