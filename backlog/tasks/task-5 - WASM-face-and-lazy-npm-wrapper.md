---
id: TASK-5
title: "WASM face of the shared logic and the lazy npm wrapper"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:20'
updated_date: '2026-10-01 20:00'
labels:
  - wasm
  - extension
dependencies:
  - TASK-2
  - TASK-3
references:
  - crates/wasm/src/lib.rs
  - packages/wasm/src/index.ts
priority: medium
ordinal: 500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose otpauth parsing (without the secret) and origin matching to the extension through wasm-bindgen; wrap in @castellan/wasm with lazy loading so an MV3 service worker never pays for an import it does not use.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `parse_otpauth` returns issuer/account/digits/period and provably no secret bytes
- [x] #2 `origin_matches` is the identical function the app links natively
- [x] #3 Wrapper throws a readable "run bun run wasm" error when the pkg output is missing
- [x] #4 wasm-pack builds for the bundler target; JS/WASM artifacts are ignored and declarations are committed
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Plain-Rust core fn + thin wasm_bindgen wrapper so host tests exercise the real function; serde-wasm-bindgen for the JS value; wasm-bindgen declarations type the module and ts-rs derives `OtpAuthInfo` from the Rust projection.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The plain Rust function keeps host tests independent of JS values. The package suite additionally instantiates the built WASM artifact with wasm-bindgen's glue and exercises the exported wrapper.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
One implementation, two targets: the extension pre-filters with the same rules the app applies authoritatively.
<!-- SECTION:FINAL_SUMMARY:END -->
