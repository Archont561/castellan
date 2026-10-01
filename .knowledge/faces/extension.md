---
type: Reference
title: "Castellan — extension face (Fob)"
description: "WXT-built MV3/Firefox companion: autofill, capture, passkey interception — and what it must never hold."
tags:
  - extension
  - fob
  - browser
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: faces/extension
category: faces
refs:
  - infrastructure/native-channel
  - security/threat-model
  - security/passkeys
---
# Extension face — Fob

WXT-based extension, one codebase for Chrome MV3 and Firefox (per-engine
quirks isolated in adapters). Everything crosses
[@castellan/core](../project/architecture.md) → native messaging → the app.

## What it does

- **Autofill**: origin detection, candidate offer, gesture-gated fill;
  multi-step logins; save/update prompts with diff review.
- **Capture**: new/changed credentials, recovery-code screens
  (site-specific + generic heuristics, always user-reviewed), otpauth setup
  QRs bound to the entry just saved.
- **Passkey interception** (m-4): `navigator.credentials` override at
  `document_start` in the MAIN world (MV3 `world: "MAIN"`, Firefox injected
  script technique), ceremonies routed to the app.
- Pre-filtering with the WASM build of `origin_matches` — the *same
  function* the app applies authoritatively.

## What it must never hold (the containment contract)

1. The vault, any KDBX byte, any database handle.
2. More than one secret at a time, for the moment of a fill.
3. TOTP seeds (it asks `get_totp` and gets a code + countdown).
4. The authority to decide an origin matches (it can only *ask*).

A compromised extension under these rules leaks at most: entry metadata for
origins it asked about, and single passwords it filled. The
[threat model](../security/threat-model.md) draws the boundary formally.

## Manifests and connection

Association handshake (task-9) + app-written native-messaging manifests with
a Repair button (task-10) — the two features that delete the entire
"extension cannot connect" forum genre.
