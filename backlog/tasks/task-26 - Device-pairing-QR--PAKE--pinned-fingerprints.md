---
id: TASK-26
title: "Device pairing: QR + PAKE + pinned fingerprints"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - sync
  - security
dependencies: [TASK-25]
references:
  - backlog/docs/specifications/device-mesh/doc-6
priority: high
ordinal: 2600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two Castellan devices become a pair: one shows a QR (identity key fingerprint + shared-secret commitment), the other scans; an SPAKE2-style PAKE derives the pairing key without either side transmitting it; device identity keys are pinned from then on.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Pairing completes with a QR scan; no secret ever crosses the wire (test captures traffic)
- [ ] #2 Pairing keys rotate; a device can be unpaired from either side and the other side notices
- [ ] #3 The pairing UI names devices and shows key fingerprints for the paranoid
- [ ] #4 A lost-device flow marks the pairing revoked; sync refuses and reports
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
PAKE via a vetted crate (spake2 or opaque), never hand-rolled; device identities in the app's own store, separate from the vault.
<!-- SECTION:PLAN:END -->
