---
id: TASK-25
title: "LocalSend protocol v2 interop: discovery and transfer"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - sync
  - protocol
dependencies: []
references:
  - backlog/docs/research/localsend-protocol/doc-8
priority: high
ordinal: 2500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Speak LocalSend's actual protocol (v2.2: UDP multicast 224.0.0.167:53317 discovery, HTTPS on 53317 with self-signed certs and fingerprint pinning) so Castellan exchanges files with real LocalSend installs — the transfer layer is proven against the world, not just against itself.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Castellan appears in a real LocalSend app's device list, and vice versa
- [ ] #2 File send/receive both work against LocalSend clients (Android + desktop) including multi-file
- [ ] #3 Self-signed cert fingerprints pinned on first contact (TOFU); a changed fingerprint is a hard error with UI
- [ ] #4 Port/multicast configurable, protocol version negotiated from the announce payload
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Rust module on tokio+rustls+rcgen; conformance against the published protocol spec (it is a documented REST API, not a reverse-engineered one).
<!-- SECTION:PLAN:END -->
