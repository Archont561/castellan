---
type: Specification
title: "Castellan — native channel"
description: "Chromium-native-messaging framing end to end, the byte-pump host, manifests, and the association handshake."
tags:
  - ipc
  - native-messaging
  - extension
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: infrastructure/native-channel
category: infrastructure
refs:
  - faces/extension
  - security/threat-model
---
# Native channel

**No localhost TCP exists anywhere in Castellan** (decision-2) — the
KeePassHttp design is the catalog of sins this replaces.

## Framing

Chromium native messaging on every leg: 4-byte LE length + JSON payload,
1 MB cap host→browser (enforced in `castellan-ipc`; list answers paginate).
The socket protocol is native messaging verbatim, so the host parses
nothing — it is a byte pump.

## Components

- **Host** = the app binary, `castellan --native-host` (~120 lines, two pump
  threads). Spawned per browser connection; version drift between host and
  app is structurally impossible.
- **IPC server** in the desktop app: `$XDG_RUNTIME_DIR/castellan/castellan.sock`
  (0700) or `\\.\pipe\castellan`; same-user enforcement via peer credentials
  (SO_PEERCRED / named-pipe client PID); sessions multiplex by request id.
- **Manifests** the app writes per browser (HKCU registry, per-browser
  `NativeMessagingHosts/` dirs), all extension IDs, plus a Repair button
  (task-10).

## Association handshake (task-9, not yet built)

Extension generates an Ed25519 keypair → app shows "Chrome wants to
connect" → public key stored → every connection proves possession with a
nonce challenge. Unassociated extensions get silence, not an error banner.

## Lifecycle and the MV3 wrinkle

Either side's EOF closes the connection; the host exits. Browsers restart
MV3 service workers at will — reconnection is the transport's job and is
invisible to client code. Safari is the exception: `SafariWebExtensionHandler`
+ XPC in the containing app, same protocol, different transport.
