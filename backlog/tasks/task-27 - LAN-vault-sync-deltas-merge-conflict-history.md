---
id: TASK-27
title: "LAN vault sync: deltas, merge, conflict history"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - sync
  - vault
dependencies: [TASK-26,TASK-8]
references:
  - backlog/docs/specifications/device-mesh/doc-6
priority: high
ordinal: 2700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The full mesh: encrypted delta sync between paired devices on the LAN, per-entry merge with last-writer-wins plus history, and a conflict view for same-entry-same-field edits — no cloud relay anywhere, ever.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Two devices editing different entries sync cleanly with no user action
- [ ] #2 Same-entry edits produce a reviewable conflict with both versions; resolution is explicit
- [ ] #3 Sync payloads are end-to-end encrypted under the pairing key; a pcap shows no readable metadata beyond transport needs
- [ ] #4 Sync state is idempotent — replaying a sync produces no changes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Change log derived from the entry history KDBX already keeps; delta format as protocol types; merge logic in the vault crate with property tests over concurrent-edit sequences.
<!-- SECTION:PLAN:END -->
