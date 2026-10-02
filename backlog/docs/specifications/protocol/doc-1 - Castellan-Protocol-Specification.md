---
id: doc-1
title: "Castellan Protocol Specification"
type: specification
created_date: '2026-10-01 21:00'
updated_date: '2026-10-02 15:00'
tags:
  - protocol
  - specification
  - codegen
---
# Specification — the Castellan protocol

**Scope.** The one message language every face speaks. Normative source:
`crates/protocol/src/lib.rs` — this document explains the rules, the Rust is
the truth, and the TypeScript is derived (decision-1).

## §1 Wire shape

Length-prefixed JSON (4-byte LE) over three transports — Tauri `invoke("rpc")`,
native messaging, and (future) the CLI's IPC socket — which share framing
byte-for-byte with Chromium's native messaging. Host→browser messages cap at
1 MB (enforced in `castellan-ipc`); list answers paginate rather than approach
it.

## §2 Requests

`RpcRequest = { id: u64 } & RpcMethod` (flattened). Methods are internally
tagged snake_case: `get_entries`, `get_totp`, `generate_passphrase`,
`save_entry`, `lock_database`, `ping`. Each is declared once in the protocol's
`rpc_contract!` invocation with request fields, success fields, and the client
faces allowed to call it. The macro emits paired Rust variants and operation
metadata; xtask derives the wire DTOs and scoped clients from that metadata.

Rules that keep the protocol safe to expose:

1. **List answers carry no secrets.** `EntrySummary` exposes booleans
   (`has_totp`, `has_passkey`), never seeds or passwords. Secrets cross one
   entry at a time through explicit getters.
2. **Every entry-scoped request carries the tab origin** where a face has
   one; the app validates it (matching rule §4) before answering. Enforcement
   lives in the trusted process, never only in the extension.
3. **`lock_database` never fails.** Locking is a right, not a privilege.

The current client scopes demonstrate the contract: shared read/TOTP/lock/ping
operations reach all three faces; passphrase generation belongs to desktop and
mobile entry editors; captured-entry saving belongs to the web extension.
All native transports terminate in `castellan-dispatch`; app shells contain no
operation match of their own.

## §3 Results and errors

`RpcResponse = { id, result?, error? }` — flat, exactly one payload set.
Every `RpcResult` uses the request's operation tag (`get_entries`, `get_totp`,
`generate_passphrase`, `save_entry`, `lock_database`, `ping`). That correlation
lets generated TypeScript derive `ResultFor<M>` with `Extract` and makes a
wrong success variant a client error rather than an empty UI value. Error
codes are stable strings: today `vault_locked`, `no_such_entry`,
`not_implemented`, `disconnected` (transport-side). The UI switches on codes;
the message is for humans.

## §4 Origin matching (normative)

`origin_matches(origin, candidate)`: strip scheme/path/query, lowercase;
exact host equality, or the *page* host is a single-label subdomain of the
*entry* host; ports must agree when both are present. Siblings never match;
parent pages never match a subdomain-only entry. One implementation in Rust;
the extension pre-filters with the WASM build of the same function.

## §5 Versioning and capabilities

`PROTOCOL_VERSION` (currently 2) is generated into TypeScript as well as
compiled into Rust — one number, two doors. Connections begin with
`ClientMessage::Hello`; the app answers `HostMessage::Hello` with its
version, vault status and capability list (`Capability`: autofill, totp,
passkeys, recovery_codes, beam). A version mismatch is input to negotiation,
never a disconnect: old extensions degrade to the methods both sides know.

## §6 Events

`HostMessage::Event` pushes `database_locked`, `database_unlocked`,
`entry_changed`. Push, not poll: lock state is live on every face, and the
stale-state failure class cannot exist.
