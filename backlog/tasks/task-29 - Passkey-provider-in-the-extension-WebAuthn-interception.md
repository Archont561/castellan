---
id: TASK-29
title: "Passkey provider in the extension (WebAuthn interception)"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - passkeys
  - extension
  - security
dependencies: [TASK-9,TASK-7]
references:
  - backlog/docs/research/passkey-providers/doc-7
  - apps/extension/entrypoints/content.ts
priority: high
ordinal: 2900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The soft-YubiKey on desktop: the content script patches navigator.credentials at document_start in the MAIN world; ceremonies become protocol requests; the app enforces RP-ID against the requesting origin and stores keys in the vault. The 1Password/Bitwarden technique, with enforcement in the trusted process.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Passkey creation and assertion work on GitHub, npm, and Facebook from Chrome and Firefox
- [ ] #2 RP-ID validation happens in the app (origin + RP ID checked against the entry before any key use); a malicious page cannot request another site's key
- [ ] #3 Ceremonies require biometric/platform-auth verification per use, configurable
- [ ] #4 The page's original functions are preserved untouched; unpatching on disconnect is clean
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Interception layer behind a connected-app gate; keys as vault entries (castellan.passkey fields); ceremony state machine shared design with the mobile providers.
<!-- SECTION:PLAN:END -->
