---
id: TASK-29.2
title: WebAuthn ceremony protocol surface
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - passkeys
  - protocol
dependencies: []
parent_task_id: TASK-29
priority: high
ordinal: 9800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The spec-first slice of the flagship: the ceremony RPC methods and state machine as a reviewable specification under backlog/docs/specifications/webauthn before any implementation. Every request carries the requesting origin; the app-side validation rules (RP ID plus origin binding) are written down once and shared by the extension and both mobile providers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A spec doc under backlog/docs/specifications/webauthn defines every ceremony RPC method with request and response shapes
- [ ] #2 Every request carries the requesting origin and the app-side RP ID and origin validation rules are explicit
- [ ] #3 The ceremony state machine (create/assert/cancel/timeout/per-use verification) is specified as the one shared design across extension and mobile providers
- [ ] #4 The reviewed protocol is recorded as a decision entry before implementation of task-29 begins
<!-- AC:END -->
