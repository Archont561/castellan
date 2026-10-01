---
type: Research
title: "Castellan — LocalSend protocol research"
description: "The v2.2 wire details Castellan implements for mesh transport interop."
tags:
  - localsend
  - sync
  - protocol
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: research/localsend-protocol
category: research
refs:
  - features/device-mesh
---
# LocalSend protocol research

*From the published v2.2 spec — LocalSend documents its protocol; that is
precisely why it was chosen over reverse-engineered alternatives.*

## Wire facts

- **Discovery**: UDP multicast group 224.0.0.167, port 53317. Announce JSON:
  alias, version, deviceModel, deviceType, fingerprint, port, protocol.
- **Transfer**: HTTPS REST on TCP 53317 — `/api/localsend/v2/prepare-upload`
  (metadata, returns tokens), `/upload?token=…`, `/register`,
  `/prepare-download` (reverse direction). JSON payloads; binary streams
  for content.
- **Security**: self-signed certificates per device; the sender verifies the
  receiver's fingerprint out-of-band (the app displays it) — TOFU with an
  explicit first-contact confirmation. The announce `fingerprint` field is
  ignored in HTTPS mode; certificate pinning replaces it.
- Both port and multicast group are user-configurable; manual IP entry is
  the fallback when multicast is blocked.

## Interop scope

File send/receive + device visibility: **full interop**, to be verified
against stock LocalSend Android and desktop apps during m-3. Vault pairing
and delta sync ride Castellan's own protocol on the same transport
([device mesh](../features/device-mesh.md)) — LocalSend clients never see
those endpoints.

## Implementation notes

tokio + axum + rustls + rcgen (self-signed generation); raw UDP socket for
announce; pin on first contact, hard-error on change. The 1 MB cap from
`castellan-ipc` does not apply here — this layer streams; the protocol
layer paginates.
