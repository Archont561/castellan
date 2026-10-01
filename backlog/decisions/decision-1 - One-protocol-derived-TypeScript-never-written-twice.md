---
id: decision-1
title: "One protocol, with TypeScript derived from it — never written twice"
date: '2026-10-01 20:05'
status: accepted
---
## Context

Three faces (desktop, mobile, extension) plus a future CLI and LAN sync all need to agree on
message shapes. The KeePass ecosystem's most-complained-about failure class is two halves
drifting: extension expects one shape, app another, and the user sees "cannot connect".
geoquery solves the same problem for its SDKs by deriving TypeScript from the Rust types
with ts-rs through an `xtask` crate; the derivation is committed so drift is a reviewable
diff.

## Decision

**Adopted, as in geoquery.** `crates/protocol` is the single source of truth: every message
is a Rust type with `#[ts(export)]`; `castellan-xtask` exports them into
`packages/protocol/src/generated` (barrel + `version.ts` included), the output is committed,
and `@castellan/protocol`'s typecheck depends on the codegen turbo task so a stale
generation fails the build. `cargo test -p castellan-protocol` regenerates the same bytes
through `TS_RS_EXPORT_DIR` (`.cargo/config.toml`), giving one output and two doors.

Hand-written TypeScript for a wire shape is a review blocker, not a style preference.

## Consequences

- Adding a protocol method touches four files (variant, result, client method, dispatch
  arm) and the compiler finds the forgotten ones.
- The generated files are noisy in review by nature (ts-rs formatting); the mitigation is
  that protocol changes are *supposed* to be loud.
- Field naming is snake_case on the wire and in TS, so both sides read the same. Revisit
  only if a second non-Rust producer of the protocol ever appears.
