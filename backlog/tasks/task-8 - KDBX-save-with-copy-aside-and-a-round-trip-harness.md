---
id: TASK-8
title: "KDBX save with copy-aside and a round-trip harness"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - vault
  - security
dependencies: [TASK-6]
references:
  - crates/vault
  - backlog/docs/specifications/vault-format-policy/doc-3
priority: high
ordinal: 800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The save path under decision-3's rule: copy the existing file aside (timestamped), write the new one, keep the copy. The harness that makes this trustworthy: parse saved output field-by-field against the original across a corpus of KeePassXC/Strongbox-authored databases, so 'drops fields keepass-rs does not parse' becomes a caught regression, not a lost database.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Save writes a valid KDBX 4 that KeePassXC and Strongbox open without warnings
- [ ] #2 The copy-aside exists before any write; a failed save leaves the original untouched
- [ ] #3 Round-trip harness compares every parsed field (incl. custom data, icons, history) and fails on any loss
- [ ] #4 Harness runs in CI against the corpus; corpus files are committed fixtures
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
keepass-rs save behind a save() that does copy→write→verify; harness as an integration test over fixtures; document the retention/prune policy in the app settings.
<!-- SECTION:PLAN:END -->
