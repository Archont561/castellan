---
id: TASK-20
title: "Audit dashboard: weak, reused, old, expiring"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - security
  - ux
dependencies: [TASK-7,TASK-8]
references:
  - apps/desktop/src/routes
priority: medium
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The health view: zxcvbn-scored weak passwords, reuse clusters, entries older than a threshold, expiring ones, and accounts where 2FA is available but not enabled (2fa.directory data bundled locally, no network lookup for that check).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Weak/reused/old/expiring computed on unlock in Rust; the face only renders
- [ ] #2 Reuse clusters group by password hash without ever exposing the hash across entries in the UI
- [ ] #3 2FA-available suggestions from bundled 2fa.directory data, checked offline
- [ ] #4 Drill-down from every finding to the entries; fixing marks the finding resolved on next audit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
zxcvbn-style scoring in Rust (or a crate if one is maintainable); audit results as protocol types so the extension/CLI can surface them too.
<!-- SECTION:PLAN:END -->
