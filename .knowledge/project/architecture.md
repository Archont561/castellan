---
type: Reference
title: "Castellan — architecture"
description: "Monorepo layout, the one-protocol principle, and the codegen flow that keeps TypeScript derived."
tags:
  - architecture
  - monorepo
  - codegen
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: project/architecture
category: project
refs:
  - infrastructure/codegen
  - infrastructure/monorepo
  - project/data-model
---
# Architecture

```
apps/
  desktop/          Tauri + SvelteKit (static) — the primary face
  mobile/           Tauri (Android/iOS) — same UI kit, provider shells
  extension/        WXT: Chrome MV3 + Firefox — Fob
crates/             Rust workspace (virtual root)
  protocol/         the message language + origin matching
  otp/              otpauth parsing, RFC 6238
  ipc/              4-byte LE framing, 1 MB cap, socket paths
  native-host/      castellan --native-host (byte pump)
  vault/            KDBX open/save/projection, passphrase gen
  wasm/             wasm-bindgen face of pure logic
  xtask/            codegen + repo automation
packages/           TS workspace
  protocol/         @castellan/protocol — generated TS + client
  core/             @castellan/core — client + session state
  ui/               @castellan/ui — shared Svelte components
  wasm/             @castellan/wasm — lazy wasm wrapper
```

## The one-protocol principle

Every face talks to the core through the same messages
([codegen](../infrastructure/codegen.md) derives the TS). The desktop uses
Tauri `invoke("rpc")`; the extension uses native messaging through the host;
all three end in the same dispatcher. A feature that bypasses the protocol
is a design error by definition.

## Data flow: one autofill request

1. Content script reports the page origin to the background service worker.
2. Extension pre-filters locally-known candidates using the **WASM build**
   of `origin_matches` (same Rust function, different target).
3. Background sends `get_entries { origin }` through the client.
4. The app dispatcher re-validates the origin **authoritatively**, projects
   matching entries (metadata + flags only), answers.
5. Fill happens on the user's gesture; the extension holds one password for
   the blink it takes to type it, never the vault.

## Why this shape

Decision records in `backlog/decisions/`: one-protocol (d-1), host-is-app
(d-2), KDBX + copy-aside (d-3), SvelteKit static (d-4), bun + virtual Rust
root (d-5), soft passkeys (d-6), naming (d-7). The setup was inherited from
geoquery/pixi-sandbox — the ledger and divergences are in
[monorepo setup](../infrastructure/monorepo.md).
