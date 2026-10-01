---
id: TASK-1
title: "Scaffold the monorepo (bun, turbo façade, biome, lefthook, cargo workspace)"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:00'
updated_date: '2026-10-01 20:00'
labels:
  - infrastructure
dependencies: []
references:
  - Cargo.toml
  - package.json
  - crates/package.json
priority: high
ordinal: 100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Copy and adjust the geoquery/pixi-sandbox monorepo setup: bun workspaces + turbo with the Rust workspace as one task-graph node, biome, lefthook, cargo-deny, strict tsconfig base, dual licensing, CI. Adjustments: virtual Cargo root, no pixi, Tauri app crates as members.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `cargo check`/`cargo test` green across all library crates; clippy `-D warnings` clean; `cargo fmt --check` clean
- [x] #2 `bun run` verbs exist for lint/typecheck/test/build/codegen/wasm and per-face dev; turbo façade at `crates/package.json`
- [x] #3 Workspace resolves all 9 members including both Tauri app crates
- [x] #4 CI workflow expresses the gate as discrete steps with cargo + turbo caches
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Copy the config set from geoquery, adapt per decision-5, validate by running the real toolchain (rustup + cargo) against every crate rather than trusting the copies.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Landed 2026-10-01. Validated in-sandbox: 30 tests pass, zero warnings under `missing_docs` + `clippy::all`, workspace metadata resolves with both app crates. Fixed en route: ts-rs 12's `Config`-based export API, hmac 0.13 needing `KeyInit` in scope, keepass 0.15's error path (`db::DatabaseOpenError`), and the `cargo test`-time ts-rs output redirected to the shared generated dir via root `.cargo/config.toml`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
A 117-file monorepo with three faces, seven crates, four packages, and every gate green. The divergence from geoquery (virtual root, no pixi, codegen task edge, WASM layer) is recorded in decision-5 and infrastructure/monorepo in the knowledge base.
<!-- SECTION:FINAL_SUMMARY:END -->
