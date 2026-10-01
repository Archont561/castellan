---
id: TASK-40
title: 'Repo reshape: git out, pixi features, clap xtask, versionrc, docs move'
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 21:37'
updated_date: '2026-10-01 21:37'
labels:
  - infrastructure
dependencies: []
priority: high
ordinal: 5800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove the git repository, split pixi dependencies into features with two generic router tasks, rebuild xtask on clap, add convco .versionrc, and move the docs site to apps/docs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 No .git in the workspace and lint:workflows runs without one (explicit workflow paths)
- [x] #2 pixi.toml dependencies split across rust/web/utils features with default carrying all three and a fresh lock byte-identical to the committed one
- [x] #3 The pixi task table is two generic routers (bun and xtask) both proven against a full pixi install
- [x] #4 xtask is clap-fronted with --root and a version subcommand reading the workspace manifest (64 tests green)
- [x] #5 .versionrc pins the ten accepted commit types matching the grep fallback and is validated against convco 0.7.2
- [x] #6 The docs site lives at apps/docs with all wiring updated and docs:build green
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Verified in a scratch copy with a full pixi install: pixi run bun --version = 1.3.11 (conda pin) and pixi run xtask version = 0.1.0. Fresh pixi lock of the feature-split manifest is byte-identical to the committed pixi.lock. convco 0.7.2 matrix: all ten .versionrc types accepted (incl. scoped+breaking); revert and wip rejected - grep fallback matches exactly.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All six ACs green. git removed (workspace ships repoless; actionlint given explicit paths so lint:workflows no longer needs a git root); pixi deps split into rust/web/utils features with default carrying all three and the lock provably unchanged; the task table reduced to two generic routers (bun and xtask) proven against a real pixi install; xtask rebuilt on clap with --root and a TOML-parsing version subcommand plus its first two tests; .versionrc added and empirically validated against convco 0.7.2 with the grep fallback kept in exact agreement (revert dropped from both - convco refuses it); docs site moved to apps/docs with lock diff showing exactly the workspace block move. Full sweep green: fmt/lint 93 files, typecheck 14/14, turbo test 11/11, cargo 64 tests + clippy + fmt, e2e 3/3, docs:build 10 pages, okf CONFORMANT, actionlint, frozen lockfile.
<!-- SECTION:FINAL_SUMMARY:END -->
