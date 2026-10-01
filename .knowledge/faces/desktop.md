---
type: Reference
title: "Castellan — desktop face"
description: "Tauri + SvelteKit static inside the webview; one rpc command; the platform shell's duties."
tags:
  - desktop
  - tauri
  - sveltekit
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: faces/desktop
category: faces
refs:
  - project/architecture
  - infrastructure/native-channel
  - features/vault
---
# Desktop face

Tauri 2 + SvelteKit with `adapter-static`, `ssr = false`, SPA fallback for
dynamic routes (`/entry/[id]`), prerendered static shells. No server code
exists anywhere in a face (decision-4): the webview loads local files, and
every byte of data crosses `invoke("rpc")`.

## The single command

One Tauri command — `rpc(request: RpcRequest) -> RpcResponse` — plus event
pushes. Face logic never links vault crates; it links `@castellan/protocol`
and `@castellan/core`. This is what keeps the webview substitutable: any UI
kit could sit on the same command.

## The shell's duties (src-tauri)

- IPC socket server (UDS / named pipe) multiplexing native-messaging hosts —
  see [native channel](../infrastructure/native-channel.md).
- Window/tray/hotkey/autostart platform integration.
- Session state machine host: unlock, lock tiers, timers (the logic lives
  in Rust crates; the shell wires OS events into it).
- Biometric unwrap via the platform plugin when enabled
  ([biometric unlock](../security/biometric-unlock.md)).

## Known platform notes

Linux webkit2gtk quirks drive Tauri's system-webview variance — test on
WebKit, not just Chromium. macOS notarization from the first signed build.
Windows: SmartScreen tolerance for unsigned builds until a cert is bought.
