---
id: TASK-22
title: "Versioned auto-backups with HMAC verification and restore"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - security
  - vault
dependencies: [TASK-8]
references:
  - crates/vault
priority: high
ordinal: 2200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Automatic backups: timestamped copies on every save (the copy-aside stream), retention policy, HMAC over each backup verified before restore, restore with a review diff. Backups are the disaster-recovery story for a no-account product — they must be boring and correct.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every save produces a backup; retention prunes by count and age policy
- [ ] #2 Restore verifies the HMAC and refuses tampered files with a clear error
- [ ] #3 Restore shows a diff of what will change (entries added/removed/changed) before writing
- [ ] #4 Backup integrity runs on app start; corruption is surfaced, not discovered at restore time
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Backup store beside the vault (never inside it); manifest with HMACs; restore path reuses the import preview machinery.
<!-- SECTION:PLAN:END -->
