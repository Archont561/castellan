---
type: Specification
title: "Castellan — device mesh"
description: "LocalSend interop as transport, PAKE pairing, encrypted LAN delta sync, entry beams."
tags:
  - sync
  - localsend
  - mesh
status: draft
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: features/device-mesh
category: features
refs:
  - research/localsend-protocol
  - security/threat-model
---
# Device mesh

The anti-cloud sync: devices that find each other, pair by QR, and sync
over the LAN. **No relay, no NAT traversal, no cloud — ever** (roadmap
do-not-cut #5; revisiting it requires a new decision record, not a task).

## Layer 1 — LocalSend interop (task-25)

The published v2.2 protocol: UDP multicast 224.0.0.167:53317 discovery,
HTTPS REST on :53317, self-signed certs with TOFU fingerprint pinning
(changes are hard errors with UI). Full interop with stock LocalSend apps —
the transport is proven against the world. Details:
[LocalSend research](../research/localsend-protocol.md).

## Layer 2 — pairing (task-26)

Per-install Ed25519 identity, QR with fingerprint + PAKE commitment,
SPAKE2-style exchange (vetted crate, never hand-rolled); the derived key
never crosses the wire. Pairings are named, rotatable, revocable from
either side.

## Layer 3 — vault sync (task-27)

Deltas derived from KDBX entry history, protocol-typed, end-to-end
encrypted under the pairing key. Per-entry LWW merge; same-field conflicts
become reviewable both-versions UI with explicit resolution. Idempotent by
property test; a pcap shows no readable metadata.

## Beams (task-28)

Time-limited single-entry transfer to a paired device: explicit acceptance,
countdown expiry, deletion on both ends, audited who/what/when (never the
value).

Status: **draft** — fully specified (`backlog/docs/specifications/device-mesh/doc-6`),
implementation not started (milestone m-3).
