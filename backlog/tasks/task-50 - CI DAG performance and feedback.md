---
id: TASK-50
title: "CI DAG performance and feedback"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-05 00:00'
updated_date: '2026-10-05 00:00'
labels:
  - infrastructure
  - performance
priority: high
ordinal: 5000
---

## Description
Refactor GitHub Actions CI into parallel cheap validation and dependent heavyweight lanes without reducing coverage.

## Acceptance Criteria
- [x] Removed unnecessary runner disk cleanup.
- [x] Cheap workflow, Biome, typecheck, Rust format, and generated checks run independently.
- [x] Native and browser dependencies are installed only in lanes that need them.
- [x] Obsolete PR/branch runs are cancelled by concurrency.
- [x] Tests, builds, E2E, Storybook, and advisories remain blocking checks.
- [x] Test annotations cannot mask a missing output file or original failure.
- [x] actionlint-compatible workflow paths and pinned Pixi setup are preserved.

## Implementation Notes
Split the former serial `checks` job into `workflow-lint`, `static-checks`, `typecheck`, `rust-format`, `generated-checks`, `build-and-test`, `e2e`, `storybook`, and `advisories`. The native apt install and Playwright dependency install now occur only after their prerequisites. Pixi's lock-backed cache remains enabled per job; Cargo and Turbo caches remain scoped to the build/test lane. No local GitHub runner timing baseline was available in the repoless sandbox; the measurable improvement is earlier scheduling/reporting of cheap failures and no disk reclamation step.
