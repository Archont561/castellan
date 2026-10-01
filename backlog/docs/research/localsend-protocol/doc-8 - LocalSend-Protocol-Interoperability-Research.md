---
id: doc-8
title: "LocalSend Protocol Interoperability Research"
type: research
created_date: '2026-10-01 21:35'
updated_date: '2026-10-01 21:35'
tags:
  - localsend
  - sync
  - research
---
# Research — interop with the real LocalSend protocol

**Question.** What exactly must Castellan implement to exchange files with
stock LocalSend installs, and what should ride its own layer instead?

## §1 The protocol (v2.2, published spec)

- **Defaults**: port 53317 (TCP) for the HTTP(S) API; UDP multicast group
  224.0.0.167:53317 for discovery. Both user-configurable; version string
  ("2.2") travels in the announce payload.
- **Discovery**: multicast announce JSON (alias, version, deviceModel,
  deviceType, fingerprint, port, protocol). The fingerprint field is ignored
  in HTTPS mode; certificate pinning replaces it. Manual IP entry is the
  fallback when multicast is blocked.
- **Transfer**: REST API — `/api/localsend/v2/prepare-upload` (metadata
  exchange, returns tokens), `/api/localsend/v2/upload?token=...`,
  `/register`, `/prepare-download` for the reverse direction. JSON payloads;
  binary streams for content.
- **Security**: HTTPS with self-signed certificates generated per device;
  the sender verifies the receiver's fingerprint out-of-band (the app shows
  it) — TOFU with an explicit first-contact confirmation.

## §2 What interops and what does not

File send/receive, device visibility: full interop, verified against
LocalSend's Android and desktop apps during m-3 development. LocalSend has
no concept of vaults, entries, or sync — **Castellan's pairing and delta
sync are a separate protocol on the same transport** (doc-6 Layer 2/3), and
LocalSend clients simply never see those endpoints.

## §3 Implementation notes for task-25

Rust stack: tokio + axum + rustls + rcgen (self-signed cert generation),
raw UDP multicast socket for announce. Pin certificates on first contact;
fingerprint changes are hard errors with UI, matching LocalSend's own
behavior. The 1 MB protocol cap from our IPC does not apply here — this
layer streams; our protocol layer paginates.
