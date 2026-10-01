---
id: TASK-30
title: "Android 14 credential provider (passkeys + autofill)"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - mobile
  - passkeys
dependencies: [TASK-23,TASK-29]
references:
  - apps/mobile
  - backlog/docs/research/passkey-providers/doc-7
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Castellan as a system-wide credential provider through Android's Credential Manager: passkey creation/assertion and password autofill from the vault, gated by the biometric unlock from task-23.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The app appears in Credential Manager as a provider; passkeys created on web AND native apps
- [ ] #2 Assertion requires the Keystore-wrapped key released by biometric prompt
- [ ] #3 Password autofill works in Chrome and at least two native apps
- [ ] #4 Excluded credentials handled (no duplicate passkey offers on re-registration)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Kotlin CredentialProviderEntry plumbing as a Tauri mobile plugin; vault access through the same dispatcher as every face.
<!-- SECTION:PLAN:END -->
