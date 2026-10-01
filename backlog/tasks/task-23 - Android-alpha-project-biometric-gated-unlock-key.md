---
id: TASK-23
title: "Android alpha: project, biometric-gated unlock key"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - mobile
  - security
dependencies: [TASK-7]
references:
  - apps/mobile
  - backlog/docs/specifications/biometric-unlock/doc-5
priority: high
ordinal: 2300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
First mobile build: tauri android init, the biometric unlock plugin (Keystore AES key with setUserAuthenticationRequired + setInvalidatedByBiometricEnrollment, StrongBox where available) wrapping the vault key, and the autofill-provider groundwork via Credential Manager.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 APK installs and unlocks a KDBX with password; biometric unlock wraps and unwraps the vault key in Keystore
- [ ] #2 New fingerprint enrollment invalidates the wrapped key → password required again (tested on device)
- [ ] #3 Failed-attempt backoff and the hard-lock policy from task-15 apply on mobile
- [ ] #4 The app appears as an autofill provider in Android settings (fill working is v0.4's task-30)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Kotlin plugin with createBioKey/wrap/unwrap/enrollmentChanged surface; Rust side holds only wrapped blobs; device tests are manual-checklist CI-exempt for now.
<!-- SECTION:PLAN:END -->
