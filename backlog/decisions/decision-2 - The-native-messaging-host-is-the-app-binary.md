---
id: decision-2
title: "The native-messaging host is the app binary, not a proxy"
date: '2026-10-01 20:10'
status: accepted
---
## Context

KeePass's browser integration historically ran an HTTP server on localhost plus a separate
proxy binary. The failure modes are catalogued: firewall prompts and port conflicts from
the socket, any local process able to query the server, and version drift between app,
proxy and extension — three components that must update in lockstep but ship separately.

## Decision

**The host is `castellan --native-host`** — the app binary itself, spawned by each browser,
pumping frames between the browser's stdio and the app's IPC socket
(`crates/native-host`, ~100 lines, parses nothing). Framing is Chromium's native-messaging
wire format on both sides, so the host is a byte pump. No localhost TCP exists anywhere.

The IPC socket is a user-scoped Unix domain socket / Windows named pipe
(`crates/ipc`), same-user verified, with the association handshake (extension keypair +
app-side confirmation) specified in doc-2 but not yet implemented.

## Consequences

- App/proxy version drift is structurally impossible; the host can never be the wrong
  version for the vault it fronts.
- The app must ship a CLI arg surface (`--native-host`) from v0.1, which the CLI task
  (task-34) reuses.
- Byte-pump framing means the socket protocol *is* native messaging — no translation layer
  to design, and the 1 MB host→browser cap is enforced once, in `castellan-ipc`.
