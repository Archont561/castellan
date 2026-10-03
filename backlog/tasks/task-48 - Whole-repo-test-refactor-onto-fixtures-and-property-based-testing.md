---
id: TASK-48
title: Whole-repo test refactor onto fixtures and property-based testing
status: Done
assignee:
  - '@me'
created_date: '2026-10-03 10:47'
updated_date: '2026-10-03 11:09'
labels:
  - testing
dependencies: []
priority: high
ordinal: 14900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Refactor survey (refactor skill, whole repo): five suites already carry the AGENTS.md convention (ipc/framing, otp/otpauth, protocol/origin_matching, vault/passphrase, vault/entry_projection on the Rust side; utils/fixtures, core/properties, protocol/roundtrip, wasm/wasm on the TS side) and stay untouched. The gaps: crates/dispatch and crates/wasm tests are plain #[test] functions with no rstest/proptest; packages/core/test/client.test.ts re-defines the ScriptedTransport/TestClient pair that properties.test.ts already defines (duplicated code); packages/tauri and apps/extension transport tests hand-roll their fakes per test with no fixture and no property. Bring every gap onto the convention: rstest #[fixture]/#[case] and proptest in Rust, createFixture and fast-check in TS, behavior preserved throughout.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 dispatch tests: #[case] matrix over RpcMethod id correlation + proptest property for any request id; rstest/proptest dev-deps from workspace
- [x] #2 wasm shared_logic tests: #[case] matrix mirroring origin matching + proptest agreement property against castellan-protocol for any generated host; otpauth cases
- [x] #3 core client tests: one shared fixture module (TestClient + ScriptedTransport defined once; scriptedClient fixture via createFixture) used by client.test.ts and properties.test.ts
- [x] #4 tauri + extension transport tests: createFixture for the fake invoke/port runtime; fast-check property for response passthrough/correlation
- [x] #5 AGENTS.md rule: implementing changes consults the local tdd and refactor skills under .agents/skills
- [x] #6 Gates green, behavior preserved: biome + tsc + bun test; cargo test/clippy/fmt over the library crates (Tauri app crates stay CI lane)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Refactor-skill loop, baseline already verified green (70 Rust lib tests, 14 turbo test tasks): 1) dispatch suite onto rstest+proptest, 2) wasm suite onto rstest+proptest, 3) extract the shared core fixture and fold both suites onto it, 4) tauri transport onto createFixture+fast-check, 5) extension transport onto createFixture+fast-check, 6) AGENTS.md agent-rule bullet, 7) full gates, AC check-off, final summary. One suite per step, tests run after each.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Baseline captured 2026-10-03: cargo test green on all 8 library crates (70 tests), bun turbo test green on 14 tasks; apps/*/src-tauri excluded in-sandbox (system webkit missing, CI installs it). Survey: dispatch.rs 3 plain tests, shared_logic.rs 2 plain tests, client.test.ts duplicates 2 classes from properties.test.ts, tauri/extension transport tests hand-roll fakes inline.

Implemented 2026-10-03, one suite per step, tests run after each: dispatch.rs 3->19 tests (6-operation case matrix incl. SaveEntry/GetEntries arms the old suite never touched, variant-correlation matrix, clamp cases, any-id and any-passphrase-request properties); wasm shared_logic.rs 2->10 (documented matrix, delegation property: the WASM re-export never disagrees with castellan-protocol over generated hosts, otpauth wrapper cases); core: TestClient+ScriptedTransport extracted to test/support.ts (the two files had drifted - one transport supported pushed events, the other dropped them), scriptedClient fixture via createFixture, property bodies construct per fc.assert run (a per-test fixture lifecycle cannot reach inside a property run - learned the hard way in the tauri suite first); tauri transport 2->4 (ScriptedInvoke fixture, passthrough-by-identity property over the full RpcRequest/RpcResponse registries, rejection-normalization property); extension transport 3->5 (nativeHost fixture killing the per-test port wiring, burst property: any burst = one hello + ordered requests + reverse-order answers resolve by id, disconnect property). Gates: biome clean, tsc green, 14/14 turbo test tasks, cargo fmt/clippy -D warnings/test 94 passed (70 baseline), cargo deny ok, codegen:check no drift. apps/*/src-tauri unverifiable in this sandbox (system webkit), CI lane.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
All six ACs green. Five suites that already carried the fixtures+PBT convention were left untouched; the five gaps were brought onto it with behavior preserved: dispatch 3->19 tests, wasm 2->10, core de-duplicated onto one shared support module + scriptedClient fixture, tauri 2->4 with a passthrough-by-identity property over full request/response registries, extension 3->5 with a burst/multiplexing property and a disconnect property. AGENTS.md now opens implementation work in the local tdd + refactor skills. Gates: biome, tsc, 14/14 turbo test tasks, cargo fmt/clippy -D warnings, 94 library-crate tests (70 baseline), cargo deny, codegen:check - all clean; Tauri app crates remain CI's lane (system webkit).
<!-- SECTION:FINAL_SUMMARY:END -->
