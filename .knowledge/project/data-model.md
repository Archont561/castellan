---
type: Reference
title: "Castellan — data model"
description: "Protocol types, the KDBX-to-entry mapping, and the rules that keep secrets out of lists."
tags:
  - data-model
  - protocol
  - kdbx
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: project/data-model
category: project
refs:
  - project/architecture
  - features/vault
  - research/keepass-rs
---
# Data model

## Protocol types (v1)

`RpcRequest = { id: u64 } & RpcMethod` (serde flatten). `RpcMethod` variants:
`get_entries { origin }`, `get_totp { entry_id, origin }`,
`generate_passphrase { words, separator, capitalize }`, `save_entry { entry }`,
`lock_database`, `ping`. `RpcResponse = { id, result?, error? }` with
`RpcResult` tagged by kind. `Hello` carries version + capability list.
Events: `database_locked`, `database_unlocked`, `entry_changed`.
All 13 types are generated to TypeScript (barrel + `version.ts`).

## The list rule (invariant)

`EntrySummary` carries `entry_id, title, username, url, has_totp, has_passkey,
tags` — **booleans for secret-adjacent facts, never values**. Secrets cross
one entry at a time via explicit getters, gated on origin validation. This
is the invariant that makes a compromised extension a bounded incident.

## KDBX mapping

| Concept | KDBX storage | Rule |
| --- | --- | --- |
| Entry | `Entry` | id = entry UUID string |
| TOTP seed | `otp` field (keepass-rs), `TOTP Seed` (KeePassXC convention) | read both, write `otp` |
| Passkey | `castellan.passkey` custom field (RP ID) + protected key material | must degrade to "weird custom field" in foreign clients |
| Recovery codes | `castellan.recovery.<n>` + `castellan.recovery.used` | structured, foreign-readable |
| Groups | `Group` tree | mapped to tags in projections |

## Passphrase generation

EFF-style wordlist, rejection-sampled word choice for uniformity over the
list size, capitalized + separated per request parameters. The wordlist
ships as crate data, not source, once the real list lands (task-6 note).
