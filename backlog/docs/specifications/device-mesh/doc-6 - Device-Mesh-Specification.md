---
id: doc-6
title: "Device Mesh Specification"
type: specification
created_date: '2026-10-01 21:20'
updated_date: '2026-10-01 21:20'
tags:
  - sync
  - localsend
  - security
  - specification
---
# Specification — the device mesh

**Scope.** Devices that find each other, pair, and sync secrets over the LAN
with no cloud, ever (milestone m-3; tasks 25–28).

## §1 Layer 1: LocalSend interop (task-25)

Implement the published LocalSend protocol v2.2: UDP multicast discovery on
224.0.0.167:53317, HTTPS REST on 53317, self-signed certificates with
fingerprint pinning (TOFU — a changed fingerprint is a hard error with UI).
This layer carries *files* and proves the transport against real LocalSend
apps. Castellan's own payloads ride Layer 2.

## §2 Layer 2: pairing (task-26)

- Device identity: Ed25519 keypair generated per install, pinned at pairing.
- QR contains: identity fingerprint + PAKE commitment. Scanning device and
  showing device run an SPAKE2-style exchange; the derived key never crosses
  the wire (traffic-capture test).
- Pairings are named, rotatable, revocable from either side; revocation is
  noticed by the peer on next contact.

## §3 Layer 3: vault sync (task-27)

- **Deltas, not databases.** Change log derived from KDBX entry history;
  protocol-typed delta messages end-to-end encrypted under the pairing key.
- **Merge**: per-entry, last-writer-wins; conflicting same-field edits
  produce a reviewable conflict with both versions and explicit resolution.
- **Idempotence**: replaying a sync message produces no changes (property
  test).
- A pcap of a full sync shows no readable metadata beyond transport framing.

## §4 Beams (task-28)

Time-limited single-entry transfer to a paired device: receiver accepts
explicitly (sees sender + origin), content self-destructs after the
countdown, both ends audit who/what/when (never the value).

## §5 Non-goals

Cloud relay, NAT traversal, and sync over the public internet. The mesh is
LAN-shaped by design; if devices never meet, they never sync — and that is a
feature the positioning says out loud.
