---
id: TASK-17
title: "Password-manager importers: Bitwarden, Chrome, KeePassXC CSV"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - vault
  - ux
dependencies: [TASK-8]
references:
  - crates/vault
priority: medium
ordinal: 1700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Arrival path for switchers: Bitwarden JSON (encrypted and plaintext exports), Chrome/Edge CSV, KeePassXC CSV. Preview-and-confirm import: nothing written until the user approves the mapping.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Bitwarden encrypted JSON unlocks with the export passphrase; item types map to entries with folders → groups
- [ ] #2 Chrome CSV imports with duplicate detection; URIs become entry URLs for the matching rule
- [ ] #3 Import preview shows exactly what will be created; failures report row and reason
- [ ] #4 TOTP seeds carried in Bitwarden totp URIs land as otp fields so codes work immediately
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Importer traits in the vault crate (parse → NewEntry list), UI in the desktop app; encrypted-Bitwarden needs its KDF (PBKDF2/Argon2) — implement against the documented format, test with fixture exports.
<!-- SECTION:PLAN:END -->
