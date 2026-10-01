---
id: TASK-32
title: "Biometric unlock matrix: SE, Keystore, TPM, keychain ACL"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - security
  - mobile
  - desktop
dependencies: [TASK-23]
references:
  - backlog/docs/specifications/biometric-unlock/doc-5
priority: high
ordinal: 3200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Close the hardware-backed loop on every platform: macOS Secure Enclave + Touch ID keychain ACL, Windows Hello TPM key, Linux best-effort (fprintd + libsecret, labeled software-gate). The architecture is fixed (biometrics gate a hardware key that wraps the vault key); the matrix is the work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 macOS: vault key wrapped by an SE key with biometryCurrentSet; new fingerprint forces re-enrollment
- [ ] #2 Windows: Hello-backed TPM key via persisted Platform Crypto Provider key
- [ ] #3 Linux: clearly-labeled convenience mode; the settings copy does not claim hardware isolation it does not have
- [ ] #4 Every platform: invalidation-on-enrollment-change tested on real hardware, documented per-platform quirks
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Per-platform plugin crates behind one trait (createBioKey/wrap/unwrap/enrollmentChanged); the trait already named in doc-5; hardware test checklist in the repo.
<!-- SECTION:PLAN:END -->
