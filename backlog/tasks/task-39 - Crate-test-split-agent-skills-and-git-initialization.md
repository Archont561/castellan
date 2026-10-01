---
id: TASK-39
title: 'Crate test split, agent skills, and git initialization'
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 21:12'
updated_date: '2026-10-01 21:13'
labels:
  - infrastructure
  - testing
dependencies: []
priority: high
ordinal: 4800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Move the crates' inline #[cfg(test)] tests into per-crate tests/ integration files over the public API, install the agent skill set (registry tdd + five pixi-sandbox picks, adapted), and initialize git with lefthook and a conventional first commit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The five test-bearing crates have tests in crates/<name>/tests/*.rs and no inline mod tests in src — cargo test/clippy/fmt green
- [x] #2 Seven skills installed under .agents/skills and pinned in skills-lock.json (backlog and session adapted to this repo)
- [x] #3 Repository initialized on main with arena-agent identity + lefthook hooks + one conventional initial commit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Move tests per crate, adapt skills, git init + commit
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Integration-test files: ipc/framing.rs (6), otp/otpauth.rs (20, data-encoding dev-dep), protocol/origin_matching.rs (13; ts-rs export tests are derive-generated and stay in the lib), vault/entry_projection.rs (1, save->open fixture via dev-only save_kdbx4 feature) + vault/passphrase.rs (7, WORDS now pub), wasm/shared_logic.rs (2). Total 62, identical to the inline baseline.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All three ACs green: tests moved to per-crate tests/ over the public API (62/62, clippy+fmt clean, vault open() gained its first coverage via the save->open fixture), seven skills pinned in skills-lock.json (backlog+session adapted to bun run backlog and the ephemeral-sandbox bootstrap), git initialized on main with lefthook and a conventional initial commit.
<!-- SECTION:FINAL_SUMMARY:END -->
