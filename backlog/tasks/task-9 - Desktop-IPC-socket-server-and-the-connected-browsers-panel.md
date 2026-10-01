---
id: TASK-9
title: "Desktop IPC socket server and the connected-browsers panel"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - ipc
  - desktop
  - security
dependencies: [TASK-4]
references:
  - crates/ipc
  - apps/desktop/src-tauri/src/lib.rs
priority: high
ordinal: 900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The app-side half of the native channel: a UDS/named-pipe server multiplexing host connections into sessions, the association handshake (extension keypair, app-side confirm, nonce challenge per connection), and the UI panel that shows connected faces — making a broken integration visible instead of silent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Multiple simultaneous host connections (two browsers, one profile each) multiplex cleanly by request id
- [ ] #2 Unassociated extensions get silence; the app UI prompts on first association and lists remembered keys
- [ ] #3 Same-user enforcement via peer credentials (SO_PEERCRED/LOCAL_PEERCRED/pipe PID) — documented and tested
- [ ] #4 Connected-browsers panel shows face, version, last request; kill switch per connection
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Server in the desktop app crate over castellan-ipc framing; association state in a small app-side store; Windows named-pipe branch lands here (task-4 left the stub).
<!-- SECTION:PLAN:END -->
