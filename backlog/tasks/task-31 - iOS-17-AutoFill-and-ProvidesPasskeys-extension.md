---
id: TASK-31
title: "iOS 17 AutoFill and ProvidesPasskeys extension"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - mobile
  - passkeys
dependencies: [TASK-23]
references:
  - apps/mobile
  - backlog/docs/research/passkey-providers/doc-7
priority: medium
ordinal: 3100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The Swift side: an AutoFill credential-provider extension declaring ProvidesPasskeys, passkey registration/assertion with Face ID, QuickType suggestions for passwords, and the shared keychain handoff from the main app (App Group).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Passkey create/assert work in Safari and in WKWebView apps, Face ID gated
- [ ] #2 Password autofill with QuickType suggestions from the vault
- [ ] #3 The extension reads the vault through the App Group + Secure Enclave wrapped key, never through the network
- [ ] #4 excludedCredentials handled (iOS 18 API where available, graceful earlier)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Swift extension target in the Xcode project tauri ios generates; types bridged through the protocol's JSON; Face ID via LAContext per ceremony.
<!-- SECTION:PLAN:END -->
