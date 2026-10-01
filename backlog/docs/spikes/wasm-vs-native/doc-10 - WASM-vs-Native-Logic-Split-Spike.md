---
id: doc-10
title: "WASM vs Native Logic Split Spike"
type: spike
created_date: '2026-10-01 21:45'
updated_date: '2026-10-01 21:45'
tags:
  - spike
  - wasm
  - architecture
---
# Spike — which logic compiles to WASM, which stays native

**Status.** Design spike, settled in the scaffold (task-5); recorded because
the boundary will be re-litigated every time a feature lands.

**Question.** The extension runs Rust via WASM; the apps run Rust natively.
What lives where, so "same scoped logic" stays true without the extension
ever holding vault data?

## §1 The rule that fell out

**WASM gets pure functions over data the extension already has. Native gets
anything that touches the vault, the disk, the socket, or a secret.** The
extension's legitimate data: page origins, otpauth URIs it is about to save,
its own UI state. Everything else arrives as protocol answers.

## §2 What shipped on each side

| Logic | Where | Why |
| --- | --- | --- |
| otpauth parsing (no secret exposure) | `castellan-otp` → WASM | validation/preview before save, without an app round-trip |
| origin matching | `castellan-protocol` → WASM | local pre-filter using the *identical* function the app applies |
| TOTP computation | native only | seeds live in the vault; the extension asks via `get_totp` |
| KDBX everything | native only | the extension never holds a database, full stop |
| passphrase generation | native only | it mutates nothing but needs vault-adjacent randomness policy — and the UI asks through RPC anyway |

## §3 The trap this spike exists to prevent

The moment someone "just adds" TOTP computation to the WASM face to save a
round-trip, the extension holds seeds, and the invariant that contains
extension compromise dies. If a feature genuinely needs offline computation
in the browser, the answer is a *synced, encrypted, sealed* cache designed
for it — a decision record, not a task.
