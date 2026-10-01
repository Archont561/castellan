---
type: Playbook
title: "Castellan — code generation"
description: "The ts-rs + xtask pipeline, the committed generated directory, and the export-dir redirect."
tags:
  - codegen
  - ts-rs
  - xtask
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: infrastructure/codegen
category: infrastructure
refs:
  - project/data-model
  - infrastructure/monorepo
---
# Code generation

**Rule: TypeScript for wire types is generated or it does not exist**
(decision-1). Hand-written wire types are a review blocker.

## Pipeline

1. `crates/protocol` types carry `#[ts(export)]`.
2. `cargo run -p castellan-xtask -- codegen` exports every type via ts-rs
   12's `Config` API (`.with_large_int("number")` — request ids are small,
   `bigint` would tax every caller) into `packages/protocol/src/generated`.
3. xtask writes the barrel (`index.ts` re-exporting all 13) and
   `version.ts` **from the Rust `PROTOCOL_VERSION` constant** — one number,
   two doors.
4. Output is committed. Codegen is idempotent — running it twice produces
   zero diff (verified in CI).

## The two-doors problem and its fix

ts-rs `#[ts(export)]` generates a hidden test per type that writes to
`TS_RS_EXPORT_DIR` (default: `./bindings` inside the crate) on every
`cargo test` — polluting the repo. Root `.cargo/config.toml` redirects it
to the shared generated directory, so `cargo test -p castellan-protocol`
and xtask produce **identical bytes**. A crate-local `.cargo/config.toml`
does not work — cargo reads config cwd-upward from the invocation dir, not
from the crate being tested.

## WASM side-channel

`@castellan/wasm` wraps wasm-pack output for the extension's pure logic
(origin matching, otpauth validation). Built by `bun run wasm` into a
git-ignored `src/pkg`; a hand-written interface declaration keeps typecheck
independent of the artifact; the wrapper throws a readable "run bun run
wasm" error when it is missing.
