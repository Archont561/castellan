---
id: TASK-2
title: "Protocol crate v1 with ts-rs codegen and a committed generated directory"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:05'
updated_date: '2026-10-01 20:00'
labels:
  - protocol
  - codegen
dependencies: []
references:
  - crates/protocol/src/lib.rs
  - crates/xtask/src/main.rs
  - packages/protocol/src/generated
priority: high
ordinal: 200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The one message language: RpcMethod/RpcResult/Event/Hello/ClientMessage/HostMessage with internally-tagged serde, origin matching, and the TypeScript surface derived by castellan-xtask (barrel + version.ts included, committed).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every type carries ts-rs export; `cargo run -p castellan-xtask -- codegen` regenerates `packages/protocol/src/generated` deterministically
- [x] #2 `version.ts` is generated from the Rust `PROTOCOL_VERSION` constant, not hand-copied
- [x] #3 Origin matching rejects siblings and parent→child, accepts exact host and child→parent, handles ports and URL-shaped inputs (tested)
- [x] #4 No secret material can appear in a list response (EntrySummary carries booleans only)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Types first, tagged serde enums, flatten on the request envelope, xtask exporting each type plus a generated barrel, then wire the turbo dependency edge so typecheck cannot skip codegen.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
ts-rs 12 changed the API: `Config::new().with_out_dir(...).with_large_int("number")` + `T::export_all(&cfg)`; large-int set to number because request ids are small and `bigint` would tax every caller. Origin matching gained scheme/path stripping after its own test caught that inputs are URL-shaped, not bare hosts.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
13 generated type files + barrel + version.ts, all committed; `@castellan/protocol` exports them and nothing behavioral.
<!-- SECTION:FINAL_SUMMARY:END -->
