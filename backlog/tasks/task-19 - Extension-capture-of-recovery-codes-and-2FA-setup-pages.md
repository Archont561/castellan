---
id: TASK-19
title: "Extension capture of recovery codes and 2FA setup pages"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - extension
  - ux
dependencies: [TASK-18,TASK-12]
references:
  - apps/extension/entrypoints
priority: medium
ordinal: 1900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The 1Password-tier detail: detect recovery-code pages (known GitHub/npm screens plus generic DOM heuristics: a table or block of similar tokens) and offer 'Save as recovery codes'; detect otpauth QR setup flows and offer to store the seed with the entry just saved.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 GitHub and npm recovery-code screens are detected specifically
- [ ] #2 Generic heuristic: N similar tokens in code/table elements triggers the offer with a review step — never a silent save
- [ ] #3 Setup QRs for known origins offer seed storage bound to the matching entry
- [ ] #4 Every capture shows exactly what will be stored before it is
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Heuristics as content-script rules with per-site allowlists first, generic second; all captures go through save flows the user reviews.
<!-- SECTION:PLAN:END -->
